use std::collections::HashMap;
use std::net::{AddrParseError, IpAddr, Ipv4Addr, Ipv6Addr};
use std::str::{FromStr};
use std::string::ToString;
use std::error::Error;
use std::time::{Duration, Instant};

use amethyst_error::AmethystNetworkError;

/// Default timeout of 10 seconds
type ConnectionTimeout = u8;
const TIMEOUT_DEFAULT: ConnectionTimeout = 10;


/// Maintains a list of all Connections and allows adding/removing them
pub struct Manager {
    /// The collection of currently connected clients
    connections: HashMap<String, Connection>,
    /// The number of seconds before we consider a client to have timed out
    timeout: Option<ConnectionTimeout>
}

impl Manager {
    pub fn new(mut self) -> Manager {
        self.connections = HashMap::new();
        self.timeout = Some(TIMEOUT_DEFAULT);
        self
    }

    pub fn with_client_timeout(mut self, timeout: ConnectionTimeout) -> Manager {
        self.timeout = Some(timeout);
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

/// Represents a virtual circuit to a remote end
pub struct Connection {
    /// IP Address of the remote endpoint
    remote_ip: IpAddr,
    /// Port the client is using
    remote_port: u16,
    /// The last moment in time we heard from this client. This is used to help detect if a client has disconnected
    last_heard: Instant,
}

impl Connection {
    pub fn new(remote_ip: &str, remote_port: u16) -> Result<Connection, AddrParseError> {
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

    const test_port: u16 = 20000;

    #[test]
    fn test_create_connection() {
        let new_conn = Connection::new("127.0.0.1", 20000);
        assert!(new_conn.is_ok());
    }

    fn test_conn_to_string() {
        let new_conn = Connection::new("127.0.0.1", 20000);
        assert!(new_conn.is_ok());
        let new_conn = new_conn.unwrap();
        assert_eq!(new_conn.to_string(), "127.0.0.1:20000");
    }
}
