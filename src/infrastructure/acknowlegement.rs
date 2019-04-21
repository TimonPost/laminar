use crate::packet::OrderingGuarantee;
use crate::sequence_buffer::SequenceBuffer;

const REDUNDANT_PACKET_ACKS_SIZE: u16 = 32;

/// Responsible for handling the acknowledgement of packets.
pub struct AcknowledgementHandler {
    // Local sequence number which we'll bump each time we send a new packet over the network
    local_seq_num: u16,
    // Last received sequence number from the remote host
    remote_seq_num: u16,

    sent_packets: SequenceBuffer<WaitingPacket>,
    received_packets: SequenceBuffer<i32>,

    // TODO: Make sure this doesn't need to be public
    pub dropped_packets: Vec<WaitingPacket>,
}

impl AcknowledgementHandler {
    /// Constructs a new `AcknowledgementHandler` with which you can perform acknowledgement operations.
    pub fn new() -> AcknowledgementHandler {
        AcknowledgementHandler {
            local_seq_num: 0,
            remote_seq_num: 0,
            sent_packets: SequenceBuffer::with_capacity(REDUNDANT_PACKET_ACKS_SIZE),
            received_packets: SequenceBuffer::with_capacity(REDUNDANT_PACKET_ACKS_SIZE),
            dropped_packets: Vec::new(),
        }
    }
}

impl AcknowledgementHandler {
    /// Returns the last sequence number we've sent.
    pub fn local_seq_num(&self) -> u16 {
        self.local_seq_num
    }

    /// Returns the last sequence number of the remote host
    pub fn remote_seq_num(&self) -> u16 {
        self.remote_seq_num
    }

    /// Returns the ack_bitfield corresponding to which of the past 32 packets we've
    /// successfully received.
    pub fn ack_bitfield(&self) -> u32 {
        let most_recent_remote_seq_num: u16 = self.received_packets.sequence_num().wrapping_sub(1);
        let mut ack_bitfield: u32 = 0;
        let mut mask: u32 = 1;

        // Iterate the past REDUNDANT_PACKET_ACKS_SIZE received packets and set the corresponding
        // bit if they exist in the buffer.
        for i in 0..REDUNDANT_PACKET_ACKS_SIZE as u16 {
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
        remote_ack_field: u32,
    ) {
        //        self.acks_of_received.ack(new_packet_seq);
        //
        //        let dropped_packets = self.waiting_packets.ack(ack_seq, ack_field);
        //        self.dropped_packets
        //            .extend(dropped_packets.into_iter().map(|(_, p)| p));
    }

    /// Enqueue the outgoing packet for acknowledgement.
    pub fn process_outgoing(&mut self, payload: &[u8], ordering_guarantee: OrderingGuarantee) {
        self.sent_packets.insert(
            self.local_seq_num,
            WaitingPacket {
                payload: Box::from(payload),
                ordering_guarantee,
            },
        );

        // Bump the local sequence number for the next outgoing packet.
        self.local_seq_num = self.local_seq_num.wrapping_add(1);
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WaitingPacket {
    pub payload: Box<[u8]>,
    pub ordering_guarantee: OrderingGuarantee,
}

#[cfg(test)]
mod test {
    use crate::infrastructure::{AcknowledgementHandler, WaitingPacket};
    use crate::packet::OrderingGuarantee;
    use log::debug;

    #[test]
    fn increment_local_seq_num_on_process_outgoing() {
        let mut handler = AcknowledgementHandler::new();
        for i in 0..10 {
            handler.process_outgoing(vec![].as_slice(), OrderingGuarantee::None);
            assert_eq!(handler.local_seq_num(), i + 1);
        }
    }

    #[test]
    fn local_seq_num_wraps_on_overflow() {
        let mut handler = AcknowledgementHandler::new();
        let i = u16::max_value();
        handler.local_seq_num = i;
        handler.process_outgoing(vec![].as_slice(), OrderingGuarantee::None);
        assert_eq!(handler.local_seq_num(), 0);
    }

    #[test]
    fn ack_bitfield_with_empty_receive() {
        let handler = AcknowledgementHandler::new();
        assert_eq!(handler.ack_bitfield(), 0)
    }

    #[test]
    fn ack_bitfield_with_some_values() {
        let mut handler = AcknowledgementHandler::new();
        handler.received_packets.insert(0, 0);
        handler.received_packets.insert(1, 0);
        handler.received_packets.insert(3, 0);
        assert_eq!(handler.ack_bitfield(), 0b1101)
    }

    #[test]
    fn packet_is_not_acked() {
        let mut handler = AcknowledgementHandler::new();

        handler.local_seq_num = 0;
        handler.process_outgoing(vec![1, 2, 3].as_slice(), OrderingGuarantee::None);
        handler.local_seq_num = 40;
        handler.process_outgoing(vec![1, 2, 4].as_slice(), OrderingGuarantee::None);

        static ARBITRARY: u16 = 23;
        handler.process_incoming(ARBITRARY, 40, 0);

        assert_eq!(
            handler.dropped_packets,
            vec![WaitingPacket {
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
            handler.local_seq_num = i;
            handler.process_outgoing(vec![1, 2, 3].as_slice(), OrderingGuarantee::None);

            other.process_incoming(i, handler.remote_seq_num(), handler.ack_bitfield());
            handler.process_incoming(i, other.remote_seq_num(), other.ack_bitfield());
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
            handler.local_seq_num = i;

            // dropping every 4th with modulo's
            if i % 4 == 0 {
                debug!("Dropping packet: {}", drop_count);
                drop_count += 1;
            } else {
                // We send them a packet
                other.process_incoming(i, handler.remote_seq_num(), handler.ack_bitfield());
                // Skipped: other.process_outgoing
                // And it makes it back
                handler.process_incoming(i, other.remote_seq_num(), other.ack_bitfield());
            }
        }

        assert_eq!(handler.dropped_packets.len(), 25);
    }

    #[test]
    fn last_seq_will_be_updated() {
        let mut handler = AcknowledgementHandler::new();
        assert_eq!(handler.remote_seq_num(), 0);
        handler.process_incoming(1, 0, 0);
        assert_eq!(handler.remote_seq_num(), 1);
        handler.process_incoming(2, 0, 0);
        assert_eq!(handler.remote_seq_num(), 2);
    }
}
