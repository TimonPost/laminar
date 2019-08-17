pub use crate::net::{NetworkQuality, RttMeasurer, VirtualConnection};

use crate::config::Config;
use crate::either::Either::{self, Left, Right};
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
    ) -> &mut VirtualConnection {
        self.connections
            .entry(address)
            .or_insert_with(|| VirtualConnection::new(address, config, time))
    }

    /// Try to get or create a [VirtualConnection] by address. If the connection does not exist, it will be
    /// created and returned, but not inserted into the table of active connections.
    pub(crate) fn get_or_create_connection(
        &mut self,
        address: SocketAddr,
        config: &Config,
        time: Instant,
    ) -> Either<&mut VirtualConnection, VirtualConnection> {
        if let Some(connection) = self.connections.get_mut(&address) {
            Left(connection)
        } else {
            Right(VirtualConnection::new(address, config, time))
        }
    }

    /// Removes the connection from `ActiveConnections` by socket address.
    pub fn remove_connection(
        &mut self,
        address: &SocketAddr,
    ) -> Option<(SocketAddr, VirtualConnection)> {
        self.connections.remove_entry(address)
    }

    /// Check for and return `VirtualConnection`s which have been idling longer than `max_idle_time`.
    pub fn idle_connections(&mut self, max_idle_time: Duration, time: Instant) -> Vec<SocketAddr> {
        self.connections
            .iter()
            .filter(|(_, connection)| connection.last_heard(time) >= max_idle_time)
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
