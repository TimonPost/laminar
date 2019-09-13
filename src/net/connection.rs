pub use crate::net::{NetworkQuality, RttMeasurer, VirtualConnection, managers::ConnectionManager};
use crate::{
    config::Config,
    either::Either::{self, Left, Right},
    net::events::{SocketEvent, DisconnectReason, DestroyReason},
    net::managers::{ConnectionState, SocketManager},
    packet::{Packet, OutgoingPacket},
    ErrorKind,
};
use crossbeam_channel::{self, Sender, SendError};

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
    buffer: Box<[u8]>
}

impl ActiveConnections {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
            // TODO actually take from config
            buffer: Box::new([0;1500])
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


    /// Try to get or create a [VirtualConnection] by address. If the connection does not exist, it will be
    /// created and returned, but not inserted into the table of active connections.
    pub(crate) fn get_or_create_connection(
        &mut self,
        address: SocketAddr,
        config: &Config,
        time: Instant,
        state_manager: Box<dyn ConnectionManager>,
    ) -> Either<&mut VirtualConnection, VirtualConnection> {
        if let Some(connection) = self.connections.get_mut(&address) {
            Left(connection)
        } else {
            Right(VirtualConnection::new(address, config, time, state_manager))
        }
    }

    pub fn try_get(&mut self, address: &SocketAddr) -> Option<&mut VirtualConnection> {
        self.connections.get_mut(address)
    }

    /// Removes the connection from `ActiveConnections` by socket address.
    pub fn remove_connection(
        &mut self,
        address: &SocketAddr,
        sender: &Sender<SocketEvent>,
        manager: &mut dyn SocketManager,
        reason: DestroyReason
    ) -> Result<bool, SendError<SocketEvent>> {
        if let Some((_, conn)) = self.connections.remove_entry(address) {
            manager.track_connection_destroyed(address);
            if let ConnectionState::Connected(_) = conn.get_current_state() {
                sender.send(SocketEvent::Disconnected(DisconnectReason::UnrecoverableError(reason.clone())))?;
            }
            sender.send(SocketEvent::Destroyed(reason))?;
            Ok(true)
        } else {
            Ok(false)
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
    pub fn dead_connections(&mut self) -> Vec<SocketAddr> {
        self.connections
            .iter()
            .filter(|(_, connection)| connection.should_be_dropped())
            .map(|(address, _)| *address)
            .collect()
    }

    /// Check for and return `VirtualConnection`s which have not sent anything for a duration of at least `heartbeat_interval`.
    pub fn heartbeat_required_connections(
        &mut self,
        heartbeat_interval: Duration,
        time: Instant,
    ) -> impl Iterator<Item = &mut VirtualConnection> {
        self.connections
            .iter_mut()
            .filter(move |(_, connection)| connection.last_sent(time) >= heartbeat_interval)
            .map(|(_, connection)| connection)
    }

    pub fn update_connections(
        &mut self,
        mut buffer: &mut [u8],
        sender: &Sender<SocketEvent>,
        manager:&mut dyn SocketManager
    ) -> Vec<(SocketAddr, Box<[u8]>)> {
        let time = Instant::now();
        self.connections
            .iter_mut()
            .filter_map(|(_, conn)| match conn.state_manager.update(&mut buffer, time) {
                Some(result) => match result {
                    Ok(event) => match event {
                        Either::Left(packet) => Some((conn.remote_address, packet.contents())),
                        Either::Right(state) => {
                            conn.current_state = state.clone();
                            if let Err(err) = match &conn.current_state {
                                    ConnectionState::Connected(data) => sender.send(SocketEvent::Connected(conn.remote_address, data.clone())),
                                    ConnectionState::Disconnected(closed_by) => sender.send(SocketEvent::Disconnected(DisconnectReason::ClosedBy(closed_by.clone()))),
                                    _ => Ok(()),} {
                                manager.track_global_error(&ErrorKind::SendError(err));
                            }
                            None
                        }
                    },
                    Err(err) => {
                        manager.track_connection_error(&conn.remote_address, &ErrorKind::ConnectionError(err));
                        None
                    }
                },
                None => None
            })
            .collect()
    }

    /// Returns true if the given connection exists.
    pub fn exists(&self, address: &SocketAddr) -> bool {
        self.connections.contains_key(&address)
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
    struct DummyConnManager {
    }

    impl ConnectionManager for DummyConnManager {

    }


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
