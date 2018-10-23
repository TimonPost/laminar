use std::error::Error;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Instant;

use error::{FragmentErrorKind, NetworkError, NetworkResult, PacketErrorKind};
use events::Event;
use net::connection::{ConnectionPool, NetworkQualityMeasurer};
use net::{NetworkConfig, SocketAddr};
use packet::header::{FragmentHeader, PacketHeader};
use packet::{Packet, PacketData};
use sequence_buffer::CongestionData;
use total_fragments_needed;

/// This type handles the UDP-socket state.
///
/// It stores the:
/// - 'virtual connections' currently (connected) to the udp socket.
/// - Manages the network quality
/// - Processes packets before sending
/// - Processes packets when received.
pub struct SocketState {
    timeout_check_thread: thread::JoinHandle<()>,
    events: (Sender<Event>, Receiver<Event>),
    connections: ConnectionPool,
    config: NetworkConfig,
    network_quality_measurer: NetworkQualityMeasurer,
}

impl SocketState {
    /// Create a new `SocketState` by passing in an instance of `NetworkConfig`
    pub fn new(config: &NetworkConfig) -> NetworkResult<SocketState> {
        let (tx, rx) = mpsc::channel();

        let connection_pool = ConnectionPool::new();
        let join_handle = connection_pool.start_time_out_loop(tx.clone())?;

        Ok(SocketState {
            connections: connection_pool,
            timeout_check_thread: join_handle,
            events: (tx, rx),
            config: config.clone(),
            network_quality_measurer: NetworkQualityMeasurer::new(config.clone()),
        })
    }

    /// This will initialize the seq number, ack number and give back the raw data of the packet with the updated information.
    pub fn pre_process_packet(
        &mut self,
        packet: Packet,
        config: &NetworkConfig,
    ) -> NetworkResult<(SocketAddr, PacketData)> {
        if packet.payload().len() > config.max_packet_size {
            error!(
                "Packet too large: Attempting to send {}, max={}",
                packet.payload().len(),
                config.max_packet_size
            );
            Err(PacketErrorKind::ExceededMaxPacketSize)?;
        }

        let connection = self.connections.get_connection_or_insert(&packet.addr())?;

        let connection_seq: u16;
        let their_last_seq: u16;
        let their_ack_field: u32;

        {
            let mut lock = connection
                .write()
                .map_err(|error| NetworkError::poisoned_connection_error(error.description()))?;

            connection_seq = lock.seq_num;
            their_last_seq = lock.their_acks.last_seq;
            their_ack_field = lock.their_acks.field;

            lock.congestion_avoidance_buffer.insert(
                CongestionData::new(connection_seq, Instant::now()),
                connection_seq,
            );

            // queue new packet
            lock.waiting_packets.enqueue(connection_seq, packet.clone());
        }

        let mut packet_data = PacketData::new();

        let packet_header = PacketHeader::new(
            connection_seq,
            their_last_seq,
            their_ack_field,
            packet.delivery_method(),
        );

        let payload = packet.payload();
        let payload_length = payload.len() as u16; /* safe cast because max packet size is u16 */

        // spit the packet if the payload lenght is greater than the allowrd fragment size.
        if payload_length <= config.fragment_size {
            packet_data.add_fragment(&packet_header, payload.to_vec());
        } else {
            let num_fragments = total_fragments_needed(payload_length, config.fragment_size) as u8; /* safe cast max fragments is u8 */

            if num_fragments > config.max_fragments {
                Err(FragmentErrorKind::ExceededMaxFragments)?;
            }

            for fragment_id in 0..num_fragments {
                let fragment =
                    FragmentHeader::new(fragment_id, num_fragments, packet_header);

                // get start end pos in buffer
                let start_fragment_pos = u16::from(fragment_id) * config.fragment_size; /* upcast is safe */
                let mut end_fragment_pos = (u16::from(fragment_id) + 1) * config.fragment_size; /* upcast is safe */

                // If remaining buffer fits int one packet just set the end position to the length of the packet payload.
                if end_fragment_pos > payload_length {
                    end_fragment_pos = payload_length;
                }

                // get specific slice of data for fragment
                let fragment_data =
                    &payload[start_fragment_pos as usize..end_fragment_pos as usize]; /* upcast is safe */

                packet_data.add_fragment(&fragment, fragment_data.to_vec());
            }
        }

        let mut lock = connection
            .write()
            .map_err(|error| NetworkError::poisoned_connection_error(error.description()))?;

        // each time we send a packet we increase the local sequence number
        lock.seq_num = lock.seq_num.wrapping_add(1);

        Ok((packet.addr(), packet_data))
    }

    /// This will return all dropped packets from this connection.
    pub fn dropped_packets(&mut self, addr: SocketAddr) -> NetworkResult<Vec<Packet>> {
        let connection = self.connections.get_connection_or_insert(&addr)?;

        let mut lock = connection
            .write()
            .map_err(|error| NetworkError::poisoned_connection_error(error.description()))?;

        let packets = lock.dropped_packets.drain(..).collect();
        Ok(packets)
    }

    /// This will process an incoming packet and update acknowledgement information.
    pub fn process_received(
        &mut self,
        addr: SocketAddr,
        packet: &PacketHeader,
    ) -> NetworkResult<()> {
        let connection = self.connections.get_connection_or_insert(&addr)?;
        let mut lock = connection
            .write()
            .map_err(|error| NetworkError::poisoned_connection_error(error.description()))?;

        lock.their_acks.ack(packet.seq);
        lock.last_heard = Instant::now();

        self.network_quality_measurer
            .update_connection_rtt(&mut lock, packet.ack_seq());

        // Update dropped packets if there are any.
        let dropped_packets = lock
            .waiting_packets
            .ack(packet.ack_seq(), packet.ack_field());

        lock.dropped_packets = dropped_packets.into_iter().map(|(_, p)| p).collect();

        Ok(())
    }

    /// This will return a `Vec` of events for processing.
    pub fn events(&self) -> Vec<Event> {
        let (_, ref rx) = self.events;

        rx.try_iter().collect()
    }

    /// Wrapper around getting the events sender
    /// This will cause a clone to be done, but this is low cost
    pub fn get_events_sender(&self) -> Sender<Event> {
        self.events.0.clone()
    }
}

#[cfg(test)]
mod test {
    use net::{constants, NetworkConfig, SocketState};
    use packet::header::{FragmentHeader, HeaderReader, PacketHeader};
    use packet::{Packet, PacketData};

    use std::io::Cursor;
    use std::net::{IpAddr, SocketAddr};
    use std::str::FromStr;

    use total_fragments_needed;

    #[test]
    pub fn construct_packet_less_than_mtu() {
        let config = NetworkConfig::default();

        // - 1 so that packet can fit inside one fragment.
        let data = vec![0; config.fragment_size as usize - 1];

        // do some test processing of the data.
        let mut processed_packet: (SocketAddr, PacketData) =
            simulate_packet_processing(data.clone(), &config);

        // check that there is only one fragment and that the data is right.
        assert_eq!(processed_packet.1.fragment_count(), 1);
        assert_eq!(
            processed_packet.1.parts()[0].len(),
            data.len() + (constants::PACKET_HEADER_SIZE as usize)
        );
    }

    #[test]
    pub fn construct_packet_greater_than_mtu() {
        let config = NetworkConfig::default();

        // test data
        let data = vec![0; config.fragment_size as usize * 4];

        // do some test processing of the data.
        let mut processed_packet: (SocketAddr, PacketData) =
            simulate_packet_processing(data.clone(), &config);

        let num_fragments = total_fragments_needed(data.len() as u16, config.fragment_size);

        // check if packet is divided into fragment right
        assert_eq!(processed_packet.1.fragment_count(), num_fragments as usize);

        // check if the first packet also contains the fragment header and packet header
        assert_eq!(
            processed_packet.1.parts()[0].len(),
            ((constants::PACKET_HEADER_SIZE + constants::FRAGMENT_HEADER_SIZE) as u16
                + config.fragment_size) as usize
        );
    }

    #[test]
    pub fn construct_packet_and_reassemble_less_than_mtu() {
        let config = NetworkConfig::default();

        // - 1 so that packet can fit inside one fragment.
        let data = vec![0; config.fragment_size as usize - 1];

        // do some test processing of the data.
        let mut processed_packet = simulate_packet_processing(data.clone(), &config);

        // check if you can parse headers from the previous assembled packet
        for packet_data in processed_packet.1.parts().into_iter() {
            let mut cursor = Cursor::new(packet_data);
            assert!(PacketHeader::read(&mut cursor).is_ok())
        }
    }

    #[test]
    pub fn construct_packet_and_reassemble_greater_than_mtu() {
        let config = NetworkConfig::default();

        // test data
        let data = vec![0; config.fragment_size as usize * 4];

        // do some test processing of the data.
        let mut processed_packet = simulate_packet_processing(data.clone(), &config);

        // check if you can parse headers from the previous assembled packet
        for packet_data in processed_packet.1.parts().into_iter() {
            let prefix = packet_data[0];
            let mut cursor = Cursor::new(packet_data);

            if prefix & 1 == 0 {
                assert!(FragmentHeader::read(&mut cursor).is_ok())
            } else {
                assert!(FragmentHeader::read(&mut cursor).is_ok())
            }
        }
    }

    fn simulate_packet_processing(
        data: Vec<u8>,
        config: &NetworkConfig,
    ) -> (SocketAddr, PacketData) {
        // create packet with test data
        let packet = Packet::sequenced_unordered(get_dummy_socket_addr(), data.clone());

        // process the packet
        let mut socket_state = SocketState::new(&NetworkConfig::default()).unwrap();
        let result = socket_state.pre_process_packet(packet, &config);
        result.unwrap()
    }

    fn get_dummy_socket_addr() -> SocketAddr {
        SocketAddr::new(
            IpAddr::from_str("127.0.0.1").expect("Unreadable input IP."),
            12348,
        )
    }
}
