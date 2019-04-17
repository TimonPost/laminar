use crate::infrastructure::{ExternalAcks, LocalAckRecord};

/// Type responsible for handling the acknowledgement of packets.
pub struct AcknowledgementHandler {
    waiting_packets: LocalAckRecord,
    acks_of_received: ExternalAcks,
    pub seq_num: u16,
    pub dropped_packets: Vec<Box<[u8]>>,
}

impl AcknowledgementHandler {
    /// Constructs a new `AcknowledgementHandler` with which you can perform acknowledgement operations.
    pub fn new() -> AcknowledgementHandler {
        AcknowledgementHandler {
            seq_num: 0,
            waiting_packets: Default::default(),
            acks_of_received: Default::default(),
            dropped_packets: Vec::new(),
        }
    }
}

impl AcknowledgementHandler {
    /// Returns the bit mask that contains the packets that we have receieved
    pub fn bit_mask(&self) -> u32 {
        self.acks_of_received.field
    }

    /// Returns the last sequence number in a packet we've received
    pub fn last_seq(&self) -> u16 {
        self.acks_of_received.last_seq
    }

    /// Process the incoming sequence number.
    ///
    /// - Acknowledge the incoming sequence number
    /// - Update dropped packets
    pub fn process_incoming(&mut self, new_packet_seq: u16, ack_seq: u16, ack_field: u32) {
        self.acks_of_received.ack(new_packet_seq);

        let dropped_packets = self.waiting_packets.ack(ack_seq, ack_field);
        self.dropped_packets
            .extend(dropped_packets.into_iter().map(|(_, p)| p));
    }

    /// Enqueue the outgoing packet for acknowledgement.
    pub fn process_outgoing(&mut self, payload: &[u8]) {
        self.waiting_packets.enqueue(self.seq_num, &payload);
    }
}

#[cfg(test)]
mod test {
    use crate::infrastructure::AcknowledgementHandler;
    use log::debug;

    #[test]
    fn packet_is_not_acket() {
        let mut handler = AcknowledgementHandler::new();

        handler.seq_num = 0;
        handler.process_outgoing(vec![1, 2, 3].as_slice());
        handler.seq_num = 40;
        handler.process_outgoing(vec![1, 2, 4].as_slice());

        static ARBITRARY: u16 = 23;
        handler.process_incoming(ARBITRARY, 40, 0);

        assert_eq!(
            handler.dropped_packets,
            vec![vec![1, 2, 3].into_boxed_slice()]
        );
    }

    #[test]
    fn acking_500_packets_without_packet_drop() {
        let mut handler = AcknowledgementHandler::new();
        let mut other = AcknowledgementHandler::new();

        for i in 0..500 {
            handler.seq_num = i;
            handler.process_outgoing(vec![1, 2, 3].as_slice());

            other.process_incoming(i, handler.last_seq(), handler.bit_mask());
            handler.process_incoming(i, other.last_seq(), other.bit_mask());
        }

        assert_eq!(handler.dropped_packets.len(), 0);
    }

    #[test]
    fn acking_2000_packets_with_packet_drop() {
        let mut handler = AcknowledgementHandler::new();
        let mut other = AcknowledgementHandler::new();

        let mut drop_count = 0;

        for i in 0..2000 {
            handler.process_outgoing(vec![1, 2, 3].as_slice());
            handler.seq_num = i;

            // dropping every 4th with modulo's
            if i % 4 == 0 {
                debug!("Dropping packet: {}", drop_count);
                drop_count += 1;
            } else {
                // We send them a packet
                other.process_incoming(i, handler.last_seq(), handler.bit_mask());
                // Skipped: other.process_outgoing
                // And it makes it back
                handler.process_incoming(i, other.last_seq(), other.bit_mask());
            }
        }

        assert_eq!(handler.dropped_packets.len(), 500);
    }

    #[test]
    fn last_seq_will_be_updated() {
        let mut handler = AcknowledgementHandler::new();
        assert_eq!(handler.last_seq(), 0);
        handler.process_incoming(1, 0, 0);
        assert_eq!(handler.last_seq(), 1);
        handler.process_incoming(2, 0, 0);
        assert_eq!(handler.last_seq(), 2);
    }
}
