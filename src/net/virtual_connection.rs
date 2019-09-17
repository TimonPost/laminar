use crate::{
    config::Config,
    error::{ErrorKind, PacketErrorKind, Result},
    infrastructure::{
        arranging::{Arranging, ArrangingSystem, OrderingSystem, SequencingSystem},
        AcknowledgmentHandler, CongestionHandler, Fragmentation, SentPacket,
    },
    net::constants::{
        ACKED_PACKET_HEADER, DEFAULT_ORDERING_STREAM, DEFAULT_SEQUENCING_STREAM,
        STANDARD_HEADER_SIZE,
    },
    net::events::{ConnectionEvent, ReceiveEvent},
    net::managers::{ConnectionManager, ConnectionState, SocketManager},
    packet::{
        DeliveryGuarantee, OrderingGuarantee, Outgoing, OutgoingPacket, OutgoingPacketBuilder,
        Packet, PacketReader, PacketType, SequenceNumber,
    },
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
    /// Last time we sent a packet to this client
    pub last_sent: Instant,
    /// The address of the remote endpoint
    pub remote_address: SocketAddr,

    ordering_system: OrderingSystem<Box<[u8]>>,
    sequencing_system: SequencingSystem<Box<[u8]>>,
    acknowledge_handler: AcknowledgmentHandler,
    congestion_handler: CongestionHandler,

    config: Config,
    fragmentation: Fragmentation,
    pub state_manager: Box<dyn ConnectionManager>,
    pub current_state: ConnectionState,
}

impl VirtualConnection {
    /// Creates and returns a new Connection that wraps the provided socket address
    pub fn new(
        addr: SocketAddr,
        config: &Config,
        time: Instant,
        state_manager: Box<dyn ConnectionManager>,
    ) -> VirtualConnection {
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
            state_manager,
            current_state: ConnectionState::Connecting,
        }
    }

    /// Determine if this connection should be dropped due to its state
    pub fn should_be_dropped(&self) -> bool {
        self.acknowledge_handler.packets_in_flight() > self.config.max_packets_in_flight
    }

    pub fn is_disconnected(&self) -> bool {
        if let ConnectionState::Disconnected(_) = self.current_state {
            true
        } else {
            false
        }
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

    pub fn get_current_state(&self) -> &ConnectionState {
        &self.current_state
    }

    /// This will create a heartbeat packet that is expected to be sent over the network
    pub fn create_and_process_heartbeat(&mut self, time: Instant) -> OutgoingPacket<'static> {
        self.last_sent = time;
        self.congestion_handler
            .process_outgoing(self.acknowledge_handler.local_sequence_num(), time);

        OutgoingPacketBuilder::new(&[])
            .with_default_header(
                PacketType::Heartbeat,
                DeliveryGuarantee::Unreliable,
                OrderingGuarantee::None,
            )
            .build()
    }

    /// This will pre-process the given buffer to be sent over the network.
    pub fn process_outgoing<'a>(
        &mut self,
        // PacketType is used by connection manager, because it can send all sorts of packets, except fragmented.
        // TODO would be nice to change this, so that it is impossible to provide bad value
        packet_type: PacketType,
        payload: &'a [u8],
        delivery_guarantee: DeliveryGuarantee,
        ordering_guarantee: OrderingGuarantee,
        last_item_identifier: Option<SequenceNumber>,
        time: Instant,
    ) -> Result<Outgoing<'a>> {
        match delivery_guarantee {
            DeliveryGuarantee::Unreliable => {
                if payload.len() <= self.config.receive_buffer_max_size {
                    let mut builder = OutgoingPacketBuilder::new(payload).with_default_header(
                        packet_type,
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

                let mut item_identifier_value = None;
                let outgoing = {
                    // spit the packet if the payload length is greater than the allowed fragment size.
                    if payload_length <= self.config.fragment_size {
                        let mut builder = OutgoingPacketBuilder::new(payload).with_default_header(
                            packet_type,
                            delivery_guarantee,
                            ordering_guarantee,
                        );

                        builder = builder.with_acknowledgment_header(
                            self.acknowledge_handler.local_sequence_num(),
                            self.acknowledge_handler.remote_sequence_num(),
                            self.acknowledge_handler.ack_bitfield(),
                        );

                        if let OrderingGuarantee::Ordered(stream_id) = ordering_guarantee {
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

                        if let OrderingGuarantee::Sequenced(stream_id) = ordering_guarantee {
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

                self.last_sent = time;
                self.congestion_handler
                    .process_outgoing(self.acknowledge_handler.local_sequence_num(), time);
                self.acknowledge_handler.process_outgoing(
                    payload,
                    ordering_guarantee,
                    item_identifier_value,
                );

                Ok(outgoing)
            }
        }
    }

    pub fn process_incoming(
        &mut self,
        received_data: &[u8],
        sender: &Sender<ConnectionEvent<ReceiveEvent>>,
        socket_manager: &mut dyn SocketManager,
        time: Instant,
    ) -> crate::Result<()> {
        // TODO pass buffer from somewhere else
        let mut buffer = [0; 1500];
        let received_data = self
            .state_manager
            .preprocess_incoming(received_data, &mut buffer)?;

        self.last_heard = time;

        let mut packet_reader = PacketReader::new(received_data);

        let header = packet_reader.read_standard_header()?;

        if !header.is_current_protocol() {
            return Err(ErrorKind::ProtocolVersionMismatch);
        }

        if header.is_heartbeat() {
            // Heartbeat packets are unreliable, unordered and empty packets.
            // We already updated our `self.last_heard` time, nothing else to be done.
            return Ok(());
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
                        self.process_packet_and_log_errors(
                            header.is_connection_manager_packet(),
                            sender,
                            socket_manager,
                            packet,
                            header.delivery_guarantee(),
                            OrderingGuarantee::Sequenced(Some(arranging_header.stream_id())),
                        );
                    }

                    return Ok(());
                }
                self.process_packet_and_log_errors(
                    header.is_connection_manager_packet(),
                    sender,
                    socket_manager,
                    packet_reader.read_payload(),
                    header.delivery_guarantee(),
                    header.ordering_guarantee(),
                );
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
                                self.process_packet_and_log_errors(
                                    false, // connection manager packet cannot be fragmented
                                    sender,
                                    socket_manager,
                                    payload.into_boxed_slice(),
                                    header.delivery_guarantee(),
                                    OrderingGuarantee::None,
                                );
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
                            stream.arrange(arranging_header.arranging_id(), payload)
                        {
                            self.process_packet_and_log_errors(
                                header.is_connection_manager_packet(),
                                sender,
                                socket_manager,
                                packet,
                                header.delivery_guarantee(),
                                OrderingGuarantee::Sequenced(Some(arranging_header.stream_id())),
                            );
                        }
                    } else if let OrderingGuarantee::Ordered(_id) = header.ordering_guarantee() {
                        let arranging_header = packet_reader.read_arranging_header(u16::from(
                            STANDARD_HEADER_SIZE + ACKED_PACKET_HEADER,
                        ))?;

                        let payload = packet_reader.read_payload();

                        let stream = self
                            .ordering_system
                            .get_or_create_stream(arranging_header.stream_id());
                        let arranged_packet =
                            stream.arrange(arranging_header.arranging_id(), payload);
                        let packets = arranged_packet
                            .into_iter()
                            .chain(stream.iter_mut())
                            .collect::<Vec<_>>();
                        for packet in packets {
                            self.process_packet_and_log_errors(
                                header.is_connection_manager_packet(),
                                sender,
                                socket_manager,
                                packet,
                                header.delivery_guarantee(),
                                OrderingGuarantee::Ordered(Some(arranging_header.stream_id())),
                            );
                        }
                    } else {
                        let payload = packet_reader.read_payload();

                        self.process_packet_and_log_errors(
                            header.is_connection_manager_packet(),
                            sender,
                            socket_manager,
                            payload,
                            header.delivery_guarantee(),
                            header.ordering_guarantee(),
                        );
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

    /// Pass packet to ConnectionManager, check for state changes, and return any newly generated packets
    fn process_packet(
        &mut self,
        is_connection_manager_packet: bool,
        sender: &Sender<ConnectionEvent<ReceiveEvent>>,
        payload: Box<[u8]>,
        delivery: DeliveryGuarantee,
        ordering: OrderingGuarantee,
    ) -> Result<()> {
        if !is_connection_manager_packet {
            if let ConnectionState::Connected(_) = self.current_state {
                sender.send(ConnectionEvent(
                    self.remote_address,
                    ReceiveEvent::Packet(Packet::new(
                        self.remote_address,
                        payload,
                        delivery,
                        ordering,
                    )),
                ))?;
            }
        } else {
            self.state_manager.process_protocol_data(payload.as_ref())?;
        }
        Ok(())
    }

    fn process_packet_and_log_errors(
        &mut self,
        is_connection_manager_packet: bool,
        sender: &Sender<ConnectionEvent<ReceiveEvent>>,
        socket_manager: &mut dyn SocketManager,
        payload: Box<[u8]>,
        delivery: DeliveryGuarantee,
        ordering: OrderingGuarantee,
    ) {
        if let Err(ref error) = self.process_packet(
            is_connection_manager_packet,
            sender,
            payload,
            delivery,
            ordering,
        ) {
            socket_manager.track_connection_error(
                &self.remote_address,
                error,
                "processing incomming packet",
            );
        }
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
    use crate::packet::{DeliveryGuarantee, OrderingGuarantee, Outgoing, Packet, PacketType};
    use crate::protocol_version::ProtocolVersion;
    use crate::SocketEvent;
    use byteorder::{BigEndian, WriteBytesExt};
    use crossbeam_channel::{unbounded, TryRecvError};
    use std::io::Write;
    use std::time::Instant;

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
                Instant::now(),
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
                Instant::now(),
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
                Instant::now(),
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
                Instant::now(),
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
                None,
                Instant::now(),
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
                None,
                Instant::now(),
            )
            .unwrap();

        connection
            .process_outgoing(
                &buffer,
                DeliveryGuarantee::Unreliable,
                OrderingGuarantee::Sequenced(None),
                None,
                Instant::now(),
            )
            .unwrap();

        connection
            .process_outgoing(
                &buffer,
                DeliveryGuarantee::Reliable,
                OrderingGuarantee::Ordered(None),
                None,
                Instant::now(),
            )
            .unwrap();

        connection
            .process_outgoing(
                &buffer,
                DeliveryGuarantee::Reliable,
                OrderingGuarantee::Sequenced(None),
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
            0,
        );

        assert_incoming_with_order(
            DeliveryGuarantee::Reliable,
            OrderingGuarantee::Ordered(Some(1)),
            &mut connection,
            Err(TryRecvError::Empty),
            2,
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
            Ok(SocketEvent::Packet(Packet::reliable_ordered(
                get_fake_addr(),
                PAYLOAD.to_vec(),
                Some(1),
            ))),
            1,
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

        connection
            .process_incoming(packet.as_slice(), &tx, Instant::now())
            .unwrap();

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

        connection
            .process_incoming(packet.as_slice(), &tx, Instant::now())
            .unwrap();

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
            .process_outgoing(&buffer, delivery, ordering, None, Instant::now())
            .unwrap();

        match outgoing {
            Outgoing::Packet(packet) => {
                assert_eq!(packet.contents().len() - buffer.len(), expected_header_size);
            }
            Outgoing::Fragments(_) => panic!("Expected packet got fragment"),
        }
    }
}
