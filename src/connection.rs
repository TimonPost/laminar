use amethyst_error::AmethystNetworkError;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::string::ToString;
use std::time::{Duration, Instant};

// Type aliases
// Number of seconds we will wait until we consider a Connection to have timed out
type ConnectionTimeout = u8;

// Default timeout of 10 seconds
const TIMEOUT_DEFAULT: ConnectionTimeout = 10;

/// Maintains a list of all Connections and allows adding/removing them
#[derive(Default)]
pub struct Manager {
    // The collection of currently connected clients
    connections: HashMap<String, Connection>,
    // The number of seconds before we consider a client to have timed out
    timeout: ConnectionTimeout,
}

impl Manager {
    pub fn new() -> Manager {
        Manager {
            connections: HashMap::new(),
            timeout: TIMEOUT_DEFAULT,
        }
    }

    pub fn with_client_timeout(mut self, timeout: ConnectionTimeout) -> Manager {
        self.timeout = timeout;
        self
    }

    // Adds a new connection to the manager.
    pub fn add_connection(&mut self, conn: Connection) -> Result<(), AmethystNetworkError> {
        if !self.connections.contains_key(&conn.to_string()) {
            self.connections.insert(conn.to_string(), conn);
            Ok(())
        } else {
            Err(AmethystNetworkError::AddConnectionToManagerFailed {
                reason: "Entry already exists".to_string(),
            })
        }
    }
}

/// Represents a virtual circuit to a remote endpoint
pub struct Connection {
    // IP Address of the remote endpoint
    remote_address: SocketAddr,
    // The last moment in time we heard from this client. This is used to help detect if a client has disconnected
    last_heard: Instant,
}

impl Connection {
    /// Creates a new connection based off a unique IP and Port combination
    pub fn new(addr: SocketAddr) -> Connection {
        Connection {
            remote_address: addr,
            last_heard: Instant::now(),
        }
    }

    /// Returns the duration since we last received a packet from this client
    pub fn last_heard(&self) -> Duration {
        let now = Instant::now();
        self.last_heard.duration_since(now)
    }
}

impl ToString for Connection {
    fn to_string(&self) -> String {
        format!(
            "{}:{}",
            self.remote_address.ip(),
            self.remote_address.port()
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    static TEST_HOST_IP: &'static str = "127.0.0.1";
    static TEST_BAD_HOST_IP: &'static str = "800.0.0.1";
    static TEST_PORT: &'static str = "20000";

    #[test]
    fn test_create_manager() {
        let manager = Manager::new().with_client_timeout(60);
        assert_eq!(manager.timeout, 60);
    }

    #[test]
    fn test_create_connection() {
        let mut addr = format!("{}:{}", TEST_HOST_IP, TEST_PORT).to_socket_addrs();
        assert!(addr.is_ok());
        let mut addr = addr.unwrap();
        let new_conn = Connection::new(addr.next().unwrap());
    }

    #[test]
    fn test_conn_to_string() {
        let mut addr = format!("{}:{}", TEST_HOST_IP, TEST_PORT).to_socket_addrs();
        assert!(addr.is_ok());
        let mut addr = addr.unwrap();
        let new_conn = Connection::new(addr.next().unwrap());
        assert_eq!(new_conn.to_string(), "127.0.0.1:20000");
    }

    #[test]
    fn test_invalid_addr_fails() {
        let mut addr = format!("{}:{}", TEST_BAD_HOST_IP, TEST_PORT).to_socket_addrs();
        assert!(addr.is_err());
    }
}
