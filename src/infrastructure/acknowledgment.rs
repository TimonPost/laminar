use crate::packet::{OrderingGuarantee, PacketType, SequenceNumber};
use crate::sequence_buffer::{sequence_greater_than, sequence_less_than, SequenceBuffer};
use std::collections::HashMap;

const REDUNDANT_PACKET_ACKS_SIZE: u16 = 32;
const DEFAULT_SEND_PACKETS_SIZE: usize = 256;

/// Responsible for handling the acknowledgment of packets.
pub struct AcknowledgmentHandler {
    // Local sequence number which we'll bump each time we send a new packet over the network
    sequence_number: SequenceNumber,
    // The last acked sequence number of the packets we've sent to the remote host.
    remote_ack_sequence_num: SequenceNumber,
    // Using a Hashmap to track every packet we send out so we can ensure that we can resend when
    // dropped.
    sent_packets: HashMap<u16, SentPacket>,
    // However, we can only reasonably ack up to REDUNDANT_PACKET_ACKS_SIZE + 1 packets on each
    // message we send so this should be that large
    received_packets: SequenceBuffer<ReceivedPacket>,
}

impl AcknowledgmentHandler {
    /// Constructs a new `AcknowledgmentHandler` with which you can perform acknowledgment operations.
    pub fn new() -> Self {
        AcknowledgmentHandler {
            sequence_number: 0,
            remote_ack_sequence_num: u16::max_value(),
            sent_packets: HashMap::with_capacity(DEFAULT_SEND_PACKETS_SIZE),
            received_packets: SequenceBuffer::with_capacity(REDUNDANT_PACKET_ACKS_SIZE + 1),
        }
    }

    /// Get the current number of not yet acknowledged packets
    pub fn packets_in_flight(&self) -> u16 {
        self.sent_packets.len() as u16
    }

    /// Returns the next sequence number to send.
    pub fn local_sequence_num(&self) -> SequenceNumber {
        self.sequence_number
    }

    /// Returns the last sequence number received from the remote host (+1)
    pub fn remote_sequence_num(&self) -> SequenceNumber {
        self.received_packets.sequence_num().wrapping_sub(1)
    }

    /// Returns the ack_bitfield corresponding to which of the past 32 packets we've
    /// successfully received.
    pub fn ack_bitfield(&self) -> u32 {
        let most_recent_remote_seq_num: u16 = self.remote_sequence_num();
        let mut ack_bitfield: u32 = 0;
        let mut mask: u32 = 1;

        // Iterate the past REDUNDANT_PACKET_ACKS_SIZE received packets and set the corresponding
        // bit for each packet which exists in the buffer.
        for i in 1..=REDUNDANT_PACKET_ACKS_SIZE {
            let sequence = most_recent_remote_seq_num.wrapping_sub(i);
            if self.received_packets.exists(sequence) {
                ack_bitfield |= mask;
            }
            mask <<= 1;
        }

        ack_bitfield
    }

    /// Process the incoming sequence number.
    ///
    /// - Acknowledge the incoming sequence number
    /// - Update dropped packets
    pub fn process_incoming(
        &mut self,
        remote_seq_num: u16,
        remote_ack_seq: u16,
        mut remote_ack_field: u32,
    ) {
        // We must ensure that self.remote_ack_sequence_num is always increasing (with wrapping)
        if sequence_greater_than(remote_ack_seq, self.remote_ack_sequence_num) {
            self.remote_ack_sequence_num = remote_ack_seq;
        }

        self.received_packets
            .insert(remote_seq_num, ReceivedPacket {});

        // The current remote_ack_seq was (clearly) received so we should remove it.
        self.sent_packets.remove(&remote_ack_seq);

        // The remote_ack_field is going to include whether or not the past 32 packets have been
        // received successfully. If so, we have no need to resend old packets.
        for i in 1..=REDUNDANT_PACKET_ACKS_SIZE {
            let ack_sequence = remote_ack_seq.wrapping_sub(i);
            if remote_ack_field & 1 == 1 {
                self.sent_packets.remove(&ack_sequence);
            }
            remote_ack_field >>= 1;
        }
    }

    /// Enqueue the outgoing packet for acknowledgment.
    pub fn process_outgoing(
        &mut self,
        packet_type: PacketType,
        payload: &[u8],
        ordering_guarantee: OrderingGuarantee,
        item_identifier: Option<SequenceNumber>,
    ) {
        self.sent_packets.insert(
            self.sequence_number,
            SentPacket {
                packet_type,
                payload: Box::from(payload),
                ordering_guarantee,
                item_identifier,
            },
        );

        // Bump the local sequence number for the next outgoing packet.
        self.sequence_number = self.sequence_number.wrapping_add(1);
    }

    /// Returns a `Vec` of packets we believe have been dropped.
    pub fn dropped_packets(&mut self) -> Vec<SentPacket> {
        let mut sent_sequences: Vec<SequenceNumber> = self.sent_packets.keys().cloned().collect();
        sent_sequences.sort();

        let remote_ack_sequence = self.remote_ack_sequence_num;
        sent_sequences
            .into_iter()
            .filter(|s| {
                if sequence_less_than(*s, remote_ack_sequence) {
                    remote_ack_sequence.wrapping_sub(*s) > REDUNDANT_PACKET_ACKS_SIZE
                } else {
                    false
                }
            })
            .flat_map(|s| self.sent_packets.remove(&s))
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SentPacket {
    pub packet_type: PacketType,
    pub payload: Box<[u8]>,
    pub ordering_guarantee: OrderingGuarantee,
    pub item_identifier: Option<SequenceNumber>,
}

// TODO: At some point we should put something useful here. Possibly timing information or total
// bytes sent for metrics tracking.
#[derive(Clone, Default)]
pub struct ReceivedPacket;

#[cfg(test)]
mod test {
    use crate::infrastructure::acknowledgment::ReceivedPacket;
    use crate::infrastructure::{AcknowledgmentHandler, SentPacket};
    use crate::packet::OrderingGuarantee;
    use log::debug;

    #[test]
    fn increment_local_seq_num_on_process_outgoing() {
        let mut handler = AcknowledgmentHandler::new();
        assert_eq!(handler.local_sequence_num(), 0);
        for i in 0..10 {
            handler.process_outgoing(vec![].as_slice(), OrderingGuarantee::None, None);
            assert_eq!(handler.local_sequence_num(), i + 1);
        }
    }

    #[test]
    fn local_seq_num_wraps_on_overflow() {
        let mut handler = AcknowledgmentHandler::new();
        handler.sequence_number = u16::max_value();
        handler.process_outgoing(vec![].as_slice(), OrderingGuarantee::None, None);
        assert_eq!(handler.local_sequence_num(), 0);
    }

    #[test]
    fn ack_bitfield_with_empty_receive() {
        let handler = AcknowledgmentHandler::new();
        assert_eq!(handler.ack_bitfield(), 0)
    }

    #[test]
    fn ack_bitfield_with_some_values() {
        let mut handler = AcknowledgmentHandler::new();
        handler
            .received_packets
            .insert(0, ReceivedPacket::default());
        handler
            .received_packets
            .insert(1, ReceivedPacket::default());
        handler
            .received_packets
            .insert(3, ReceivedPacket::default());
        assert_eq!(handler.remote_sequence_num(), 3);
        assert_eq!(handler.ack_bitfield(), 0b110)
    }

    #[test]
    fn packet_is_not_acked() {
        let mut handler = AcknowledgmentHandler::new();

        handler.sequence_number = 0;
        handler.process_outgoing(vec![1, 2, 3].as_slice(), OrderingGuarantee::None, None);
        handler.sequence_number = 40;
        handler.process_outgoing(vec![1, 2, 4].as_slice(), OrderingGuarantee::None, None);

        static ARBITRARY: u16 = 23;
        handler.process_incoming(ARBITRARY, 40, 0);

        assert_eq!(
            handler.dropped_packets(),
            vec![SentPacket {
                payload: vec![1, 2, 3].into_boxed_slice(),
                ordering_guarantee: OrderingGuarantee::None,
                item_identifier: None,
            }]
        );
    }

    #[test]
    fn acking_500_packets_without_packet_drop() {
        let mut handler = AcknowledgmentHandler::new();
        let mut other = AcknowledgmentHandler::new();

        for i in 0..500 {
            handler.sequence_number = i;
            handler.process_outgoing(vec![1, 2, 3].as_slice(), OrderingGuarantee::None, None);

            other.process_incoming(i, handler.remote_sequence_num(), handler.ack_bitfield());
            handler.process_incoming(i, other.remote_sequence_num(), other.ack_bitfield());
        }

        assert_eq!(handler.dropped_packets().len(), 0);
    }

    #[test]
    fn acking_many_packets_with_packet_drop() {
        let mut handler = AcknowledgmentHandler::new();
        let mut other = AcknowledgmentHandler::new();

        let mut drop_count = 0;

        for i in 0..100 {
            handler.process_outgoing(vec![1, 2, 3].as_slice(), OrderingGuarantee::None, None);
            handler.sequence_number = i;

            // dropping every 4th with modulo's
            if i % 4 == 0 {
                debug!("Dropping packet: {}", drop_count);
                drop_count += 1;
            } else {
                // We send them a packet
                other.process_incoming(i, handler.remote_sequence_num(), handler.ack_bitfield());
                // Skipped: other.process_outgoing
                // And it makes it back
                handler.process_incoming(i, other.remote_sequence_num(), other.ack_bitfield());
            }
        }
        assert_eq!(drop_count, 25);
        assert_eq!(handler.remote_sequence_num(), 99);
        // Ack reads from right to left. So we know we have 99 since it's the last one we received.
        // Then, the first bit is acking 98, then 97, then we're missing 96 which makes sense
        // because 96 is evenly divisible by 4 and so on...
        assert_eq!(handler.ack_bitfield(), 0b10111011101110111011101110111011);
        assert_eq!(handler.dropped_packets().len(), 17);
    }

    #[test]
    fn remote_seq_num_will_be_updated() {
        let mut handler = AcknowledgmentHandler::new();
        assert_eq!(handler.remote_sequence_num(), 65535);
        handler.process_incoming(0, 0, 0);
        assert_eq!(handler.remote_sequence_num(), 0);
        handler.process_incoming(1, 0, 0);
        assert_eq!(handler.remote_sequence_num(), 1);
    }

    #[test]
    fn processing_a_full_set_of_packets() {
        let mut handler = AcknowledgmentHandler::new();
        for i in 0..33 {
            handler.process_incoming(i, 0, 0);
        }
        assert_eq!(handler.remote_sequence_num(), 32);
        assert_eq!(handler.ack_bitfield(), !0);
    }

    #[test]
    fn test_process_outgoing() {
        let mut handler = AcknowledgmentHandler::new();
        handler.process_outgoing(vec![1, 2, 3].as_slice(), OrderingGuarantee::None, None);
        assert_eq!(handler.sent_packets.len(), 1);
        assert_eq!(handler.local_sequence_num(), 1);
    }

    #[test]
    fn remote_ack_seq_must_never_be_less_than_prior() {
        let mut handler = AcknowledgmentHandler::new();
        // Second packet received before first
        handler.process_incoming(1, 1, 1);
        assert_eq!(handler.remote_ack_sequence_num, 1);
        // First packet received
        handler.process_incoming(0, 0, 0);
        assert_eq!(handler.remote_ack_sequence_num, 1);
    }

    #[test]
    fn remote_ack_seq_must_never_be_less_than_prior_wrap_boundary() {
        let mut handler = AcknowledgmentHandler::new();
        // newer packet received before first
        handler.process_incoming(1, 0, 1);
        assert_eq!(handler.remote_ack_sequence_num, 0);
        // earlier packet received
        handler.process_incoming(0, u16::max_value(), 0);
        assert_eq!(handler.remote_ack_sequence_num, 0);
    }
}
