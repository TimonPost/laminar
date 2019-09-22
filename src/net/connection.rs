pub use crate::net::{
    managers::ConnectionManager, NetworkQuality, ReliabilitySystem, RttMeasurer, VirtualConnection,
};
use crate::{
    net::events::{ConnectionEvent, DestroyReason, DisconnectReason, ReceiveEvent},
    net::managers::ConnectionState,
    net::socket::SocketWithConditioner,
    net::MetricsCollector,
    ErrorKind,
};

use crossbeam_channel::{self, Sender};

use std::{
    collections::HashMap,
    net::SocketAddr,
    time::{Duration, Instant},
};

/// Maintains a registry of active "connections".
#[derive(Debug)]
pub struct ActiveConnections {
    connections: HashMap<SocketAddr, VirtualConnection>,
}

impl ActiveConnections {
    /// Initialized active connection list.
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
        }
    }

    /// Inserts new connection, and calls `update` method on `ConnectionManager` to initialized it.
    pub fn insert_and_init_connection(
        &mut self,
        connection: VirtualConnection,
        socket: &mut SocketWithConditioner,
        event_sender: &Sender<ConnectionEvent<ReceiveEvent>>,
        metrics: &mut MetricsCollector,
        time: Instant,
        tmp_buffer: &mut [u8],
    ) -> &mut VirtualConnection {
        let conn = self
            .connections
            .entry(connection.remote_address())
            .or_insert(connection);

        conn.update_connection_manager(event_sender, metrics, socket, time, tmp_buffer);
        metrics.track_connection_created(&conn.remote_address());
        if let Err(err) = event_sender.send(ConnectionEvent(
            conn.remote_address(),
            ReceiveEvent::Created,
        )) {
            metrics.track_connection_error(
                &conn.remote_address(),
                &ErrorKind::from(err),
                "sending connection create event",
            );
        }
        conn
    }

    /// Returns `VirtualConnection` or None if it doesn't exists for a given address.
    pub fn try_get(&mut self, address: &SocketAddr) -> Option<&mut VirtualConnection> {
        self.connections.get_mut(address)
    }

    /// Iterates through all active connections, and `update`s each connection manager.
    pub fn update_connections(
        &mut self,
        sender: &Sender<ConnectionEvent<ReceiveEvent>>,
        metrics: &mut MetricsCollector,
        socket: &mut SocketWithConditioner,
        time: Instant,
        buffer: &mut [u8],
    ) {
        self.connections.iter_mut().for_each(|(_, conn)| {
            conn.update_connection_manager(sender, metrics, socket, time, buffer)
        });
    }

    /// Iterate through all of the connections and check if any of them should be dropped.
    /// Remove dropped connections from the active connections. For each connection removed, we will send an event to the `event_sender` channel.
    pub fn handle_dead_clients(
        &mut self,
        time: Instant,
        idle_connection_timeout: Duration,
        sender: &Sender<ConnectionEvent<ReceiveEvent>>,
        metrics: &mut MetricsCollector,
    ) {
        let drop_list: Vec<_> = self
            .connections
            .iter_mut()
            .filter_map(|(_, connection)| {
                connection
                    .should_be_dropped(idle_connection_timeout, time)
                    .map(|reason| (connection.remote_address(), reason))
            })
            .collect();

        for (address, reason) in drop_list {
            self.remove_connection(&address, sender, metrics, reason, "removing dead clients");
        }
    }

    pub fn handle_heartbeat(
        &mut self,
        time: Instant,
        heartbeat_interval: Duration,
        socket: &mut SocketWithConditioner,
        metrics: &mut MetricsCollector,
    ) {
        // Iterate over all connections which have not sent a packet for a duration of at least
        // `heartbeat_interval` (from config), and send a heartbeat packet to each.
        let connections = self.connections.iter_mut();
        connections.for_each(|(_, connection)| {
            connection.handle_heartbeat(time, heartbeat_interval, socket, metrics);
        });
    }

    /// Removes the connection from `ActiveConnections` by socket address, and sends appropriate events.
    fn remove_connection(
        &mut self,
        address: &SocketAddr,
        sender: &Sender<ConnectionEvent<ReceiveEvent>>,
        metrics: &mut MetricsCollector,
        reason: DestroyReason,
        error_context: &str,
    ) -> bool {
        if let Some((_, conn)) = self.connections.remove_entry(address) {
            if let ConnectionState::Connected(_) = conn.get_current_state() {
                if let Err(err) = sender.send(ConnectionEvent(
                    conn.remote_address(),
                    ReceiveEvent::Disconnected(DisconnectReason::Destroying(reason.clone())),
                )) {
                    metrics.track_connection_error(
                        &conn.remote_address(),
                        &ErrorKind::from(err),
                        error_context,
                    );
                }
            }
            if let Err(err) = sender.send(ConnectionEvent(
                conn.remote_address(),
                ReceiveEvent::Destroyed(reason),
            )) {
                metrics.track_connection_error(
                    &conn.remote_address(),
                    &ErrorKind::from(err),
                    error_context,
                );
            }
            metrics.track_connection_destroyed(address);
            true
        } else {
            false
        }
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
