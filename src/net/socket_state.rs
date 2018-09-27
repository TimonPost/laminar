use std::sync::{Arc, RwLock};
use std::collections::HashMap;

use bincode::serialize;

use super::{Connection, Packet, RawPacket, SocketAddr};
use amethyst_error::AmethystNetworkError;

// Type aliases
// Number of seconds we will wait until we consider a Connection to have timed out
type ConnectionTimeout = u8;
type ConnectionMap = Arc<RwLock<HashMap<SocketAddr, Arc<RwLock<Connection>>>>>;

// Default timeout of 10 seconds
const TIMEOUT_DEFAULT: ConnectionTimeout = 10;

// Default time between checks of all clients for timeouts in seconds
const TIMEOUT_POLL_INTERVAL: u8 = 1;

/// This holds the 'virtual connections' currently (connected) to the udp socket.
pub struct SocketState {
    timeout: ConnectionTimeout,
    connections: ConnectionMap
}

impl SocketState {
    pub fn new() -> SocketState {
        SocketState {
            connections: Arc::new(RwLock::new(HashMap::new())),
            timeout: TIMEOUT_DEFAULT,
        }
    }

    pub fn with_client_timeout(mut self, timeout: ConnectionTimeout) -> SocketState {
        self.timeout = timeout;
        self
    }

    /// This will initialize the seq number, ack number and give back the raw data of the packet with the updated information.
    pub fn pre_process_packet(&mut self, packet: Packet) -> Result<(SocketAddr, Vec<u8>), AmethystNetworkError> {
        let connection = self.create_connection_if_not_exists(&packet.addr)?;
        // queue new packet
        if let Ok(mut l) = connection.write() {
            l
                .waiting_packets
                .enqueue(connection.write().unwrap().seq_num, packet.clone());
        }

        let mut raw_packet: RawPacket;
        // initialize packet data, seq, acked_seq etc.
        if let Ok(mut l) = connection.write() {
            raw_packet = RawPacket::new(l.seq_num, &packet, l.their_acks.last_seq, l.their_acks.field);
            // increase sequence number
            l.seq_num = l.seq_num.wrapping_add(1);
            // TODO: remove unwrap
            let buffer = serialize(&raw_packet).unwrap();
            return Ok((packet.addr, buffer));
        }

        Err(AmethystNetworkError::Unknown)


    }

    /// This will return all dropped packets from this connection.
    pub fn dropped_packets(&mut self, addr: SocketAddr) -> Result<Vec<Packet>, AmethystNetworkError> {
        let connection = self.create_connection_if_not_exists(&addr)?;
        if let Ok(mut lock) = connection.write() {
            let packets = lock.dropped_packets.drain(..).collect();
            return Ok(packets);
        }
        Err(AmethystNetworkError::Unknown)
    }

    /// This will process an incoming packet and update acknowledgement information.
    pub fn process_received(&mut self, addr: SocketAddr, packet: &RawPacket) -> Result<Packet, AmethystNetworkError> {
        let connection = self.create_connection_if_not_exists(&addr)?;
        if let Ok(mut lock) = connection.write() {
            lock.their_acks.ack(packet.seq);
        }

        // Update dropped packets if there are any.
        if let Ok(mut lock) = connection.write() {
            let dropped_packets = lock
                .waiting_packets
                .ack(packet.ack_seq, packet.ack_field);
            lock.dropped_packets = dropped_packets.into_iter().map(|(_, p)| p).collect();
            return Ok(Packet {
                addr,
                payload: packet.payload.clone(),
            });
        }
        Err(AmethystNetworkError::Unknown)
    }

    // Regularly checks the last_heard attribute of all the connections in the manager to see if any have timed out
    fn check_for_timeouts(&mut self) {
        loop {

        }
    }

    #[inline]
    /// If there is no connection with the given socket address an new connection will be made.
    fn create_connection_if_not_exists(&mut self, addr: &SocketAddr) -> Result<Arc<RwLock<Connection>>, AmethystNetworkError> {
        if let Ok(mut lock) = self.connections.write() {
            if lock.contains_key(addr) {
                if let Some(c) = lock.get_mut(addr) {
                    return Ok(c.clone());
                }
            } else {
                let new_conn = Arc::new(RwLock::new(Connection::new(*addr)));
                lock.insert(*addr, new_conn);
            }
        }
        return Err(AmethystNetworkError::AddConnectionToManagerFailed{err: String::from("Unable to acquire lock on connection hash")});
    }
}


#[cfg(test)]
mod test {
    use super::{Manager};
    use net::connection::Connection;
    use std::net::{ToSocketAddrs};
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
        let addr = format!("{}:{}", TEST_HOST_IP, TEST_PORT).to_socket_addrs();
        assert!(addr.is_ok());
        let mut addr = addr.unwrap();
        let new_conn = Connection::new(addr.next().unwrap());
    }

    #[test]
    fn test_conn_to_string() {
        let addr = format!("{}:{}", TEST_HOST_IP, TEST_PORT).to_socket_addrs();
        assert!(addr.is_ok());
        let mut addr = addr.unwrap();
        let new_conn = Connection::new(addr.next().unwrap());
        assert_eq!(new_conn.to_string(), "127.0.0.1:20000");
    }

    #[test]
    fn test_invalid_addr_fails() {
        let addr = format!("{}:{}", TEST_BAD_HOST_IP, TEST_PORT).to_socket_addrs();
        assert!(addr.is_err());
    }
}
