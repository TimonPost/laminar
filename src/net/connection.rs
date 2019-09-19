pub use crate::net::{managers::ConnectionManager, NetworkQuality, RttMeasurer, VirtualConnection};
use crate::{
    config::Config,
    either::Either,
    net::events::{ConnectionEvent, DestroyReason, DisconnectReason, ReceiveEvent},
    net::managers::{ConnectionManagerError, ConnectionState, SocketManager},
    net::socket::SocketWithConditioner,
    packet::Outgoing,
    ErrorKind,
};

use crossbeam_channel::{self, SendError, Sender};

use std::{
    collections::HashMap,
    net::SocketAddr,
    time::{Duration, Instant},
};

/// Maintains a registry of active "connections". Essentially, when we receive a packet on the
/// socket from a particular `SocketAddr`, we will track information about it here.
#[derive(Debug)]
pub struct ActiveConnections {
    connections: HashMap<SocketAddr, VirtualConnection>,
}

impl ActiveConnections {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
        }
    }

    /// Try to get a `VirtualConnection` by address. If the connection does not exist, it will be
    /// inserted and returned.
    pub fn get_or_insert_connection(
        &mut self,
        address: SocketAddr,
        config: &Config,
        time: Instant,
        state_manager: Box<dyn ConnectionManager>,
    ) -> &mut VirtualConnection {
        self.connections
            .entry(address)
            .or_insert_with(|| VirtualConnection::new(address, config, time, state_manager))
    }

    /// Returns `VirtualConnection` or None if it doesn't exists for a given address.
    pub fn try_get(&mut self, address: &SocketAddr) -> Option<&mut VirtualConnection> {
        self.connections.get_mut(address)
    }

    /// Removes the connection from `ActiveConnections` by socket address.
    pub fn remove_connection(
        &mut self,
        address: &SocketAddr,
        sender: &Sender<ConnectionEvent<ReceiveEvent>>,
        manager: &mut dyn SocketManager,
        reason: DestroyReason,
        error_context: &str,
    ) -> bool {
        if let Some((_, conn)) = self.connections.remove_entry(address) {
            if let ConnectionState::Connected(_) = conn.get_current_state() {
                if let Err(err) = sender.send(ConnectionEvent(
                    conn.remote_address,
                    ReceiveEvent::Disconnected(DisconnectReason::Destroying(reason.clone())),
                )) {
                    manager.track_connection_error(
                        &conn.remote_address,
                        &ErrorKind::SendError(SendError(Either::Right(err.0))),
                        error_context,
                    );
                }
            }
            if let Err(err) = sender.send(ConnectionEvent(
                conn.remote_address,
                ReceiveEvent::Destroyed(reason),
            )) {
                manager.track_connection_error(
                    &conn.remote_address,
                    &ErrorKind::SendError(SendError(Either::Right(err.0))),
                    error_context,
                );
            }
            manager.track_connection_destroyed(address);
            true
        } else {
            false
        }
    }

    /// Check for and return `VirtualConnection`s which have been idling longer than `max_idle_time`.
    pub fn idle_connections(&mut self, max_idle_time: Duration, time: Instant) -> Vec<SocketAddr> {
        self.connections
            .iter()
            .filter(|(_, connection)| connection.last_heard(time) >= max_idle_time)
            .map(|(address, _)| *address)
            .collect()
    }

    /// Get a list of addresses of dead connections
    pub fn dead_connections(&mut self) -> Vec<(SocketAddr, DestroyReason)> {
        self.connections
            .iter()
            .filter_map(|(_, connection)| {
                if connection.should_be_dropped() {
                    Some((
                        connection.remote_address,
                        DestroyReason::TooManyPacketsInFlight,
                    ))
                } else if connection.is_disconnected() {
                    Some((
                        connection.remote_address,
                        DestroyReason::GracefullyDisconnected,
                    ))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Check for and return `VirtualConnection`s which have not sent anything for a duration of at least `heartbeat_interval`.
    pub fn heartbeat_required_connections(
        &mut self,
        heartbeat_interval: Duration,
        time: Instant,
        manager: &mut dyn SocketManager,
        socket: &mut SocketWithConditioner,
    ) {
        self.connections
            .iter_mut()
            .filter(move |(_, connection)| connection.last_sent(time) >= heartbeat_interval)
            .for_each(|(_, connection)| {
                let packet = connection.create_and_process_heartbeat(time);
                socket.send_packet_and_log(
                    &connection.remote_address,
                    connection.state_manager.as_mut(),
                    &packet.contents(),
                    manager,
                    "sending heartbeat packet",
                );
            });
    }

    /// Calls `update` method for `ConnectionManager`, in the loop, until it returns None
    /// These updates returns either new packets to be sent, or connection state changes.
    pub fn update_connection_manager(
        conn: &mut VirtualConnection,
        sender: &Sender<ConnectionEvent<ReceiveEvent>>,
        manager: &mut dyn SocketManager,
        socket: &mut SocketWithConditioner,
        time: Instant,
        buffer: &mut [u8],
    ) {
        while let Some(changes) = conn.state_manager.update(buffer, time) {
            match changes {
                Ok(event) => match event {
                    Either::Left(packet) => {
                        match conn.process_outgoing(
                            packet.packet_type,
                            packet.payload,
                            packet.delivery,
                            packet.ordering,
                            None,
                            time,
                        ) {
                            Ok(packet) => {
                                if let Outgoing::Packet(outgoing) = packet {
                                    socket.send_packet_and_log(
                                        &conn.remote_address,
                                        conn.state_manager.as_mut(),
                                        &outgoing.contents(),
                                        manager,
                                        "sending packet from connection manager",
                                    );
                                } else {
                                    manager.track_connection_error(
                                        &conn.remote_address,
                                        &ErrorKind::ConnectionError(ConnectionManagerError::Fatal(
                                            String::from(
                                                "connection manager cannot send fragmented packets",
                                            ),
                                        )),
                                        "sending packet from connection manager",
                                    );
                                }
                            }
                            Err(err) => manager.track_connection_error(
                                &conn.remote_address,
                                &err,
                                "sending packet from connection manager",
                            ),
                        }
                    }
                    Either::Right(state) => {
                        if let Some(old) = conn.current_state.try_change(&state) {
                            if let Err(err) = match &conn.current_state {
                                ConnectionState::Connected(data) => sender.send(ConnectionEvent(
                                    conn.remote_address,
                                    ReceiveEvent::Connected(data.clone()),
                                )),
                                ConnectionState::Disconnected(closed_by) => {
                                    sender.send(ConnectionEvent(
                                        conn.remote_address,
                                        ReceiveEvent::Disconnected(DisconnectReason::ClosedBy(
                                            closed_by.clone(),
                                        )),
                                    ))
                                }
                                _ => {
                                    manager.track_connection_error(
                                        &conn.remote_address,
                                        &ErrorKind::ConnectionError(ConnectionManagerError::Fatal(
                                            format!(
                                                "Invalid state transition: {:?} -> {:?}",
                                                old, conn.current_state
                                            ),
                                        )),
                                        "changing connection manager state",
                                    );
                                    Ok(())
                                }
                            } {
                                manager.track_connection_error(
                                    &conn.remote_address,
                                    &ErrorKind::SendError(SendError(Either::Right(err.0))),
                                    "sending connection state update",
                                );
                            }
                        } else {
                            manager.track_connection_error(
                                &conn.remote_address,
                                &ErrorKind::ConnectionError(ConnectionManagerError::Fatal(
                                    format!(
                                        "Invalid state transition: {:?} -> {:?}",
                                        conn.current_state, state
                                    ),
                                )),
                                "changing connection manager state",
                            );
                        }
                    }
                },
                Err(err) => {
                    manager.track_connection_error(
                        &conn.remote_address,
                        &ErrorKind::ConnectionError(err),
                        "recieved connection manager error",
                    );
                }
            }
        }
    }

    /// Iterates through all active connections, and `update`s each connection manager.
    pub fn update_connections(
        &mut self,
        sender: &Sender<ConnectionEvent<ReceiveEvent>>,
        manager: &mut dyn SocketManager,
        socket: &mut SocketWithConditioner,
        time: Instant,
        buffer: &mut [u8],
    ) {
        self.connections.iter_mut().for_each(|(_, conn)| {
            ActiveConnections::update_connection_manager(
                conn, sender, manager, socket, time, buffer,
            )
        });
    }

    /// Returns the number of connected clients.
    #[cfg(test)]
    pub(crate) fn count(&self) -> usize {
        self.connections.len()
    }
}

#[cfg(test)]
mod tests {

    use super::{ActiveConnections, Config};
    use std::{
        sync::Arc,
        time::{Duration, Instant},
    };

    use super::managers::ConnectionManager;

    #[derive(Debug)]
    struct DummyConnManager {}

    impl ConnectionManager for DummyConnManager {}

    const ADDRESS: &str = "127.0.0.1:12345";

    #[test]
    fn connection_timed_out() {
        let mut connections = ActiveConnections::new();
        let config = Config::default();

        let now = Instant::now();

        // add 10 clients
        for i in 0..10 {
            connections.get_or_insert_connection(
                format!("127.0.0.1:122{}", i).parse().unwrap(),
                &config,
                now,
            );
        }

        assert_eq!(connections.count(), 10);

        let wait = Duration::from_millis(200);

        #[cfg(not(windows))]
        let epsilon = Duration::from_nanos(1);
        #[cfg(windows)]
        let epsilon = Duration::from_millis(1);

        let timed_out_connections = connections.idle_connections(wait, now + wait - epsilon);
        assert_eq!(timed_out_connections.len(), 0);

        let timed_out_connections = connections.idle_connections(wait, now + wait + epsilon);
        assert_eq!(timed_out_connections.len(), 10);
    }

    #[test]
    fn insert_connection() {
        let mut connections = ActiveConnections::new();
        let config = Config::default();

        let address = ADDRESS.parse().unwrap();
        connections.get_or_insert_connection(address, &config, Instant::now());
        assert!(connections.connections.contains_key(&address));
    }

    #[test]
    fn insert_existing_connection() {
        let mut connections = ActiveConnections::new();
        let config = Config::default();

        let address = ADDRESS.parse().unwrap();
        connections.get_or_insert_connection(address, &config, Instant::now());
        assert!(connections.connections.contains_key(&address));
        connections.get_or_insert_connection(address, &config, Instant::now());
        assert!(connections.connections.contains_key(&address));
    }

    #[test]
    fn remove_connection() {
        let mut connections = ActiveConnections::new();
        let config = Arc::new(Config::default());

        let address = ADDRESS.parse().unwrap();
        connections.get_or_insert_connection(address, &config, Instant::now());
        assert!(connections.connections.contains_key(&address));
        connections.remove_connection(&address);
        assert!(!connections.connections.contains_key(&address));
    }

    #[test]
    fn remove_non_existent_connection() {
        let mut connections = ActiveConnections::new();

        let address = &ADDRESS.parse().unwrap();
        connections.remove_connection(address);
        assert!(!connections.connections.contains_key(address));
    }
}
