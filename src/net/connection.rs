use super::{ExternalAcks, LocalAckRecord, Packet};
use std::fmt;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

/// Contains the information about a certain 'virtual connection' over udp.
/// This stores information about the last sequence number, dropped packages, packages waiting for acknowledgement and acknowledgements gotten from the other side.
pub struct Connection {
    pub seq_num: u16,
    pub dropped_packets: Vec<Packet>,
    pub waiting_packets: LocalAckRecord,
    pub their_acks: ExternalAcks,
    pub last_heard: Instant,
    pub remote_address: SocketAddr,
    pub quality: Quality,
}

impl Connection {
    /// Creates and returns a new Connection that wraps the provided socket address
    pub fn new(addr: SocketAddr) -> Connection {
        Connection {
            seq_num: 0,
            dropped_packets: Vec::new(),
            waiting_packets: Default::default(),
            their_acks: Default::default(),
            last_heard: Instant::now(),
            quality: Quality::Good,
            remote_address: addr,
        }
    }

    /// Returns a Duration representing since we last heard from the client
    pub fn last_heard(&self) -> Duration {
        let now = Instant::now();
        now.duration_since(self.last_heard)
    }
}

impl fmt::Debug for Connection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}:{}",
            self.remote_address.ip(),
            self.remote_address.port()
        )
    }
}

// TODO:
/// This defines whether the connection is good or bad.
/// We should use this for handling Congestion Avoidance so that when the network of the client is bad we do not flood the router with small packets.
///
/// When network conditions are `Good` we send 30 packets per-second, and when network conditions are `Bad` we drop to 10 packets per-second.
pub enum Quality {
    Good,
    Bad,
}

#[cfg(test)]
mod test {
    use net::connection::Connection;
    use std::net::ToSocketAddrs;

    static TEST_HOST_IP: &'static str = "127.0.0.1";
    static TEST_BAD_HOST_IP: &'static str = "800.0.0.1";
    static TEST_PORT: &'static str = "20000";

    #[test]
    fn test_create_connection() {
        let mut addr = format!("{}:{}", TEST_HOST_IP, TEST_PORT)
            .to_socket_addrs()
            .unwrap();
        let _new_conn = Connection::new(addr.next().unwrap());
    }
}
