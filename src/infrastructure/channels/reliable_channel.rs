use super::Channel;

use crate::config::Config;
use crate::error::{PacketErrorKind, Result};
use crate::infrastructure::{DeliveryMethod, Fragmentation};
use crate::net::{ExternalAcks, LocalAckRecord, NetworkQuality, RttMeasurer};
use crate::packet::header::{AckedPacketHeader, HeaderReader, HeaderWriter, StandardHeader};
use crate::packet::{PacketData, PacketTypeId};
use crate::sequence_buffer::{CongestionData, SequenceBuffer};

use log::error;
use std::io::Cursor;
use std::time::Instant;

/// This channel should be used for processing packets reliable. All packets will be sent and received, ordering depends on given 'ordering' parameter.
///
/// *Details*
///
/// |   Packet Drop   | Packet Duplication | Packet Order     | Packet Fragmentation | Packet Delivery |
/// | :-------------: | :-------------:    | :-------------:  | :-------------:      | :-------------: |
/// |       No        |      No            |     Optional     |      Yes             |       Yes       |
///
/// Basically this is almost has all features TCP has.
/// Receive every packet and if specified (file downloading for example) in order (any missing keeps the later ones buffered until they are received).
pub struct ReliableChannel {
    // settings
    ordered: bool,
    config: Config,

    // reliability control
    seq_num: u16,
    waiting_packets: LocalAckRecord,
    their_acks: ExternalAcks,
    dropped_packets: Vec<Box<[u8]>>,

    // congestion control
    rtt_measurer: RttMeasurer,
    congestion_data: SequenceBuffer<CongestionData>,
    _quality: NetworkQuality,
    rtt: f32,
}

impl ReliableChannel {
    /// Creates a new instance of the reliable channel by specifying if channel needs to order incoming packets.
    pub fn new(ordered: bool, config: &Config) -> ReliableChannel {
        ReliableChannel {
            // settings
            ordered,
            config: config.clone(),

            // reliability control
            seq_num: 0,
            waiting_packets: Default::default(),
            their_acks: Default::default(),
            dropped_packets: Vec::new(),

            // congestion control
            rtt_measurer: RttMeasurer::new(config),
            congestion_data: SequenceBuffer::with_capacity(<u16>::max_value() as usize),
            _quality: NetworkQuality::Good,
            rtt: 0.0,
        }
    }

    /// Checks if channel is ordered or not
    #[allow(dead_code)]
    pub fn is_ordered(&self) -> bool {
        self.ordered
    }

    /// Check if this channel has dropped packets.
    ///
    /// You could directly call `ReliableChannel::drain_dropped_packets()` and if it returns an empty vector you know there are no packets.
    /// But draining a vector will have its extra check logic even if it's empty.
    /// So that's why this function exists just a little shortcut to check if there are dropped packets which will be faster at the end.
    pub fn has_dropped_packets(&self) -> bool {
        !self.dropped_packets.is_empty()
    }

    /// Creates a draining iterator that removes dropped packets and yield the ones that are removed.
    ///
    /// So why drain?
    /// You have to think about the packet flow first.
    /// 1. Once we send a packet we place it in a queue until acknowledged.
    /// 2. If the packet doesn't get acknowledged in some time it will be dropped.
    /// 3. When we notice the packet drop we directly want to resend the dropped packet.
    /// 4. Once we notice that we start at '1' again.
    ///
    /// So keeping track of old dropped packets does not make sense, at least for now.
    /// We except when dropped packets are retrieved they will be sent out so we don't need to keep track of them internally the caller of this function will have ownership over them after the call.
    pub fn drain_dropped_packets(&mut self) -> Vec<Box<[u8]>> {
        self.dropped_packets.drain(..).collect()
    }
}

impl Channel for ReliableChannel {
    /// This will pre-process a reliable packet
    ///
    /// 1. Add congestion data entry so that it can be monitored.
    /// 2. Queue new packet in acknowledgement system.
    /// 3. Fragmentation of the payload.
    fn process_outgoing(
        &mut self,
        payload: &[u8],
        delivery_method: DeliveryMethod,
    ) -> Result<PacketData> {
        if payload.len() > self.config.max_packet_size {
            error!(
                "Packet too large: Attempting to send {}, max={}",
                payload.len(),
                self.config.max_packet_size
            );
            Err(PacketErrorKind::ExceededMaxPacketSize)?;
        }

        // queue congestion data.
        self.congestion_data.insert(
            CongestionData::new(self.seq_num, Instant::now()),
            self.seq_num,
        );

        // queue packet for awaiting acknowledgement.
        self.waiting_packets.enqueue(self.seq_num, &payload);

        // calculate size for our packet data.
        // safe cast because max packet size is u16
        let payload_length = payload.len() as u16;
        let packet_data_size =
            Fragmentation::total_fragments_needed(payload_length, self.config.fragment_size);
        let mut packet_data = PacketData::with_capacity(packet_data_size as usize);

        let packet_type = if packet_data_size > 1 {
            PacketTypeId::Fragment
        } else {
            PacketTypeId::Packet
        };

        // create our reliable header and write it to an buffer.
        let header = AckedPacketHeader::new(
            StandardHeader::new(delivery_method, packet_type),
            self.seq_num,
            self.their_acks.last_seq,
            self.their_acks.field,
        );
        let mut buffer = Vec::with_capacity(header.size() as usize);
        header.parse(&mut buffer)?;

        // spit the packet if the payload length is greater than the allowed fragment size.
        if payload_length <= self.config.fragment_size {
            packet_data.add_fragment(&buffer, payload)?;
        } else {
            Fragmentation::spit_into_fragments(payload, header, &mut packet_data, &self.config)?;
        }

        // increase local sequence number.
        self.seq_num = self.seq_num.wrapping_add(1);

        Ok(packet_data)
    }

    /// Process a packet on receive.
    ///
    /// 1. Read reliable header.
    /// 2. Update acknowledgement data.
    /// 3. Calculate RTT time.
    /// 4. Update dropped packets.
    fn process_incoming<'d>(&mut self, buffer: &'d [u8]) -> Result<&'d [u8]> {
        let mut cursor = Cursor::new(buffer);
        let acked_header = AckedPacketHeader::read(&mut cursor)?;

        self.their_acks.ack(acked_header.seq);

        // update congestion information.
        let congestion_data = self.congestion_data.get_mut(acked_header.ack_seq());
        self.rtt = self.rtt_measurer.get_rtt(congestion_data);

        // Update dropped packets if there are any.
        let dropped_packets = self
            .waiting_packets
            .ack(acked_header.ack_seq(), acked_header.ack_field());

        self.dropped_packets = dropped_packets.into_iter().map(|(_, p)| p).collect();

        Ok(&buffer[acked_header.size() as usize..buffer.len()])
    }
}
