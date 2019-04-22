use crate::packet::OrderingGuarantee;
use crate::sequence_buffer::SequenceBuffer;
use crate::packet::SequenceNumber;
use std::collections::HashMap;

const REDUNDANT_PACKET_ACKS_SIZE: u16 = 32;

/// Responsible for handling the acknowledgement of packets.
pub struct AcknowledgementHandler {
    // Local sequence number which we'll bump each time we send a new packet over the network
    sequence_number: SequenceNumber,
    // Using a Hashmap to track every packet we send out so we can ensure that we can resend when
    // dropped.
    sent_packets: HashMap<u16, SentPacket>,
    // However, we can only reasonably ack up to REDUNDANT_PACKET_ACKS_SIZE + 1 packets on each
    // message we send so this should be REDUNDANT_PACKET_ACKS_SIZE large.
    received_packets: SequenceBuffer<ReceivedPacket>,

    pub dropped_packets: Vec<SentPacket>,
}

impl AcknowledgementHandler {
    /// Constructs a new `AcknowledgementHandler` with which you can perform acknowledgement operations.
    pub fn new() -> AcknowledgementHandler {
        AcknowledgementHandler {
            sequence_number: 0,
            sent_packets: HashMap::new(),
            received_packets: SequenceBuffer::with_capacity(REDUNDANT_PACKET_ACKS_SIZE),
            dropped_packets: Vec::new(),
        }
    }
}

impl AcknowledgementHandler {
    /// Returns the next sequence number to send.
    pub fn local_sequence_num(&self) -> SequenceNumber {
        self.sequence_number
    }

    /// Returns the last sequence number received from the remote host (+1)
    pub fn remote_sequence_num(&self) -> SequenceNumber {
        self.received_packets.sequence_num()
    }

    /// Returns the ack_bitfield corresponding to which of the past 32 packets we've
    /// successfully received.
    pub fn ack_bitfield(&self) -> u32 {
        let most_recent_remote_seq_num: u16 = self.received_packets.sequence_num().wrapping_sub(1);
        let mut ack_bitfield: u32 = 0;
        let mut mask: u32 = 1;

        // Iterate the past REDUNDANT_PACKET_ACKS_SIZE received packets and set the corresponding
        // bit for each packet which exists in the buffer.
        for i in 0..REDUNDANT_PACKET_ACKS_SIZE {
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
        self.received_packets.insert(remote_seq_num, ReceivedPacket {});

        // The current remote_ack_seq was (clearly) received so we should remove it.
        self.sent_packets.remove(&remote_ack_seq);

        // The remote_ack_field is going to include whether or not the past 32 packets have been
        // received successfully. If so, we have no need to resend old packets.
        for i in 1..REDUNDANT_PACKET_ACKS_SIZE + 1 {
            let ack_sequence = remote_ack_seq.wrapping_sub(i);
            if remote_ack_field & 1 == 1 {
                self.sent_packets.remove(&ack_sequence);
            }
            remote_ack_field >>= 1;
        }

        // Finally, iterate the sent packets and push dropped_packets
        let sent_sequences: Vec<SequenceNumber> = self.sent_packets.keys().map(|s| *s).collect();
        sent_sequences.into_iter()
            .filter(|s| remote_ack_seq.wrapping_sub(*s) > REDUNDANT_PACKET_ACKS_SIZE)
            .for_each(|s| {
                if let Some(dropped) = self.sent_packets.remove(&s) {
                    self.dropped_packets.push(dropped);
                }
            });
    }

    /// Enqueue the outgoing packet for acknowledgement.
    pub fn process_outgoing(&mut self, payload: &[u8], ordering_guarantee: OrderingGuarantee) {
        self.sent_packets.insert(
            self.sequence_number,
            SentPacket {
                payload: Box::from(payload),
                ordering_guarantee,
            },
        );

        // Bump the local sequence number for the next outgoing packet.
        self.sequence_number = self.sequence_number.wrapping_add(1);
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SentPacket {
    pub payload: Box<[u8]>,
    pub ordering_guarantee: OrderingGuarantee,
}

// TODO: At some point we should put something useful here.
#[derive(Clone, Default)]
pub struct ReceivedPacket;

#[cfg(test)]
mod test {
    use crate::infrastructure::{AcknowledgementHandler, SentPacket};
    use crate::packet::OrderingGuarantee;
    use log::debug;
    use crate::infrastructure::acknowledgement::ReceivedPacket;

    #[test]
    fn increment_local_seq_num_on_process_outgoing() {
        let mut handler = AcknowledgementHandler::new();
        for i in 0..10 {
            handler.process_outgoing(vec![].as_slice(), OrderingGuarantee::None);
            assert_eq!(handler.local_sequence_num(), i + 1);
        }
    }

    #[test]
    fn local_seq_num_wraps_on_overflow() {
        let mut handler = AcknowledgementHandler::new();
        let i = u16::max_value();
        handler.sequence_number = i;
        handler.process_outgoing(vec![].as_slice(), OrderingGuarantee::None);
        assert_eq!(handler.local_sequence_num(), 0);
    }

    #[test]
    fn ack_bitfield_with_empty_receive() {
        let handler = AcknowledgementHandler::new();
        assert_eq!(handler.ack_bitfield(), 0)
    }

    #[test]
    fn ack_bitfield_with_some_values() {
        let mut handler = AcknowledgementHandler::new();
        handler.received_packets.insert(0, ReceivedPacket::default());
        handler.received_packets.insert(1, ReceivedPacket::default());
        handler.received_packets.insert(3, ReceivedPacket::default());
        assert_eq!(handler.ack_bitfield(), 0b1101)
    }

    #[test]
    fn packet_is_not_acked() {
        let mut handler = AcknowledgementHandler::new();

        handler.sequence_number = 0;
        handler.process_outgoing(vec![1, 2, 3].as_slice(), OrderingGuarantee::None);
        handler.sequence_number = 40;
        handler.process_outgoing(vec![1, 2, 4].as_slice(), OrderingGuarantee::None);

        static ARBITRARY: u16 = 23;
        handler.process_incoming(ARBITRARY, 40, 0);

        assert_eq!(
            handler.dropped_packets,
            vec![SentPacket {
                payload: vec![1, 2, 3].into_boxed_slice(),
                ordering_guarantee: OrderingGuarantee::None,
            }]
        );
    }

    #[test]
    fn acking_500_packets_without_packet_drop() {
        let mut handler = AcknowledgementHandler::new();
        let mut other = AcknowledgementHandler::new();

        for i in 0..500 {
            handler.sequence_number = i;
            handler.process_outgoing(vec![1, 2, 3].as_slice(), OrderingGuarantee::None);

            other.process_incoming(i, handler.remote_sequence_num(), handler.ack_bitfield());
            handler.process_incoming(i, other.remote_sequence_num(), other.ack_bitfield());
        }

        assert_eq!(handler.dropped_packets.len(), 0);
    }

    #[test]
    fn acking_many_packets_with_packet_drop() {
        let mut handler = AcknowledgementHandler::new();
        let mut other = AcknowledgementHandler::new();

        let mut drop_count = 0;

        for i in 0..100 {
            handler.process_outgoing(vec![1, 2, 3].as_slice(), OrderingGuarantee::None);
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

        // TODO: Is this what we want?
//        assert_eq!(handler.dropped_packets.len(), 25);
        assert_eq!(handler.dropped_packets.len(), 17);
    }

    #[test]
    fn remote_seq_num_will_be_updated() {
        let mut handler = AcknowledgementHandler::new();
        assert_eq!(handler.remote_sequence_num(), 0);
        handler.process_incoming(0, 0, 0);
        assert_eq!(handler.remote_sequence_num(), 1);
        handler.process_incoming(1, 0, 0);
        assert_eq!(handler.remote_sequence_num(), 2);
    }
}