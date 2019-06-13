pub use crate::net::{NetworkQuality, RttMeasurer, VirtualConnection};

use crate::config::Config;
use std::{
    collections::HashMap,
    net::SocketAddr,
    time::{Duration, Instant},
};

/// Maintains a registry of active "connections". Essentially, when we receive a packet on the
/// socket from a particular `SocketAddr`, we will track information about it here.
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

    /// Returns true if the given connection exists.
    pub fn exists(&self, address: &SocketAddr) -> bool {
        self.connections.contains_key(&address)
    }

    /// Returns the number of connected clients.
    #[cfg(test)]
    pub fn count(&self) -> usize {
        self.connections.len()
    }
}

#[cfg(test)]
mod tests {
    use super::{ActiveConnections, Config};
    use std::{
        sync::Arc,
        thread,
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
                format!("127.0.0.1:123{}", i).parse().unwrap(),
                &config,
                now,
            );
        }

        assert_eq!(connections.count(), 10);

        let wait = Duration::from_millis(200);

        let timed_out_connections =
            connections.idle_connections(wait, now + wait - Duration::from_nanos(1));
        assert_eq!(timed_out_connections.len(), 0);

        let timed_out_connections = connections.idle_connections(wait, now + wait);
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
