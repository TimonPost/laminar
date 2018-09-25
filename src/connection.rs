use std::collections::HashMap;
use std::net::{AddrParseError, IpAddr, Ipv4Addr, Ipv6Addr};
use std::str::{FromStr};
use std::string::ToString;
use std::error::Error;
use std::time::{Duration, Instant};
use std::default::Default;

use amethyst_error::AmethystNetworkError;


/// Type aliases
/// Number of seconds we will wait until we consider a Connection to have timed out
type ConnectionTimeout = u8;
/// The port associated with this Connection
type NetworkPort = u16;

/// Default timeout of 10 seconds
const TIMEOUT_DEFAULT: ConnectionTimeout = 10;


/// Maintains a list of all Connections and allows adding/removing them
#[derive(Default)]
pub struct Manager {
    /// The collection of currently connected clients
    connections: HashMap<String, Connection>,
    /// The number of seconds before we consider a client to have timed out
    timeout: ConnectionTimeout
}

impl Manager {
    pub fn new() -> Manager {
        Manager {
            connections: HashMap::new(),
            timeout: TIMEOUT_DEFAULT
        }
    }

    pub fn with_client_timeout(mut self, timeout: ConnectionTimeout) -> Manager {
        self.timeout = timeout;
        self
    }

    /// Adds a new connection to the manager.
    pub fn add_connection(&mut self, conn: Connection) -> Result<(), AmethystNetworkError> {
        if !self.connections.contains_key(&conn.to_string()) {
            self.connections.insert(conn.to_string(), conn);
            Ok(())
        } else {
            Err(AmethystNetworkError::AddConnectionToManagerFailed{reason: "Entry already exists".to_string()})
        }
    }
}

/// Represents a virtual circuit to a remote endpoint
pub struct Connection {
    /// IP Address of the remote endpoint
    remote_ip: IpAddr,
    /// Port the client is using
    remote_port: NetworkPort,
    /// The last moment in time we heard from this client. This is used to help detect if a client has disconnected
    last_heard: Instant,
}

impl Connection {
    /// Creates a new connection based off a unique IP and Port combination
    /// TODO: Should we use a where clause for the remote_ip arg, to only allow things that implement Into<IpAddr>?
    pub fn new(remote_ip: &str, remote_port: NetworkPort) -> Result<Connection, AddrParseError> {
        match remote_ip.parse::<IpAddr>() {
            Ok(addr) => {
                Ok(Connection {
                    remote_ip: addr,
                    remote_port,
                    last_heard: Instant::now(),
                })
            },
            Err(e) => {
                Err(e)
            }
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
        String::from(format!("{}", self.remote_ip)) + ":" + &self.remote_port.to_string()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const TEST_PORT: u16 = 20000;

    #[test]
    fn test_create_manager() {
        let manager = Manager::new().with_client_timeout(60);
        assert_eq!(manager.timeout, 60);
    }

    #[test]
    fn test_create_connection() {
        let new_conn = Connection::new("127.0.0.1", TEST_PORT);
        assert!(new_conn.is_ok());
    }

    #[test]
    fn test_conn_to_string() {
        let new_conn = Connection::new("127.0.0.1", TEST_PORT);
        assert!(new_conn.is_ok());
        let new_conn = new_conn.unwrap();
        assert_eq!(new_conn.to_string(), "127.0.0.1:20000");
    }

    #[test]
    fn test_invalid_addr_fails() {
        let new_conn = Connection::new("800.0.0.1", TEST_PORT);
        assert!(new_conn.is_err());
    }
}
