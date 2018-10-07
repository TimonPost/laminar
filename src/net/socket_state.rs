use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

use packet::{Packet, PacketData};
use packet::header::{FragmentHeader, PacketHeader};
use net::{Connection,SocketAddr, NetworkConfig};
use error::{NetworkError, Result};
use total_fragments_needed;

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
    pub fn pre_process_packet(&mut self, packet: Packet, config: &NetworkConfig) ->  Result<(SocketAddr, PacketData)>  {

        if packet.payload().len() > config.max_packet_size {
            error!("Packet too large: Attempting to send {}, max={}", packet.payload().len(), config.max_packet_size);
            return Err(NetworkError::ExceededMaxPacketSize.into());
        }

        let connection = self.create_connection_if_not_exists(&packet.addr())?;

        let mut connection_seq: u16 = 0;
        let mut their_last_seq: u16 = 0;
        let mut their_ack_field: u32 = 0;

        {
            let mut lock = connection
                .write()
                .map_err(|_| NetworkError::AddConnectionToManagerFailed)?;

            connection_seq = lock.seq_num;
            their_last_seq = lock.their_acks.last_seq;
            their_ack_field = lock.their_acks.field;

            // queue new packet
            lock.waiting_packets.enqueue(connection_seq, packet.clone());
        }

        let mut packet_data = PacketData::new();

        // create packet header
        let packet_header = PacketHeader::new(connection_seq, their_last_seq, their_ack_field);

        let payload = packet.payload();
        let payload_length = payload.len() as u16; /* safe cast because max packet size is u16 */

        // spit the packet if the payload lenght is greater than the allowrd fragment size.
        if payload_length <= config.fragment_size {
            packet_data.add_fragment(&packet_header, payload.to_vec());
        }else {
            let num_fragments = total_fragments_needed(payload_length, config.fragment_size) as u8; /* safe cast max fragments is u8 */

            if num_fragments > config.max_fragments {
                return Err(NetworkError::ExceededMaxFragments.into());
            }

            for fragment_id in 0..num_fragments {
                let fragment = FragmentHeader::new(fragment_id, num_fragments, packet_header.clone());

                // get start end pos in buffer
                let start_fragment_pos = fragment_id as u16 * config.fragment_size; /* upcast is safe */
                let mut end_fragment_pos = (fragment_id as u16 + 1) * config.fragment_size; /* upcast is safe */

                // If remaining buffer fits int one packet just set the end position to the length of the packet payload.
                if end_fragment_pos > payload_length {
                    end_fragment_pos = payload_length;
                }

                // get specific slice of data for fragment
                let fragment_data = &payload[start_fragment_pos as usize..end_fragment_pos as usize]; /* upcast is safe */

                packet_data.add_fragment(&fragment, fragment_data.to_vec());
            }
        }

        let mut lock = connection
            .write()
            .map_err(|_| NetworkError::AddConnectionToManagerFailed)?;

        lock.seq_num = lock.seq_num.wrapping_add(1);

        Ok((packet.addr(), packet_data))
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
    pub fn process_received(&mut self, addr: SocketAddr, packet: &PacketHeader) -> Result<()>{
        let connection = self.create_connection_if_not_exists(&addr)?;
        let mut lock = connection
            .write()
            .map_err(|_| NetworkError::AddConnectionToManagerFailed)?;

        lock.their_acks.ack(packet.seq);

        // Update dropped packets if there are any.
        let dropped_packets = lock
            .waiting_packets
            .ack(packet.ack_seq(), packet.ack_field());

        lock.dropped_packets = dropped_packets.into_iter().map(|(_, p)| p).collect();

        Ok(())
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
                
                    trace!("Checking for timeouts");
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
    use net::{Connection, NetworkConfig, SocketState, constants};
    use packet::{Packet, PacketData};
    use packet::header::{FragmentHeader, PacketHeader, HeaderReader};

    use std::io::Cursor;
    use std::net::{ToSocketAddrs, SocketAddr, IpAddr};
    use std::str::FromStr;
    use std::{thread, time};

    use total_fragments_needed;

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

    #[test]
    pub fn construct_packet_less_than_mtu()
    {
        let config = NetworkConfig::default();

        // - 1 so that packet can fit inside one fragment.
        let mut data = vec![0; config.fragment_size as usize - 1];

        // do some test processing of the data.
        let mut processed_packet: (SocketAddr, PacketData) = simulate_packet_processing(data.clone(), &config);

        // check that there is only one fragment and that the data is right.
        assert_eq!(processed_packet.1.fragment_count(), 1);
        assert_eq!(processed_packet.1.parts()[0].len(), data.len() + (constants::PACKET_HEADER_SIZE as usize));
    }

    #[test]
    pub fn construct_packet_greater_than_mtu()
    {
        let config = NetworkConfig::default();

        /// test data
        let data = vec![0; config.fragment_size as usize * 4];

        // do some test processing of the data.
        let mut processed_packet: (SocketAddr, PacketData) = simulate_packet_processing(data.clone(), &config);

        let num_fragments = total_fragments_needed(data.len() as u16, config.fragment_size);

        // check if packet is divided into fragment right
        assert_eq!(processed_packet.1.fragment_count(), num_fragments as usize);

        // check if the first packet also contains the fragment header and packet header
        assert_eq!(processed_packet.1.parts()[0].len() ,((constants::PACKET_HEADER_SIZE + constants::FRAGMENT_HEADER_SIZE) as u16 + config.fragment_size) as usize);
    }

    #[test]
    pub fn construct_packet_and_reassemble_less_than_mtu()
    {
        let config = NetworkConfig::default();

        // - 1 so that packet can fit inside one fragment.
        let data = vec![0; config.fragment_size as usize  - 1];

        // do some test processing of the data.
        let mut processed_packet = simulate_packet_processing(data.clone(), &config);

        // check if you can parse headers from the previous assembled packet
        for packet_data in processed_packet.1.parts().into_iter() {
            let mut cursor = Cursor::new(packet_data);
            assert!(PacketHeader::read(&mut cursor).is_ok())
        }
    }

    #[test]
    pub fn construct_packet_and_reassemble_greater_than_mtu()
    {
        let config = NetworkConfig::default();

        /// test data
        let data = vec![0; config.fragment_size as usize * 4];

        // do some test processing of the data.
        let mut processed_packet = simulate_packet_processing(data.clone(), &config);

        // check if you can parse headers from the previous assembled packet
        for packet_data in processed_packet.1.parts().into_iter() {
            let prefix = packet_data[0];
            let mut cursor = Cursor::new(packet_data);

            if prefix & 1 == 0 {
                assert!(FragmentHeader::read(&mut cursor).is_ok())
            }else {
                assert!(FragmentHeader::read(&mut cursor).is_ok())
            }
        }
    }

    fn simulate_packet_processing(data: Vec<u8>, config: &NetworkConfig) -> (SocketAddr, PacketData)
    {
        // create packet with test data
        let packet = Packet::new(get_dummy_socket_addr(), data.clone());

        // process the packet
        let mut socket_state = SocketState::new().unwrap();;
        let result = socket_state.pre_process_packet(packet, &config);
        result.unwrap()
    }

    fn get_dummy_socket_addr() -> SocketAddr
    {
        SocketAddr::new(
            IpAddr::from_str("127.0.0.1").expect("Unreadable input IP."),
            12348,
        )
    }
}
