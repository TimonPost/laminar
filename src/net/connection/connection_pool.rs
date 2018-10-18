use super::VirtualConnection;
use error::{NetworkError, Result};
use events::Event;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::mpsc::Sender;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

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
    pub fn get_connection_or_insert(&mut self, addr: &SocketAddr) -> Result<Connection> {
        let mut lock = self
            .connections
            .write()
            .map_err(|_| NetworkError::AddConnectionToManagerFailed)?;

        let connection = lock
            .entry(*addr)
            .or_insert_with(|| Arc::new(RwLock::new(VirtualConnection::new(*addr))));

        Ok(connection.clone())
    }

    /// Start loop that will detect if connections will are disconnected.
    ///
    /// This function starts a background thread that does the following:
    /// 1. Gets a read lock on the HashMap containing all the connections
    /// 2. Iterate through each one
    /// 3. Check if the last time we have heard from them (received a packet from them) is greater than the amount of time considered to be a timeout
    /// 4. If they have timed out, send a notification up the stack
    pub fn start_time_out_loop(
        &self,
        events_sender: Sender<Event>,
    ) -> Result<thread::JoinHandle<()>> {
        let connections = self.connections.clone();
        let poll_interval = self.poll_interval;

        let sender = events_sender.clone();
        Ok(thread::Builder::new()
            .name("check_for_timeouts".into())
            .spawn(move || loop {
                match connections.read() {
                    Ok(lock) => {
                        ConnectionPool::check_for_timeouts(&*lock, poll_interval, &sender);
                    }
                    Err(e) => {
                        error!("Unable to acquire read lock to check for timed out connections")
                    }
                }
                thread::sleep(poll_interval);
            })?)
    }

    /// Check if there are any connections that have not been active for the given Duration.
    fn check_for_timeouts(
        connections: &Connections,
        sleepy_time: Duration,
        events_sender: &Sender<Event>,
    ) {
        for (key, value) in connections.iter() {
            if let Ok(c) = value.read() {
                if c.last_heard() >= sleepy_time {
                    let event = Event::TimedOut(value.clone());

                    events_sender
                        .send(event)
                        .expect("Unable to send disconnect event");

                    error!("Client has timed out: {:?}", key);
                }
            }
        }
    }
}

#[cfg(tests)]
mod test {
    use std::sync::mpsc::channel;
    use std::thread;
    use std::time::Duration;

    use super::{Arc, ConnectionPool, Mutex, TIMEOUT_POLL_INTERVAL};
    use events::Event;
    use net::connection::VirtualConnection;

    #[test]
    fn connection_timed_out() {
        let (tx, rx) = channel();

        let mut connections = ConnectionPool::new();
        let handle = connections.start_time_out_loop(tx.clone()).unwrap();

        connections.get_connection_or_insert(&("127.0.0.1:12345".parse().unwrap()));

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
    }

    #[test]
    fn insert_connection() {
        let mut connections = ConnectionPool::new();

        let addr = &("127.0.0.1:12345".parse().unwrap());
        connections.get_connection_or_insert(addr);
        assert!(connections.connections.read().unwrap().contains_key(addr));
    }
}
