use crate::{
    config::Config,
    either::Either,
    error::{ErrorKind, PacketErrorKind, Result},
    infrastructure::{
        arranging::{Arranging, ArrangingSystem, OrderingSystem, SequencingSystem},
        AcknowledgmentHandler, CongestionHandler, Fragmentation, SentPacket,
    },
    net::constants::{
        ACKED_PACKET_HEADER, DEFAULT_ORDERING_STREAM, DEFAULT_SEQUENCING_STREAM,
        STANDARD_HEADER_SIZE,
    },
    packet::{
        DeliveryGuarantee, GenericPacket, OrderingGuarantee, OutgoingPacket, OutgoingPacketBuilder,
        Packet, PacketReader, PacketType, SequenceNumber,
    },
};

use std::fmt;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use std::collections::VecDeque;
/// Helper class that implement Iterator, and is used to return incomming (from bytes to packets) or outgoing (from packet to bytes) packets.
/// It is used as optimization in cases, where most of the time there is only one element to iterate, and we don't want to create vector for it
pub struct ZeroOrMore<T> {
    data: Either<Option<T>, VecDeque<T>>,
}

impl<T> ZeroOrMore<T> {
    fn zero() -> Self {
        Self {
            data: Either::Left(None),
        }
    }

    fn one(data: T) -> Self {
        Self {
            data: Either::Left(Some(data)),
        }
    }

    fn many(vec: VecDeque<T>) -> Self {
        Self {
            data: Either::Right(vec),
        }
    }
}

impl<T> Iterator for ZeroOrMore<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.data {
            Either::Left(option) => option.take(),
            Either::Right(vec) => vec.pop_front(),
        }
    }
}

/// Stores packets with headers that will be sent to network
pub struct OutgoingPackets<'a> {
    data: ZeroOrMore<OutgoingPacket<'a>>,
}

impl<'a> OutgoingPackets<'a> {
    fn one(packet: OutgoingPacket<'a>) -> Self {
        Self {
            data: ZeroOrMore::one(packet),
        }
    }
    fn many(packets: VecDeque<OutgoingPacket<'a>>) -> Self {
        Self {
            data: ZeroOrMore::many(packets),
        }
    }
}

impl<'a> IntoIterator for OutgoingPackets<'a> {
    type Item = OutgoingPacket<'a>;
    type IntoIter = ZeroOrMore<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.data
    }
}

/// Stores parsed packets with their types, that was received from network
pub struct IncomingPackets {
    data: ZeroOrMore<(Packet, PacketType)>,
}

impl IncomingPackets {
    fn zero() -> Self {
        Self {
            data: ZeroOrMore::zero(),
        }
    }

    fn one(packet: Packet, packet_type: PacketType) -> Self {
        Self {
            data: ZeroOrMore::one((packet, packet_type)),
        }
    }

    fn many(vec: VecDeque<(Packet, PacketType)>) -> Self {
        Self {
            data: ZeroOrMore::many(vec),
        }
    }
}

impl IntoIterator for IncomingPackets {
    type Item = (Packet, PacketType);
    type IntoIter = ZeroOrMore<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.data
    }
}

/// Contains the information about a certain 'virtual connection' over udp.
/// This connections also keeps track of network quality, processing packets, buffering data related to connection etc.
pub struct VirtualConnection {
    /// Last time we received a packet from this client
    pub last_heard: Instant,
    /// Last time we sent a packet to this client
    pub last_sent: Instant,
    /// The address of the remote endpoint
    pub remote_address: SocketAddr,

    ordering_system: OrderingSystem<(Box<[u8]>, PacketType)>,
    sequencing_system: SequencingSystem<Box<[u8]>>,
    acknowledge_handler: AcknowledgmentHandler,
    congestion_handler: CongestionHandler,

    config: Config,
    fragmentation: Fragmentation,
}

impl VirtualConnection {
    /// Creates and returns a new Connection that wraps the provided socket address
    pub fn new(addr: SocketAddr, config: &Config, time: Instant) -> VirtualConnection {
        VirtualConnection {
            last_heard: time,
            last_sent: time,
            remote_address: addr,
            ordering_system: OrderingSystem::new(),
            sequencing_system: SequencingSystem::new(),
            acknowledge_handler: AcknowledgmentHandler::new(),
            congestion_handler: CongestionHandler::new(config),
            fragmentation: Fragmentation::new(config),
            config: config.to_owned(),
        }
    }

    /// Determine if this connection should be dropped due to its state
    pub fn should_be_dropped(&self) -> bool {
        self.acknowledge_handler.packets_in_flight() > self.config.max_packets_in_flight
    }

    /// Returns a [Duration] representing the interval since we last heard from the client
    pub fn last_heard(&self, time: Instant) -> Duration {
        // TODO: Replace with saturating_duration_since once it becomes stable.
        // This function panics if the user supplies a time instant earlier than last_heard.
        time.duration_since(self.last_heard)
    }

    /// Returns a [Duration] representing the interval since we last sent to the client
    pub fn last_sent(&self, time: Instant) -> Duration {
        // TODO: Replace with saturating_duration_since once it becomes stable.
        // This function panics if the user supplies a time instant earlier than last_heard.
        time.duration_since(self.last_sent)
    }

    /// This will pre-process the given buffer to be sent over the network.
    pub fn process_outgoing<'a>(
        &mut self,
        packet: GenericPacket<'a>,
        last_item_identifier: Option<SequenceNumber>,
        time: Instant,
    ) -> Result<OutgoingPackets<'a>> {
        self.last_sent = time;
        match packet.delivery {
            DeliveryGuarantee::Unreliable => {
                if packet.payload.len() <= self.config.receive_buffer_max_size {
                    if packet.packet_type == PacketType::Heartbeat {
                        // TODO (bug?) is this really required here?
                        self.congestion_handler
                            .process_outgoing(self.acknowledge_handler.local_sequence_num(), time);
                    }

                    let mut builder = OutgoingPacketBuilder::new(packet.payload)
                        .with_default_header(packet.packet_type, packet.delivery, packet.ordering);

                    if let OrderingGuarantee::Sequenced(stream_id) = packet.ordering {
                        let item_identifier = self
                            .sequencing_system
                            .get_or_create_stream(stream_id.unwrap_or(DEFAULT_SEQUENCING_STREAM))
                            .new_item_identifier();

                        builder = builder.with_sequencing_header(item_identifier as u16, stream_id);
                    };

                    Ok(OutgoingPackets::one(builder.build()))
                } else {
                    Err(PacketErrorKind::ExceededMaxPacketSize.into())
                }
            }
            DeliveryGuarantee::Reliable => {
                let payload_length = packet.payload.len() as u16;

                let mut item_identifier_value = None;
                let outgoing = {
                    // spit the packet if the payload length is greater than the allowed fragment size.
                    if payload_length <= self.config.fragment_size {
                        let mut builder = OutgoingPacketBuilder::new(packet.payload)
                            .with_default_header(
                                packet.packet_type,
                                packet.delivery,
                                packet.ordering,
                            );

                        builder = builder.with_acknowledgment_header(
                            self.acknowledge_handler.local_sequence_num(),
                            self.acknowledge_handler.remote_sequence_num(),
                            self.acknowledge_handler.ack_bitfield(),
                        );

                        if let OrderingGuarantee::Ordered(stream_id) = packet.ordering {
                            let item_identifier =
                                if let Some(item_identifier) = last_item_identifier {
                                    item_identifier
                                } else {
                                    self.ordering_system
                                        .get_or_create_stream(
                                            stream_id.unwrap_or(DEFAULT_ORDERING_STREAM),
                                        )
                                        .new_item_identifier()
                                };

                            item_identifier_value = Some(item_identifier);

                            builder = builder.with_ordering_header(item_identifier, stream_id);
                        };

                        if let OrderingGuarantee::Sequenced(stream_id) = packet.ordering {
                            let item_identifier =
                                if let Some(item_identifier) = last_item_identifier {
                                    item_identifier
                                } else {
                                    self.sequencing_system
                                        .get_or_create_stream(
                                            stream_id.unwrap_or(DEFAULT_SEQUENCING_STREAM),
                                        )
                                        .new_item_identifier()
                                };

                            item_identifier_value = Some(item_identifier);

                            builder = builder.with_sequencing_header(item_identifier, stream_id);
                        };

                        OutgoingPackets::one(builder.build())
                    } else {
                        if packet.packet_type != PacketType::Packet {
                            return Err(PacketErrorKind::PacketCannotBeFragmented.into());
                        }
                        OutgoingPackets::many(
                            Fragmentation::spit_into_fragments(packet.payload, &self.config)?
                                .into_iter()
                                .enumerate()
                                .map(|(fragment_id, fragment)| {
                                    let fragments_needed = Fragmentation::fragments_needed(
                                        payload_length,
                                        self.config.fragment_size,
                                    )
                                        as u8;

                                    let mut builder = OutgoingPacketBuilder::new(fragment)
                                        .with_default_header(
                                            PacketType::Fragment, // change from Packet to Fragment type, it only matters when assembling/dissasembling packet header.
                                            packet.delivery,
                                            packet.ordering,
                                        );

                                    builder = builder.with_fragment_header(
                                        self.acknowledge_handler.local_sequence_num(),
                                        fragment_id as u8,
                                        fragments_needed,
                                    );

                                    if fragment_id == 0 {
                                        builder = builder.with_acknowledgment_header(
                                            self.acknowledge_handler.local_sequence_num(),
                                            self.acknowledge_handler.remote_sequence_num(),
                                            self.acknowledge_handler.ack_bitfield(),
                                        );
                                    }

                                    builder.build()
                                })
                                .collect(),
                        )
                    }
                };

                self.congestion_handler
                    .process_outgoing(self.acknowledge_handler.local_sequence_num(), time);
                self.acknowledge_handler.process_outgoing(
                    packet.packet_type,
                    packet.payload,
                    packet.ordering,
                    item_identifier_value,
                );

                Ok(outgoing)
            }
        }
    }

    /// This processes the incoming data and returns a packet if the data is complete.
    pub fn process_incoming(
        &mut self,
        received_data: &[u8],
        time: Instant,
    ) -> Result<IncomingPackets> {
        self.last_heard = time;

        let mut packet_reader = PacketReader::new(received_data);

        let header = packet_reader.read_standard_header()?;

        if !header.is_current_protocol() {
            return Err(ErrorKind::ProtocolVersionMismatch);
        }

        if header.is_heartbeat() {
            // Heartbeat packets are unreliable, unordered and empty packets.
            // We already updated our `self.last_heard` time, nothing else to be done.
            return Ok(IncomingPackets::zero());
        }

        match header.delivery_guarantee() {
            DeliveryGuarantee::Unreliable => {
                if let OrderingGuarantee::Sequenced(_id) = header.ordering_guarantee() {
                    let arranging_header =
                        packet_reader.read_arranging_header(u16::from(STANDARD_HEADER_SIZE))?;

                    let payload = packet_reader.read_payload();

                    let stream = self
                        .sequencing_system
                        .get_or_create_stream(arranging_header.stream_id());

                    if let Some(packet) = stream.arrange(arranging_header.arranging_id(), payload) {
                        return Ok(IncomingPackets::one(
                            Packet::new(
                                self.remote_address,
                                packet,
                                header.delivery_guarantee(),
                                OrderingGuarantee::Sequenced(Some(arranging_header.stream_id())),
                            ),
                            header.packet_type(),
                        ));
                    }

                    return Ok(IncomingPackets::zero());
                }

                return Ok(IncomingPackets::one(
                    Packet::new(
                        self.remote_address,
                        packet_reader.read_payload(),
                        header.delivery_guarantee(),
                        header.ordering_guarantee(),
                    ),
                    header.packet_type(),
                ));
            }
            DeliveryGuarantee::Reliable => {
                if header.is_fragment() {
                    if let Ok((fragment_header, acked_header)) = packet_reader.read_fragment() {
                        let payload = packet_reader.read_payload();

                        match self.fragmentation.handle_fragment(
                            fragment_header,
                            &payload,
                            acked_header,
                        ) {
                            Ok(Some((payload, acked_header))) => {
                                self.congestion_handler
                                    .process_incoming(acked_header.sequence());
                                self.acknowledge_handler.process_incoming(
                                    acked_header.sequence(),
                                    acked_header.ack_seq(),
                                    acked_header.ack_field(),
                                );

                                return Ok(IncomingPackets::one(
                                    Packet::new(
                                        self.remote_address,
                                        payload.into_boxed_slice(),
                                        header.delivery_guarantee(),
                                        header.ordering_guarantee(),
                                    ),
                                    PacketType::Packet, // change from Fragment to Packet type, it only matters when assembling/dissasembling packet header.
                                ));
                            }
                            Ok(None) => return Ok(IncomingPackets::zero()),
                            Err(e) => return Err(e),
                        };
                    }
                } else {
                    let acked_header = packet_reader.read_acknowledge_header()?;

                    self.congestion_handler
                        .process_incoming(acked_header.sequence());
                    self.acknowledge_handler.process_incoming(
                        acked_header.sequence(),
                        acked_header.ack_seq(),
                        acked_header.ack_field(),
                    );

                    if let OrderingGuarantee::Sequenced(_) = header.ordering_guarantee() {
                        let arranging_header = packet_reader.read_arranging_header(u16::from(
                            STANDARD_HEADER_SIZE + ACKED_PACKET_HEADER,
                        ))?;

                        let payload = packet_reader.read_payload();

                        let stream = self
                            .sequencing_system
                            .get_or_create_stream(arranging_header.stream_id());

                        if let Some(packet) =
                            stream.arrange(arranging_header.arranging_id(), payload)
                        {
                            return Ok(IncomingPackets::one(
                                Packet::new(
                                    self.remote_address,
                                    packet,
                                    header.delivery_guarantee(),
                                    OrderingGuarantee::Sequenced(Some(
                                        arranging_header.stream_id(),
                                    )),
                                ),
                                header.packet_type(),
                            ));
                        }
                    } else if let OrderingGuarantee::Ordered(_id) = header.ordering_guarantee() {
                        let arranging_header = packet_reader.read_arranging_header(u16::from(
                            STANDARD_HEADER_SIZE + ACKED_PACKET_HEADER,
                        ))?;

                        let payload = packet_reader.read_payload();

                        let stream = self
                            .ordering_system
                            .get_or_create_stream(arranging_header.stream_id());
                        let address = self.remote_address;
                        return Ok(IncomingPackets::many(
                            stream
                                .arrange(
                                    arranging_header.arranging_id(),
                                    (payload, header.packet_type()),
                                )
                                .into_iter()
                                .chain(stream.iter_mut())
                                .map(|(packet, packet_type)| {
                                    (
                                        Packet::new(
                                            address,
                                            packet,
                                            header.delivery_guarantee(),
                                            OrderingGuarantee::Ordered(Some(
                                                arranging_header.stream_id(),
                                            )),
                                        ),
                                        packet_type,
                                    )
                                })
                                .collect(),
                        ));
                    } else {
                        let payload = packet_reader.read_payload();
                        return Ok(IncomingPackets::one(
                            Packet::new(
                                self.remote_address,
                                payload,
                                header.delivery_guarantee(),
                                header.ordering_guarantee(),
                            ),
                            header.packet_type(),
                        ));
                    }
                }
            }
        }
        Ok(IncomingPackets::zero())
    }

    /// This will gather dropped packets from the acknowledgment handler.
    ///
    /// Note that after requesting dropped packets the dropped packets will be removed from this client.
    pub fn gather_dropped_packets(&mut self) -> Vec<SentPacket> {
        self.acknowledge_handler.dropped_packets()
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
    use super::VirtualConnection;
    use crate::config::Config;
    use crate::net::constants;
    use crate::packet::header::{AckedPacketHeader, ArrangingHeader, HeaderWriter, StandardHeader};
    use crate::packet::{DeliveryGuarantee, GenericPacket, OrderingGuarantee, Packet, PacketType};
    use crate::protocol_version::ProtocolVersion;
    use byteorder::{BigEndian, WriteBytesExt};
    use std::io::Write;
    use std::time::{Duration, Instant};

    const PAYLOAD: [u8; 4] = [1, 2, 3, 4];

    #[test]
    fn set_last_sent_and_last_heard_when_processing() {
        let mut connection = create_virtual_connection();
        let curr_sent = connection.last_sent;
        let curr_heard = connection.last_heard;

        let out_packet = connection
            .process_outgoing(
                GenericPacket::heartbeat_packet(&[]),
                None,
                curr_sent + Duration::from_secs(1),
            )
            .unwrap()
            .into_iter()
            .next()
            .unwrap();
        let in_packet = connection
            .process_incoming(&out_packet.contents(), curr_heard + Duration::from_secs(2))
            .unwrap()
            .into_iter()
            .next();

        assert_eq!(
            connection.last_sent.duration_since(curr_sent),
            Duration::from_secs(1)
        );
        assert_eq!(
            connection.last_heard.duration_since(curr_heard),
            Duration::from_secs(2)
        );
        assert_eq!(in_packet.is_none(), true);
    }

    #[test]
    fn assure_right_fragmentation() {
        let mut protocol_version = Vec::new();
        protocol_version
            .write_u16::<BigEndian>(ProtocolVersion::get_crc16())
            .unwrap();

        let standard_header = [protocol_version, vec![1, 1, 2]].concat();

        let acked_header = vec![0, 0, 0, 4, 0, 0, 255, 255, 0, 0, 0, 0];
        let first_fragment = vec![0, 0, 1, 4];
        let second_fragment = vec![0, 0, 2, 4];
        let third_fragment = vec![0, 0, 3, 4];

        let mut connection = create_virtual_connection();
        let packet = connection
            .process_incoming(
                [standard_header.as_slice(), acked_header.as_slice()]
                    .concat()
                    .as_slice(),
                Instant::now(),
            )
            .unwrap()
            .into_iter()
            .next();
        assert!(packet.is_none());
        let packet = connection
            .process_incoming(
                [
                    standard_header.as_slice(),
                    first_fragment.as_slice(),
                    &PAYLOAD,
                ]
                .concat()
                .as_slice(),
                Instant::now(),
            )
            .unwrap()
            .into_iter()
            .next();
        assert!(packet.is_none());
        let packet = connection
            .process_incoming(
                [
                    standard_header.as_slice(),
                    second_fragment.as_slice(),
                    &PAYLOAD,
                ]
                .concat()
                .as_slice(),
                Instant::now(),
            )
            .unwrap()
            .into_iter()
            .next();
        assert!(packet.is_none());
        let (packets, _) = connection
            .process_incoming(
                [
                    standard_header.as_slice(),
                    third_fragment.as_slice(),
                    &PAYLOAD,
                ]
                .concat()
                .as_slice(),
                Instant::now(),
            )
            .unwrap()
            .into_iter()
            .next()
            .unwrap();
        assert_eq!(
            packets.payload(),
            &*[PAYLOAD, PAYLOAD, PAYLOAD].concat().into_boxed_slice()
        );
    }

    #[test]
    fn expect_fragmentation() {
        let mut connection = create_virtual_connection();

        let buffer = vec![1; 4000];

        let packets: Vec<_> = connection
            .process_outgoing(
                GenericPacket::user_packet(
                    &buffer,
                    DeliveryGuarantee::Reliable,
                    OrderingGuarantee::Ordered(None),
                ),
                None,
                Instant::now(),
            )
            .unwrap()
            .into_iter()
            .collect();
        assert_eq!(packets.len(), 4);
    }

    #[test]
    fn assure_correct_outgoing_processing() {
        let mut connection = create_virtual_connection();

        let buffer = vec![1; 1000];

        connection
            .process_outgoing(
                GenericPacket::user_packet(
                    &buffer,
                    DeliveryGuarantee::Unreliable,
                    OrderingGuarantee::None,
                ),
                None,
                Instant::now(),
            )
            .unwrap();

        connection
            .process_outgoing(
                GenericPacket::user_packet(
                    &buffer,
                    DeliveryGuarantee::Unreliable,
                    OrderingGuarantee::Sequenced(None),
                ),
                None,
                Instant::now(),
            )
            .unwrap();

        connection
            .process_outgoing(
                GenericPacket::user_packet(
                    &buffer,
                    DeliveryGuarantee::Reliable,
                    OrderingGuarantee::Ordered(None),
                ),
                None,
                Instant::now(),
            )
            .unwrap();

        connection
            .process_outgoing(
                GenericPacket::user_packet(
                    &buffer,
                    DeliveryGuarantee::Reliable,
                    OrderingGuarantee::Sequenced(None),
                ),
                None,
                Instant::now(),
            )
            .unwrap();
    }

    #[test]
    fn assure_right_sequencing() {
        let mut connection = create_virtual_connection();

        assert_incoming_with_order(
            DeliveryGuarantee::Unreliable,
            OrderingGuarantee::Sequenced(Some(1)),
            &mut connection,
            Some(Packet::unreliable_sequenced(
                get_fake_addr(),
                PAYLOAD.to_vec(),
                Some(1),
            )),
            1,
        );

        assert_incoming_with_order(
            DeliveryGuarantee::Unreliable,
            OrderingGuarantee::Sequenced(Some(1)),
            &mut connection,
            Some(Packet::unreliable_sequenced(
                get_fake_addr(),
                PAYLOAD.to_vec(),
                Some(1),
            )),
            3,
        );

        assert_incoming_with_order(
            DeliveryGuarantee::Unreliable,
            OrderingGuarantee::Sequenced(Some(1)),
            &mut connection,
            None,
            2,
        );

        assert_incoming_with_order(
            DeliveryGuarantee::Unreliable,
            OrderingGuarantee::Sequenced(Some(1)),
            &mut connection,
            Some(Packet::unreliable_sequenced(
                get_fake_addr(),
                PAYLOAD.to_vec(),
                Some(1),
            )),
            4,
        );

        assert_incoming_with_order(
            DeliveryGuarantee::Reliable,
            OrderingGuarantee::Sequenced(Some(1)),
            &mut connection,
            Some(Packet::reliable_sequenced(
                get_fake_addr(),
                PAYLOAD.to_vec(),
                Some(1),
            )),
            5,
        );
    }

    #[test]
    fn assure_right_ordering() {
        let mut connection = create_virtual_connection();

        assert_incoming_with_order(
            DeliveryGuarantee::Reliable,
            OrderingGuarantee::Ordered(Some(1)),
            &mut connection,
            Some(Packet::reliable_ordered(
                get_fake_addr(),
                PAYLOAD.to_vec(),
                Some(1),
            )),
            0,
        );

        assert_incoming_with_order(
            DeliveryGuarantee::Reliable,
            OrderingGuarantee::Ordered(Some(1)),
            &mut connection,
            None,
            2,
        );

        assert_incoming_with_order(
            DeliveryGuarantee::Reliable,
            OrderingGuarantee::Ordered(Some(1)),
            &mut connection,
            None,
            3,
        );

        assert_incoming_with_order(
            DeliveryGuarantee::Reliable,
            OrderingGuarantee::Ordered(Some(1)),
            &mut connection,
            Some(Packet::reliable_ordered(
                get_fake_addr(),
                PAYLOAD.to_vec(),
                Some(1),
            )),
            1,
        );
    }

    #[test]
    fn assure_correct_processing_of_incoming() {
        let mut connection = create_virtual_connection();

        assert_incoming_without_order(
            DeliveryGuarantee::Unreliable,
            &mut connection,
            Packet::unreliable(get_fake_addr(), PAYLOAD.to_vec()),
        );

        assert_incoming_without_order(
            DeliveryGuarantee::Reliable,
            &mut connection,
            Packet::reliable_unordered(get_fake_addr(), PAYLOAD.to_vec()),
        );

        assert_incoming_with_order(
            DeliveryGuarantee::Unreliable,
            OrderingGuarantee::Sequenced(Some(1)),
            &mut connection,
            Some(Packet::unreliable_sequenced(
                get_fake_addr(),
                PAYLOAD.to_vec(),
                Some(1),
            )),
            1,
        );

        assert_incoming_with_order(
            DeliveryGuarantee::Reliable,
            OrderingGuarantee::Ordered(Some(1)),
            &mut connection,
            Some(Packet::reliable_ordered(
                get_fake_addr(),
                PAYLOAD.to_vec(),
                Some(1),
            )),
            0,
        );
    }

    #[test]
    fn assure_right_header_size() {
        assert_right_header_size(
            DeliveryGuarantee::Unreliable,
            OrderingGuarantee::None,
            (constants::STANDARD_HEADER_SIZE) as usize,
        );
        assert_right_header_size(
            DeliveryGuarantee::Unreliable,
            OrderingGuarantee::Sequenced(None),
            (constants::STANDARD_HEADER_SIZE + constants::ARRANGING_PACKET_HEADER) as usize,
        );
        assert_right_header_size(
            DeliveryGuarantee::Reliable,
            OrderingGuarantee::None,
            (constants::STANDARD_HEADER_SIZE + constants::ACKED_PACKET_HEADER) as usize,
        );
        assert_right_header_size(
            DeliveryGuarantee::Reliable,
            OrderingGuarantee::Ordered(None),
            (constants::STANDARD_HEADER_SIZE
                + constants::ACKED_PACKET_HEADER
                + constants::ARRANGING_PACKET_HEADER) as usize,
        );
    }

    #[test]
    fn ensure_input_header_data_does_not_access_out_of_bounds() {
        let mut protocol_version = Vec::new();
        protocol_version
            .write_u16::<BigEndian>(ProtocolVersion::get_crc16())
            .unwrap();

        let standard_header = [protocol_version, vec![1, 1, 2]].concat();

        let acked_header = vec![0, 0, 255, 4, 0, 0, 255, 255, 0, 0, 0, 0];

        use crate::error::{ErrorKind, FragmentErrorKind};

        let mut connection = create_virtual_connection();
        let result = connection.process_incoming(
            [standard_header.as_slice(), acked_header.as_slice()]
                .concat()
                .as_slice(),
            Instant::now(),
        );

        match result {
            Err(ErrorKind::FragmentError(FragmentErrorKind::ExceededMaxFragments)) => {
                // Ok
            }
            _ => {
                panic!["Supposed to get a fragment error"];
            }
        }
    }

    /// ======= helper functions =========
    fn create_virtual_connection() -> VirtualConnection {
        VirtualConnection::new(get_fake_addr(), &Config::default(), Instant::now())
    }

    fn get_fake_addr() -> std::net::SocketAddr {
        "127.0.0.1:0".parse().unwrap()
    }

    // assert that the processing of the given `DeliveryGuarantee` and `OrderingGuarantee` results into the given `result_event`
    fn assert_incoming_with_order(
        delivery: DeliveryGuarantee,
        ordering: OrderingGuarantee,
        connection: &mut VirtualConnection,
        result_packet: Option<Packet>,
        order_id: u16,
    ) {
        let mut packet = Vec::new();

        // configure the right header based on specified guarantees.
        let header = StandardHeader::new(delivery, ordering, PacketType::Packet);
        header.parse(&mut packet).unwrap();

        if let OrderingGuarantee::Sequenced(val) = ordering {
            if delivery == DeliveryGuarantee::Reliable {
                let ack_header = AckedPacketHeader::new(1, 2, 3);
                ack_header.parse(&mut packet).unwrap();
            }

            let order_header = ArrangingHeader::new(order_id, val.unwrap());
            order_header.parse(&mut packet).unwrap();
        }

        if let OrderingGuarantee::Ordered(val) = ordering {
            if delivery == DeliveryGuarantee::Reliable {
                let ack_header = AckedPacketHeader::new(1, 2, 3);
                let order_header = ArrangingHeader::new(order_id, val.unwrap());
                ack_header.parse(&mut packet).unwrap();
                order_header.parse(&mut packet).unwrap();
            }
        }

        if let OrderingGuarantee::None = ordering {
            if delivery == DeliveryGuarantee::Reliable {
                let ack_header = AckedPacketHeader::new(1, 2, 3);
                ack_header.parse(&mut packet).unwrap();
            }
        }

        packet.write_all(&PAYLOAD).unwrap();

        let packets = connection
            .process_incoming(packet.as_slice(), Instant::now())
            .unwrap()
            .into_iter()
            .next()
            .map(|(packet, _)| packet);
        assert_eq!(packets, result_packet);
    }

    // assert that the given `DeliveryGuarantee` results into the given `Packet` after processing.
    fn assert_incoming_without_order(
        delivery: DeliveryGuarantee,
        connection: &mut VirtualConnection,
        result_packet: Packet,
    ) {
        let mut packet = Vec::new();

        // configure the right header based on specified guarantees.
        let header = StandardHeader::new(delivery, OrderingGuarantee::None, PacketType::Packet);
        header.parse(&mut packet).unwrap();

        if delivery == DeliveryGuarantee::Reliable {
            let ack_header = AckedPacketHeader::new(1, 2, 3);
            ack_header.parse(&mut packet).unwrap();
        }

        packet.write_all(&PAYLOAD).unwrap();

        let (packet, _) = connection
            .process_incoming(packet.as_slice(), Instant::now())
            .unwrap()
            .into_iter()
            .next()
            .unwrap();

        assert_eq!(packet, result_packet);
    }

    // assert that the size of the processed header is the same as the given one.
    fn assert_right_header_size(
        delivery: DeliveryGuarantee,
        ordering: OrderingGuarantee,
        expected_header_size: usize,
    ) {
        let mut connection = create_virtual_connection();

        let buffer = vec![1; 500];

        let outgoing = connection
            .process_outgoing(
                GenericPacket::user_packet(&buffer, delivery, ordering),
                None,
                Instant::now(),
            )
            .unwrap();
        let mut iter = outgoing.into_iter();
        assert_eq!(
            iter.next().unwrap().contents().len() - buffer.len(),
            expected_header_size
        );
        if iter.next().is_some() {
            panic!("Expected not fragmented packet")
        }
    }
}
