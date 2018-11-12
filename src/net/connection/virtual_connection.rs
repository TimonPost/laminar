use error::{NetworkErrorKind, NetworkResult};
use infrastructure::{
    Channel, DeliveryMethod, Fragmentation, ReliableChannel, SequencedChannel, UnreliableChannel,
};
use config::NetworkConfig;
use packet::header::HeaderReader;
use packet::header::StandardHeader;
use packet::{Packet, PacketData, PacketTypeId};
use protocol_version::ProtocolVersion;

use log::error;
use std::fmt;
use std::io::Cursor;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Contains the information about a certain 'virtual connection' over udp.
/// This connections also keeps track of network quality, processing packets, buffering data related to connection etc.
pub struct VirtualConnection {
    // client information
    /// Last time we received a packet from this client
    pub last_heard: Instant,
    /// The address of the remote endpoint
    pub remote_address: SocketAddr,

    // reliability channels for processing packets.
    unreliable_unordered_channel: UnreliableChannel,
    reliable_unordered_channel: ReliableChannel,
    sequenced_channel: SequencedChannel,

    // fragmentation
    fragmentation: Fragmentation,
}

impl VirtualConnection {
    /// Creates and returns a new Connection that wraps the provided socket address
    pub fn new(addr: SocketAddr, config: &Arc<NetworkConfig>) -> VirtualConnection {
        VirtualConnection {
            // client information
            last_heard: Instant::now(),
            remote_address: addr,

            // reliability channels for processing packets.
            unreliable_unordered_channel: UnreliableChannel::new(true),
            reliable_unordered_channel: ReliableChannel::new(false, &config),
            sequenced_channel: SequencedChannel::new(),

            fragmentation: Fragmentation::new(&config),
        }
    }

    /// Returns a Duration representing the interval since we last heard from the client
    pub fn last_heard(&self) -> Duration {
        let now = Instant::now();
        now.duration_since(self.last_heard)
    }

    /// This pre-process the given buffer to be send over the network.
    /// 1. It will append the right header.
    /// 2. It will perform some actions related to how the packet should be delivered.
    pub fn process_outgoing(
        &mut self,
        payload: &[u8],
        delivery_method: DeliveryMethod,
    ) -> NetworkResult<PacketData> {
        let channel: &mut Channel = match delivery_method {
            DeliveryMethod::UnreliableUnordered => &mut self.unreliable_unordered_channel,
            DeliveryMethod::ReliableUnordered => &mut self.reliable_unordered_channel,
            DeliveryMethod::Sequenced => &mut self.sequenced_channel,
            _ => {
                error!("Tried using channel type which is not supported yet. Swished to unreliable unordered packet handling.");
                &mut self.unreliable_unordered_channel
            }
        };

        let packet_data: PacketData = channel.process_outgoing(payload, delivery_method)?;

        Ok(packet_data)
    }

    /// This process the incoming data and returns an packet if the data is complete.
    ///
    /// Returns `Ok(None)`:
    /// 1. In the case of fragmentation and not all fragments are received
    /// 2. In the case of the packet being queued for ordering and we are waiting on older packets first.
    pub fn process_incoming(&mut self, received_data: &[u8]) -> NetworkResult<Option<Packet>> {
        self.last_heard = Instant::now();

        let mut cursor = Cursor::new(received_data);
        let header = StandardHeader::read(&mut cursor)?;

        if !ProtocolVersion::valid_version(header.protocol_version) {
            return Err(NetworkErrorKind::ProtocolVersionMismatch.into());
        }

        if header.packet_type_id == PacketTypeId::Fragment {
            cursor.set_position(0);
            match self.fragmentation.handle_fragment(&mut cursor) {
                Ok(Some(payload)) => {
                    return Ok(Some(Packet::new(
                        self.remote_address,
                        payload.into_boxed_slice(),
                        header.delivery_method,
                    )))
                }
                Ok(None) => return Ok(None),
                Err(e) => return Err(e),
            }
        }

        // get the right channel to process the packet.
        let channel: &mut Channel = match header.delivery_method {
            DeliveryMethod::UnreliableUnordered => &mut self.unreliable_unordered_channel,
            DeliveryMethod::ReliableUnordered => &mut self.reliable_unordered_channel,
            DeliveryMethod::Sequenced => &mut self.sequenced_channel,
            _ => {
                error!("Tried using channel type which is not supported yet. Swished to unreliable unordered packet handling.");
                &mut self.unreliable_unordered_channel
            }
        };

        let payload = channel.process_incoming(received_data)?;

        Ok(Some(Packet::new(
            self.remote_address,
            Box::from(payload),
            header.delivery_method,
        )))
    }

    /// This will gather dropped packets from the reliable channels.
    ///
    /// Note that after requesting dropped packets the dropped packets will be removed from this client.
    pub fn gather_dropped_packets(&mut self) -> Vec<Box<[u8]>> {
        if self.reliable_unordered_channel.has_dropped_packets() {
            return self.reliable_unordered_channel.drain_dropped_packets();
        }

        Vec::new()
    }
}

impl fmt::Debug for VirtualConnection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}:{}",
            self.remote_address.ip(),
            self.remote_address.port()
        )
    }
}

#[cfg(test)]
mod tests {
    use infrastructure::DeliveryMethod;
    use net::connection::VirtualConnection;
    use config::NetworkConfig;
    use std::sync::Arc;

    const SERVER_ADDR: &str = "127.0.0.1:12345";

    fn create_virtual_connection() -> VirtualConnection {
        VirtualConnection::new(
            SERVER_ADDR.parse().unwrap(),
            &Arc::new(NetworkConfig::default()),
        )
    }

    fn assert_packet_payload(
        buffer: &[u8],
        parts: &Vec<Vec<u8>>,
        connection: &mut VirtualConnection,
    ) {
        for part in parts {
            let packet = connection.process_incoming(&part).unwrap().unwrap();
            assert_eq!(buffer, packet.payload());
        }
    }

    #[test]
    fn process_unreliable_packet() {
        let mut connection = create_virtual_connection();

        let buffer = vec![1; 500];

        let mut packet_data = connection
            .process_outgoing(&buffer, DeliveryMethod::UnreliableUnordered)
            .unwrap();
        assert_eq!(packet_data.fragment_count(), 1);

        assert_packet_payload(&buffer, packet_data.parts(), &mut connection);
    }

    #[test]
    fn process_reliable_unordered_packet() {
        let mut connection = create_virtual_connection();

        let buffer = vec![1; 500];

        let mut packet_data = connection
            .process_outgoing(&buffer, DeliveryMethod::ReliableUnordered)
            .unwrap();
        assert_eq!(packet_data.fragment_count(), 1);

        assert_packet_payload(&buffer, packet_data.parts(), &mut connection);
    }

    #[test]
    fn process_fragmented_packet() {
        let mut connection = create_virtual_connection();

        let buffer = vec![1; 4000];

        let mut packet_data = connection
            .process_outgoing(&buffer, DeliveryMethod::ReliableUnordered)
            .unwrap();

        // there should be 4 fragments
        assert_eq!(packet_data.fragment_count(), 4);

        for (index, part) in packet_data.parts().into_iter().enumerate() {
            let option = connection.process_incoming(&part).unwrap();

            // take note index 3 will contain the fragment data because the bytes of the fragmented packet will be returned when all fragments are processed.
            // that is why the last packet (index 3) can be asserted on.
            match option {
                None => if index < 3 {
                    assert!(true)
                } else {
                    assert!(false)
                },
                Some(packet) => if index == 3 {
                    assert_eq!(buffer, packet.payload());
                },
            }
        }
    }
}
