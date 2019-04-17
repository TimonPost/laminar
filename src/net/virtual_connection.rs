use crate::{
    config::Config,
    error::{ErrorKind, PacketErrorKind, Result},
    infrastructure::{
        arranging::{Arranging, ArrangingSystem, OrderingSystem, SequencingSystem},
        AcknowledgementHandler, CongestionHandler, Fragmentation,
    },
    net::constants::{
        ACKED_PACKET_HEADER, DEFAULT_ORDERING_STREAM, DEFAULT_SEQUENCING_STREAM,
        STANDARD_HEADER_SIZE,
    },
    packet::{
        DeliveryGuarantee, OrderingGuarantee, Outgoing, OutgoingPacketBuilder, Packet,
        PacketReader, PacketType,
    },
    SocketEvent,
};

use crossbeam_channel::{self, Sender};
use std::fmt;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

/// Contains the information about a certain 'virtual connection' over udp.
/// This connections also keeps track of network quality, processing packets, buffering data related to connection etc.
pub struct VirtualConnection {
    /// Last time we received a packet from this client
    pub last_heard: Instant,
    /// The address of the remote endpoint
    pub remote_address: SocketAddr,

    ordering_system: OrderingSystem<Box<[u8]>>,
    sequencing_system: SequencingSystem<Box<[u8]>>,
    acknowledge_handler: AcknowledgementHandler,
    congestion_handler: CongestionHandler,

    config: Config,
    fragmentation: Fragmentation,
}

impl VirtualConnection {
    /// Creates and returns a new Connection that wraps the provided socket address
    pub fn new(addr: SocketAddr, config: &Config) -> VirtualConnection {
        VirtualConnection {
            last_heard: Instant::now(),
            remote_address: addr,
            ordering_system: OrderingSystem::new(),
            sequencing_system: SequencingSystem::new(),
            acknowledge_handler: AcknowledgementHandler::new(),
            congestion_handler: CongestionHandler::new(config),
            fragmentation: Fragmentation::new(config),
            config: config.to_owned(),
        }
    }

    /// Returns a Duration representing the interval since we last heard from the client
    pub fn last_heard(&self) -> Duration {
        let now = Instant::now();
        now.duration_since(self.last_heard)
    }

    /// This pre-process the given buffer to be send over the network.
    pub fn process_outgoing<'a>(
        &mut self,
        payload: &'a [u8],
        delivery_guarantee: DeliveryGuarantee,
        ordering_guarantee: OrderingGuarantee,
    ) -> Result<Outgoing<'a>> {
        match delivery_guarantee {
            DeliveryGuarantee::Unreliable => {
                if payload.len() <= self.config.receive_buffer_max_size {
                    let mut builder = OutgoingPacketBuilder::new(payload).with_default_header(
                        PacketType::Packet,
                        delivery_guarantee,
                        ordering_guarantee,
                    );

                    if let OrderingGuarantee::Sequenced(stream_id) = ordering_guarantee {
                        let item_identifier = self
                            .sequencing_system
                            .get_or_create_stream(stream_id.unwrap_or(DEFAULT_SEQUENCING_STREAM))
                            .new_item_identifier();

                        builder = builder.with_sequencing_header(item_identifier as u16, stream_id);
                    };

                    Ok(Outgoing::Packet(builder.build()))
                } else {
                    Err(ErrorKind::PacketError(
                        PacketErrorKind::ExceededMaxPacketSize,
                    ))
                }
            }
            DeliveryGuarantee::Reliable => {
                let payload_length = payload.len() as u16;

                let outgoing = {
                    // spit the packet if the payload length is greater than the allowed fragment size.
                    if payload_length <= self.config.fragment_size {
                        let mut builder = OutgoingPacketBuilder::new(payload).with_default_header(
                            PacketType::Packet,
                            delivery_guarantee,
                            ordering_guarantee,
                        );

                        builder = builder.with_acknowledgement_header(
                            self.acknowledge_handler.seq_num,
                            self.acknowledge_handler.last_seq(),
                            self.acknowledge_handler.bit_mask(),
                        );

                        if let OrderingGuarantee::Ordered(stream_id) = ordering_guarantee {
                            let item_identifier = self
                                .ordering_system
                                .get_or_create_stream(stream_id.unwrap_or(DEFAULT_ORDERING_STREAM))
                                .new_item_identifier();

                            builder =
                                builder.with_ordering_header(item_identifier as u16, stream_id);
                        };

                        if let OrderingGuarantee::Sequenced(stream_id) = ordering_guarantee {
                            let item_identifier = self
                                .sequencing_system
                                .get_or_create_stream(
                                    stream_id.unwrap_or(DEFAULT_SEQUENCING_STREAM),
                                )
                                .new_item_identifier();

                            builder =
                                builder.with_sequencing_header(item_identifier as u16, stream_id);
                        };

                        Outgoing::Packet(builder.build())
                    } else {
                        Outgoing::Fragments(
                            Fragmentation::spit_into_fragments(payload, &self.config)?
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
                                            PacketType::Fragment,
                                            delivery_guarantee,
                                            ordering_guarantee,
                                        );

                                    builder = builder.with_fragment_header(
                                        self.acknowledge_handler.seq_num,
                                        fragment_id as u8,
                                        fragments_needed,
                                    );

                                    if fragment_id == 0 {
                                        builder = builder.with_acknowledgement_header(
                                            self.acknowledge_handler.seq_num,
                                            self.acknowledge_handler.last_seq(),
                                            self.acknowledge_handler.bit_mask(),
                                        );
                                    }

                                    builder.build()
                                })
                                .collect(),
                        )
                    }
                };

                self.congestion_handler
                    .process_outgoing(self.acknowledge_handler.seq_num);
                self.acknowledge_handler.process_outgoing(payload);

                self.acknowledge_handler.seq_num = self.acknowledge_handler.seq_num.wrapping_add(1);

                Ok(outgoing)
            }
        }
    }

    /// This processes the incoming data and returns an packet if the data is complete.
    pub fn process_incoming(
        &mut self,
        received_data: &[u8],
        sender: &Sender<SocketEvent>,
    ) -> crate::Result<()> {
        self.last_heard = Instant::now();

        let mut packet_reader = PacketReader::new(received_data);

        let header = packet_reader.read_standard_header()?;

        if !header.is_current_protocol() {
            return Err(ErrorKind::ProtocolVersionMismatch);
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

                    if let Some(packet) =
                        stream.arrange(arranging_header.arranging_id() as usize, payload)
                    {
                        Self::queue_packet(
                            sender,
                            packet,
                            self.remote_address,
                            header.delivery_guarantee(),
                            OrderingGuarantee::Sequenced(Some(arranging_header.stream_id())),
                        )?;
                    }

                    return Ok(());
                }

                Self::queue_packet(
                    sender,
                    packet_reader.read_payload(),
                    self.remote_address,
                    header.delivery_guarantee(),
                    header.ordering_guarantee(),
                )?;
            }
            DeliveryGuarantee::Reliable => {
                if header.is_fragment() {
                    if let Ok((fragment_header, acked_header)) = packet_reader.read_fragment() {
                        let payload = packet_reader.read_payload();

                        match self
                            .fragmentation
                            .handle_fragment(fragment_header, &payload)
                        {
                            Ok(Some(payload)) => {
                                Self::queue_packet(
                                    sender,
                                    payload.into_boxed_slice(),
                                    self.remote_address,
                                    header.delivery_guarantee(),
                                    OrderingGuarantee::None,
                                )?;
                            }
                            Ok(None) => return Ok(()),
                            Err(e) => return Err(e),
                        };

                        if let Some(acked_header) = acked_header {
                            self.congestion_handler
                                .process_incoming(acked_header.sequence());
                            self.acknowledge_handler.process_incoming(
                                acked_header.sequence(),
                                acked_header.ack_seq(),
                                acked_header.ack_field(),
                            );
                        }
                    }
                } else {
                    let acked_header = packet_reader.read_acknowledge_header()?;

                    if let OrderingGuarantee::Sequenced(_) = header.ordering_guarantee() {
                        let arranging_header = packet_reader.read_arranging_header(u16::from(
                            STANDARD_HEADER_SIZE + ACKED_PACKET_HEADER,
                        ))?;

                        let payload = packet_reader.read_payload();

                        let stream = self
                            .sequencing_system
                            .get_or_create_stream(arranging_header.stream_id());

                        if let Some(packet) =
                            stream.arrange(arranging_header.arranging_id() as usize, payload)
                        {
                            Self::queue_packet(
                                sender,
                                packet,
                                self.remote_address,
                                header.delivery_guarantee(),
                                OrderingGuarantee::Sequenced(Some(arranging_header.stream_id())),
                            )?;
                        }
                    }

                    if let OrderingGuarantee::Ordered(_id) = header.ordering_guarantee() {
                        let arranging_header = packet_reader.read_arranging_header(u16::from(
                            STANDARD_HEADER_SIZE + ACKED_PACKET_HEADER,
                        ))?;

                        let payload = packet_reader.read_payload();

                        let stream = self
                            .ordering_system
                            .get_or_create_stream(arranging_header.stream_id());

                        if let Some(packet) =
                            stream.arrange(arranging_header.arranging_id() as usize, payload)
                        {
                            Self::queue_packet(
                                sender,
                                packet,
                                self.remote_address,
                                header.delivery_guarantee(),
                                OrderingGuarantee::Ordered(Some(arranging_header.stream_id())),
                            )?;

                            while let Some(packet) = stream.iter_mut().next() {
                                Self::queue_packet(
                                    sender,
                                    packet,
                                    self.remote_address,
                                    header.delivery_guarantee(),
                                    OrderingGuarantee::Ordered(Some(arranging_header.stream_id())),
                                )?;
                            }
                        }
                    } else {
                        let payload = packet_reader.read_payload();

                        Self::queue_packet(
                            sender,
                            payload,
                            self.remote_address,
                            header.delivery_guarantee(),
                            header.ordering_guarantee(),
                        )?;
                    }

                    self.congestion_handler
                        .process_incoming(acked_header.sequence());
                    self.acknowledge_handler.process_incoming(
                        acked_header.sequence(),
                        acked_header.ack_seq(),
                        acked_header.ack_field(),
                    );
                }
            }
        }

        Ok(())
    }

    fn queue_packet(
        tx: &Sender<SocketEvent>,
        payload: Box<[u8]>,
        remote_addr: SocketAddr,
        delivery: DeliveryGuarantee,
        ordering: OrderingGuarantee,
    ) -> Result<()> {
        tx.send(SocketEvent::Packet(Packet::new(
            remote_addr,
            payload,
            delivery,
            ordering,
        )))?;
        Ok(())
    }

    /// This will gather dropped packets from the reliable channels.
    ///
    /// Note that after requesting dropped packets the dropped packets will be removed from this client.
    pub fn gather_dropped_packets(&mut self) -> Vec<Box<[u8]>> {
        self.acknowledge_handler.dropped_packets.drain(..).collect()
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
    use crate::packet::{DeliveryGuarantee, OrderingGuarantee, Outgoing, Packet, PacketType};
    use crate::protocol_version::ProtocolVersion;
    use crate::SocketEvent;
    use byteorder::{BigEndian, WriteBytesExt};
    use crossbeam_channel::{unbounded, TryRecvError};
    use std::io::Write;

    const PAYLOAD: [u8; 4] = [1, 2, 3, 4];

    #[test]
    fn assure_right_fragmentation() {
        let mut protocol_version = Vec::new();
        protocol_version
            .write_u16::<BigEndian>(ProtocolVersion::get_crc16())
            .unwrap();

        let standard_header = [protocol_version, vec![1, 1, 2]].concat();

        let acked_header = vec![1, 0, 0, 2, 0, 0, 0, 3];
        let first_fragment = vec![0, 1, 1, 3];
        let second_fragment = vec![0, 1, 2, 3];
        let third_fragment = vec![0, 1, 3, 3];

        let (tx, rx) = unbounded::<SocketEvent>();

        let mut connection = create_virtual_connection();
        connection
            .process_incoming(
                [standard_header.as_slice(), acked_header.as_slice()]
                    .concat()
                    .as_slice(),
                &tx,
            )
            .unwrap();
        assert!(rx.try_recv().is_err());
        connection
            .process_incoming(
                [
                    standard_header.as_slice(),
                    first_fragment.as_slice(),
                    &PAYLOAD,
                ]
                .concat()
                .as_slice(),
                &tx,
            )
            .unwrap();
        assert!(rx.try_recv().is_err());
        connection
            .process_incoming(
                [
                    standard_header.as_slice(),
                    second_fragment.as_slice(),
                    &PAYLOAD,
                ]
                .concat()
                .as_slice(),
                &tx,
            )
            .unwrap();
        assert!(rx.try_recv().is_err());
        connection
            .process_incoming(
                [
                    standard_header.as_slice(),
                    third_fragment.as_slice(),
                    &PAYLOAD,
                ]
                .concat()
                .as_slice(),
                &tx,
            )
            .unwrap();

        let complete_fragment = rx.try_recv().unwrap();

        match complete_fragment {
            SocketEvent::Packet(fragment) => assert_eq!(
                fragment.payload(),
                &*[PAYLOAD, PAYLOAD, PAYLOAD].concat().into_boxed_slice()
            ),
            _ => {
                panic!("Expected fragment other result.");
            }
        }
    }

    #[test]
    fn expect_fragmentation() {
        let mut connection = create_virtual_connection();

        let buffer = vec![1; 4000];

        let outgoing = connection
            .process_outgoing(
                &buffer,
                DeliveryGuarantee::Reliable,
                OrderingGuarantee::Ordered(None),
            )
            .unwrap();

        match outgoing {
            Outgoing::Packet(_) => panic!("Expected fragment got packet"),
            Outgoing::Fragments(fragments) => {
                assert_eq!(fragments.len(), 4);
            }
        }
    }

    #[test]
    fn assure_correct_outgoing_processing() {
        let mut connection = create_virtual_connection();

        let buffer = vec![1; 1000];

        connection
            .process_outgoing(
                &buffer,
                DeliveryGuarantee::Unreliable,
                OrderingGuarantee::None,
            )
            .unwrap();

        connection
            .process_outgoing(
                &buffer,
                DeliveryGuarantee::Unreliable,
                OrderingGuarantee::Sequenced(None),
            )
            .unwrap();

        connection
            .process_outgoing(
                &buffer,
                DeliveryGuarantee::Reliable,
                OrderingGuarantee::Ordered(None),
            )
            .unwrap();

        connection
            .process_outgoing(
                &buffer,
                DeliveryGuarantee::Reliable,
                OrderingGuarantee::Sequenced(None),
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
            Ok(SocketEvent::Packet(Packet::unreliable_sequenced(
                get_fake_addr(),
                PAYLOAD.to_vec(),
                Some(1),
            ))),
            1,
        );

        assert_incoming_with_order(
            DeliveryGuarantee::Unreliable,
            OrderingGuarantee::Sequenced(Some(1)),
            &mut connection,
            Ok(SocketEvent::Packet(Packet::unreliable_sequenced(
                get_fake_addr(),
                PAYLOAD.to_vec(),
                Some(1),
            ))),
            3,
        );

        assert_incoming_with_order(
            DeliveryGuarantee::Unreliable,
            OrderingGuarantee::Sequenced(Some(1)),
            &mut connection,
            Err(TryRecvError::Empty),
            2,
        );

        assert_incoming_with_order(
            DeliveryGuarantee::Unreliable,
            OrderingGuarantee::Sequenced(Some(1)),
            &mut connection,
            Ok(SocketEvent::Packet(Packet::unreliable_sequenced(
                get_fake_addr(),
                PAYLOAD.to_vec(),
                Some(1),
            ))),
            4,
        );

        assert_incoming_with_order(
            DeliveryGuarantee::Reliable,
            OrderingGuarantee::Sequenced(Some(1)),
            &mut connection,
            Ok(SocketEvent::Packet(Packet::reliable_sequenced(
                get_fake_addr(),
                PAYLOAD.to_vec(),
                Some(1),
            ))),
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
            Ok(SocketEvent::Packet(Packet::reliable_ordered(
                get_fake_addr(),
                PAYLOAD.to_vec(),
                Some(1),
            ))),
            1,
        );

        assert_incoming_with_order(
            DeliveryGuarantee::Reliable,
            OrderingGuarantee::Ordered(Some(1)),
            &mut connection,
            Err(TryRecvError::Empty),
            3,
        );

        assert_incoming_with_order(
            DeliveryGuarantee::Reliable,
            OrderingGuarantee::Ordered(Some(1)),
            &mut connection,
            Err(TryRecvError::Empty),
            4,
        );

        assert_incoming_with_order(
            DeliveryGuarantee::Reliable,
            OrderingGuarantee::Ordered(Some(1)),
            &mut connection,
            Ok(SocketEvent::Packet(Packet::reliable_ordered(
                get_fake_addr(),
                PAYLOAD.to_vec(),
                Some(1),
            ))),
            2,
        );
    }

    #[test]
    fn assure_correct_processing_of_incoming() {
        let mut connection = create_virtual_connection();

        assert_incoming_without_order(
            DeliveryGuarantee::Unreliable,
            &mut connection,
            SocketEvent::Packet(Packet::unreliable(get_fake_addr(), PAYLOAD.to_vec())),
        );

        assert_incoming_without_order(
            DeliveryGuarantee::Reliable,
            &mut connection,
            SocketEvent::Packet(Packet::reliable_unordered(
                get_fake_addr(),
                PAYLOAD.to_vec(),
            )),
        );

        assert_incoming_with_order(
            DeliveryGuarantee::Unreliable,
            OrderingGuarantee::Sequenced(Some(1)),
            &mut connection,
            Ok(SocketEvent::Packet(Packet::unreliable_sequenced(
                get_fake_addr(),
                PAYLOAD.to_vec(),
                Some(1),
            ))),
            1,
        );

        assert_incoming_with_order(
            DeliveryGuarantee::Reliable,
            OrderingGuarantee::Ordered(Some(1)),
            &mut connection,
            Ok(SocketEvent::Packet(Packet::reliable_ordered(
                get_fake_addr(),
                PAYLOAD.to_vec(),
                Some(1),
            ))),
            1,
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

    /// ======= helper functions =========
    fn create_virtual_connection() -> VirtualConnection {
        VirtualConnection::new(get_fake_addr(), &Config::default())
    }

    fn get_fake_addr() -> std::net::SocketAddr {
        "127.0.0.1:0".parse().unwrap()
    }

    // assert that the processing of the given `DeliveryGuarantee` and `OrderingGuarantee` results into the given `result_event`
    fn assert_incoming_with_order(
        delivery: DeliveryGuarantee,
        ordering: OrderingGuarantee,
        connection: &mut VirtualConnection,
        result_event: Result<SocketEvent, TryRecvError>,
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

        let (tx, rx) = unbounded::<SocketEvent>();

        connection.process_incoming(packet.as_slice(), &tx).unwrap();

        let event = rx.try_recv();

        match event {
            Ok(val) => assert_eq!(val, result_event.unwrap()),
            Err(e) => assert_eq!(e, result_event.err().unwrap()),
        }
    }

    // assert that the given `DeliveryGuarantee` results into the given `SocketEvent` after processing.
    fn assert_incoming_without_order(
        delivery: DeliveryGuarantee,
        connection: &mut VirtualConnection,
        result_event: SocketEvent,
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

        let (tx, rx) = unbounded::<SocketEvent>();

        connection.process_incoming(packet.as_slice(), &tx).unwrap();

        let event = rx.try_recv();

        assert_eq!(event, Ok(result_event));
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
            .process_outgoing(&buffer, delivery, ordering)
            .unwrap();

        match outgoing {
            Outgoing::Packet(packet) => {
                assert_eq!(packet.contents().len() - buffer.len(), expected_header_size);
            }
            Outgoing::Fragments(_) => panic!("Expected packet got fragment"),
        }
    }
}
