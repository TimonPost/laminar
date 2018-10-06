use bincode::serialize;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

use super::{Connection, Packet, RawPacket, SocketAddr};
use error::{NetworkError, Result};

// Type aliases
// Number of seconds we will wait until we consider a Connection to have timed out
type ConnectionTimeout = u64;
type ConnectionMap = Arc<RwLock<HashMap<SocketAddr, Arc<RwLock<Connection>>>>>;

// Default timeout of a specific client
const TIMEOUT_DEFAULT: ConnectionTimeout = 10;

// Default time between checks of all clients for timeouts in seconds
const TIMEOUT_POLL_INTERVAL: u64 = 1;

/// This holds the 'virtual connections' currently (connected) to the udp socket.
pub struct SocketState {
    timeout: ConnectionTimeout,
    connections: ConnectionMap,
    timeout_check_thread: thread::JoinHandle<()>
}

impl SocketState {
    pub fn new() -> Result<SocketState> {
        let connections: ConnectionMap = Arc::new(RwLock::new(HashMap::new()));
        let thread_handle = SocketState::check_for_timeouts(connections.clone())?;
        Ok(SocketState {
            connections: Arc::new(RwLock::new(HashMap::new())),
            timeout: TIMEOUT_DEFAULT,
            timeout_check_thread: thread_handle
        })
    }

    pub fn with_client_timeout(mut self, timeout: ConnectionTimeout) -> SocketState {
        self.timeout = timeout;
        self
    }

    /// This will initialize the seq number, ack number and give back the raw data of the packet with the updated information.
    pub fn pre_process_packet(&mut self, packet: Packet) -> Result<(SocketAddr, Vec<u8>)> {
        let connection = self.create_connection_if_not_exists(&packet.addr)?;
        // queue new packet
        if let Ok(mut lock) = connection.write() {
            let seq_num = lock.seq_num;
            lock.waiting_packets.enqueue(seq_num, packet.clone());
        }

        let raw_packet: RawPacket;
        // initialize packet data, seq, acked_seq etc.
        let mut l = connection
            .write()
            .map_err(|_| NetworkError::AddConnectionToManagerFailed)?;

        raw_packet = RawPacket::new(
            l.seq_num,
            &packet,
            l.their_acks.last_seq,
            l.their_acks.field,
        );
        // increase sequence number
        l.seq_num = l.seq_num.wrapping_add(1);
        let buffer = serialize(&raw_packet)?;
        Ok((packet.addr, buffer))
    }

    /// This will return all dropped packets from this connection.
    pub fn dropped_packets(&mut self, addr: SocketAddr) -> Result<Vec<Packet>> {
        let connection = self.create_connection_if_not_exists(&addr)?;
        let mut lock = connection
            .write()
            .map_err(|_| NetworkError::AddConnectionToManagerFailed)?;

        let packets = lock.dropped_packets.drain(..).collect();
        Ok(packets)
    }

    /// This will process an incoming packet and update acknowledgement information.
    pub fn process_received(&mut self, addr: SocketAddr, packet: &RawPacket) -> Result<Packet> {
        let connection = self.create_connection_if_not_exists(&addr)?;
        let mut lock = connection
            .write()
            .map_err(|_| NetworkError::AddConnectionToManagerFailed)?;;

        lock.their_acks.ack(packet.seq);
        // Update dropped packets if there are any.
        let dropped_packets = lock.waiting_packets.ack(packet.ack_seq, packet.ack_field);
        lock.dropped_packets = dropped_packets.into_iter().map(|(_, p)| p).collect();
        Ok(Packet {
            addr,
            payload: packet.payload.clone(),
        })
    }

    // This function starts a background thread that does the following:
    // 1. Gets a read lock on the HashMap containing all the connections
    // 2. Iterate through each one
    // 3. Check if the last time we have heard from them (received a packet from them) is greater than the amount of time considered to be a timeout
    // 4. If they have timed out, send a notification up the stack
    fn check_for_timeouts(connections: ConnectionMap) -> Result<thread::JoinHandle<()>> {
        let sleepy_time = Duration::from_secs(TIMEOUT_DEFAULT);
        let poll_interval = Duration::from_secs(TIMEOUT_POLL_INTERVAL);

        Ok(thread::Builder::new()
            .name("check_for_timeouts".into())
            .spawn(move || loop {
                {
                    debug!("Checking for timeouts");
                    match connections.read() {
                        Ok(lock) => {
                            for (key, value) in lock.iter() {
                                if let Ok(c) = value.read() {
                                    if c.last_heard() >= sleepy_time {
                                        // TODO: pass up client TimedOut event
                                        error!("Client has timed out: {:?}", key);
                                    }
                                }
                            }
                        },
                        Err(e) => {
                            error!("Unable to acquire read lock to check for timed out connections")
                        }
                    }
                }
                thread::sleep(poll_interval);
            })?
        )
    }

    #[inline]
    /// If there is no connection with the given socket address an new connection will be made.
    fn create_connection_if_not_exists(
        &mut self,
        addr: &SocketAddr,
    ) -> Result<Arc<RwLock<Connection>>> {
        let mut lock = self
            .connections
            .write()
            .map_err(|_| NetworkError::AddConnectionToManagerFailed)?;

        let connection = lock
            .entry(*addr)
            .or_insert_with(|| Arc::new(RwLock::new(Connection::new(*addr))));

        Ok(connection.clone())
    }
}

#[cfg(test)]
mod test {
    use super::SocketState;
    use net::connection::Connection;
    use std::net::ToSocketAddrs;
    use std::{thread, time};
    static TEST_HOST_IP: &'static str = "127.0.0.1";
    static TEST_BAD_HOST_IP: &'static str = "800.0.0.1";
    static TEST_PORT: &'static str = "20000";

    #[test]
    fn test_create_connection() {
        let addr = format!("{}:{}", TEST_HOST_IP, TEST_PORT).to_socket_addrs();
        assert!(addr.is_ok());
        let mut addr = addr.unwrap();
        let new_conn = Connection::new(addr.next().unwrap());
    }

    #[test]
    fn test_invalid_addr_fails() {
        let addr = format!("{}:{}", TEST_BAD_HOST_IP, TEST_PORT).to_socket_addrs();
        assert!(addr.is_err());
    }

    #[test]
    fn test_poll_for_invalid_clients() {
        let mut socket_state = SocketState::new();
        thread::sleep(time::Duration::from_secs(10));
    }
}
