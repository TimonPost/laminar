use super::VirtualConnection;
use error::{NetworkResult, NetworkError};
use events::Event;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::mpsc::Sender;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;
use std::error::Error;

pub type Connection = Arc<RwLock<VirtualConnection>>;
pub type Connections = HashMap<SocketAddr, Connection>;
pub type ConnectionsCollection = Arc<RwLock<Connections>>;

// Default time between checks of all clients for timeouts in seconds
const TIMEOUT_POLL_INTERVAL: u64 = 1;

/// This is a pool of virtual connections (connected) over UDP.
pub struct ConnectionPool {
    timeout: Duration,
    connections: ConnectionsCollection,
    sleepy_time: Duration,
    poll_interval: Duration,
}

impl ConnectionPool {
    pub fn new() -> ConnectionPool {
        let sleepy_time = Duration::from_secs(1);
        let poll_interval = Duration::from_secs(TIMEOUT_POLL_INTERVAL);

        ConnectionPool {
            timeout: Duration::from_secs(1),
            connections: Arc::new(RwLock::new(HashMap::new())),
            sleepy_time,
            poll_interval,
        }
    }

    /// Set disconnect timeout (duration after which a client is seen as disconnected).
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    /// Insert connection if it does not exists.
    pub fn get_connection_or_insert(&mut self, addr: &SocketAddr) -> NetworkResult<Connection> {
        let mut lock = self
            .connections
            .write()
            .map_err(|error| NetworkError::poisoned_connection_error(error.description()))?;

        let connection = lock
            .entry(*addr)
            .or_insert_with(|| Arc::new(RwLock::new(VirtualConnection::new(*addr))));

        Ok(connection.clone())
    }

    // Get the number of connected clients.
    pub fn count(&self) -> usize {
        match self.connections.read() {
            Ok(connections) => { connections.len() },
            Err(_) => { 0 },
        }
    }

    /// Start loop that detects when a connection has timed out.
    ///
    /// This function starts a background thread that does the following:
    /// 1. Gets a read lock on the HashMap containing all the connections
    /// 2. Iterate through each one
    /// 3. Check if the last time we have heard from them (received a packet from them) is greater than the amount of time considered to be a timeout
    /// 4. If they have timed out, send a notification up the stack
    pub fn start_time_out_loop(
        &self,
        events_sender: Sender<Event>,
    ) -> NetworkResult<thread::JoinHandle<()>> {
        let connections = self.connections.clone();
        let poll_interval = self.poll_interval;

        Ok(thread::Builder::new()
            .name("check_for_timeouts".into())
            .spawn(move || loop {
                let timed_out_clients = ConnectionPool::check_for_timeouts(&connections, poll_interval, &events_sender);

                if timed_out_clients.len() > 0 {
                    match connections.write() {
                        Ok(ref mut connections) => {
                            for timed_out_client in timed_out_clients {
                                connections.remove(&timed_out_client);
                            }
                        }
                        Err(e) => {
                            panic!("Error when checking for timed out connections: {}", e)
                        }
                    }
                }

                thread::sleep(poll_interval);
            })?)
    }

    /// Check if there are any connections that have not been active for the given Duration.
    fn check_for_timeouts(
        connections: &ConnectionsCollection,
        sleepy_time: Duration,
        events_sender: &Sender<Event>,
    ) -> Vec<SocketAddr> {
        let mut timed_out_clients: Vec<SocketAddr> = Vec::new();

        match connections.read() {
            Ok(ref connections) => {
                for (key, value) in connections.iter() {
                    if let Ok(connection) = value.read() {
                        if connection.last_heard() >= sleepy_time {
                            timed_out_clients.push(key.clone());
                            let event = Event::TimedOut(value.clone());

                            events_sender
                                .send(event)
                                .expect("Unable to send disconnect event");

                            info!("Client has timed out: {:?}", key);
                        }
                    }
                }
            }
            Err(e) => {
                panic!("Error when checking for timed out connections: {}", e)
            }
        }

        timed_out_clients
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::channel;
    use std::thread;
    use std::time::Duration;

    use super::{Arc, ConnectionPool, TIMEOUT_POLL_INTERVAL};
    use events::Event;
    use net::connection::VirtualConnection;

    #[test]
    fn connection_timed_out() {
        let (tx, rx) = channel();

        let mut connections = ConnectionPool::new();
        let handle = connections.start_time_out_loop(tx.clone()).unwrap();

        connections.get_connection_or_insert(&("127.0.0.1:12345".parse().unwrap()));

        assert_eq!(connections.count(), 1);

        /// Sleep a little longer than te polling interval.
        thread::sleep(Duration::from_millis(TIMEOUT_POLL_INTERVAL * 1000 + 100));

        /// We should have the timeout event by now.
        match rx.try_recv() {
            Ok(event) => {
                match event {
                    Event::TimedOut(client) => {
                        assert_eq!(
                            client.read().unwrap().remote_address,
                            "127.0.0.1:12345".parse().unwrap()
                        );
                    }
                    _ => panic!("Didn't expect any other events than TimedOut."),
                };
            }
            Err(e) => panic!("No events found!"),
        };

        assert_eq!(connections.count(), 0);
    }

    #[test]
    fn insert_connection() {
        let mut connections = ConnectionPool::new();

        let addr = &("127.0.0.1:12345".parse().unwrap());
        connections.get_connection_or_insert(addr);
        assert!(connections.connections.read().unwrap().contains_key(addr));
    }
}
