use super::{Connection, ConnectionsCollection, VirtualConnection};
use crate::config::NetworkConfig;
use crate::error::{NetworkError, NetworkErrorKind, NetworkResult};
use crate::events::Event;
use log::{error, info};
use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::mpsc::Sender;
use std::sync::{Arc, RwLock};
use std::time::Duration;

/// This is a pool of virtual connections (connected) over UDP.
pub struct ConnectionPool {
    connections: ConnectionsCollection,
    config: Arc<NetworkConfig>,
}

impl ConnectionPool {
    pub fn new(config: Arc<NetworkConfig>) -> ConnectionPool {
        ConnectionPool {
            connections: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Try getting connection by address if the connection does not exists it will be inserted.
    pub fn get_connection_or_insert(&self, addr: &SocketAddr) -> NetworkResult<Connection> {
        let lock = self
            .connections
            .read()
            .map_err(|error| NetworkError::poisoned_connection_error(error.description()))?;

        if lock.contains_key(addr) {
            match lock.get(addr) {
                Some(connection) => Ok(connection.clone()),
                None => Err(NetworkErrorKind::ConnectionPoolError(String::from(
                    "Could not get connection from connection pool",
                ))
                .into()),
            }
        } else {
            drop(lock);

            let mut lock = self
                .connections
                .write()
                .map_err(|error| NetworkError::poisoned_connection_error(error.description()))?;

            let connection = lock.entry(*addr).or_insert_with(|| {
                Arc::new(RwLock::new(VirtualConnection::new(
                    *addr,
                    self.config.clone(),
                )))
            });

            Ok(connection.clone())
        }
    }

    /// Removes the connection from connection pool by socket address.
    pub fn remove_connection(
        &self,
        addr: &SocketAddr,
    ) -> NetworkResult<Option<(SocketAddr, Arc<RwLock<VirtualConnection>>)>> {
        let mut lock = self
            .connections
            .write()
            .map_err(|error| NetworkError::poisoned_connection_error(error.description()))?;

        Ok(lock.remove_entry(addr))
    }

    /// Check if there are any connections that have not been active for the given Duration.
    /// And returns a vector of clients been idling for to long.
    pub fn check_for_timeouts(
        &self,
        sleepy_time: Duration,
        events_sender: &Sender<Event>,
    ) -> NetworkResult<Vec<SocketAddr>> {
        let mut timed_out_clients: Vec<SocketAddr> = Vec::new();

        match self.connections.read() {
            Ok(ref connections) => {
                for (key, value) in connections.iter() {
                    if let Ok(connection) = value.read() {
                        if connection.last_heard() >= sleepy_time {
                            timed_out_clients.push(key.clone());
                            let event = Event::TimedOut(value.clone());

                            events_sender
                                .send(event)
                                .map_err(|e| NetworkErrorKind::PoisonedLock(format!("Error when trying to send timeout event over channel. Reason: {}", e)))?;

                            info!("Client has timed out: {:?}", key);
                        }
                    }
                }
            }
            Err(e) => {
                error!("Error when checking for timed out connections: {:?}", e);
                return Err(NetworkErrorKind::PoisonedLock(format!(
                    "Error when checking for timed out connections: {:?}",
                    e
                ))
                .into());
            }
        }

        Ok(timed_out_clients)
    }

    /// Get the number of connected clients.
    /// This function could fail because it needs to acquire a read lock before it can know the size.
    #[allow(dead_code)]
    pub fn count(&self) -> NetworkResult<usize> {
        let connections = self.connections.read()?;
        Ok(connections.len())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::channel;
    use std::thread;
    use std::time::Duration;

    use super::{Arc, ConnectionPool};
    use crate::config::NetworkConfig;
    use crate::events::Event;

    #[test]
    fn connection_timed_out() {
        let connections = Arc::new(ConnectionPool::new(Arc::new(NetworkConfig::default())));
        let (tx, rx) = channel();

        // add 10 clients
        for i in 0..10 {
            connections
                .get_connection_or_insert(&(format!("127.0.0.1:123{}", i).parse().unwrap()))
                .unwrap();
        }

        assert_eq!(connections.count().unwrap(), 10);

        // Sleep a little longer than te polling interval.
        thread::sleep(Duration::from_millis(700));

        let timed_out_connections = connections
            .check_for_timeouts(Duration::from_millis(500), &tx)
            .unwrap();

        // We should have received 10 timeouts event by now.
        let mut events_received = 0;
        while let Ok(event) = rx.try_recv() {
            match event {
                Event::TimedOut(_) => {
                    events_received += 1;
                }
                _ => {}
            }
        }

        assert_eq!(timed_out_connections.len(), 10);
        assert_eq!(events_received, 10);
    }

    #[test]
    fn insert_connection() {
        let connections = ConnectionPool::new(Arc::new(NetworkConfig::default()));

        let addr = &("127.0.0.1:12345".parse().unwrap());
        connections.get_connection_or_insert(addr).unwrap();
        assert!(connections.connections.read().unwrap().contains_key(addr));
    }

    #[test]
    fn insert_existing_connection() {
        let connections = ConnectionPool::new(Arc::new(NetworkConfig::default()));

        let addr = &("127.0.0.1:12345".parse().unwrap());
        connections.get_connection_or_insert(addr).unwrap();
        assert!(connections.connections.read().unwrap().contains_key(addr));
        connections.get_connection_or_insert(addr).unwrap();
        assert!(connections.connections.read().unwrap().contains_key(addr));
    }

    #[test]
    fn removes_connection() {
        let connections = ConnectionPool::new(Arc::new(NetworkConfig::default()));

        let addr = &("127.0.0.1:12345".parse().unwrap());
        connections.get_connection_or_insert(addr).unwrap();
        assert!(connections.connections.read().unwrap().contains_key(addr));
        connections.remove_connection(addr).unwrap();
        assert!(!connections.connections.read().unwrap().contains_key(addr));
    }

    #[test]
    fn remove_not_existing_connection() {
        let connections = ConnectionPool::new(Arc::new(NetworkConfig::default()));

        let addr = &("127.0.0.1:12345".parse().unwrap());
        connections.remove_connection(addr).unwrap();
        assert!(!connections.connections.read().unwrap().contains_key(addr));
    }
}
