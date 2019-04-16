use crate::infrastructure::{ExternalAcks, LocalAckRecord};

/// Type responsible for handling the acknowledgement of packets.
pub struct AcknowledgementHandler {
    waiting_packets: LocalAckRecord,
    their_acks: ExternalAcks,
    pub seq_num: u16,
    pub dropped_packets: Vec<Box<[u8]>>,
}

impl AcknowledgementHandler {
    /// Constructs a new `AcknowledgementHandler` with which you can perform acknowledgement operations.
    pub fn new() -> AcknowledgementHandler {
        AcknowledgementHandler {
            seq_num: 0,
            waiting_packets: Default::default(),
            their_acks: Default::default(),
            dropped_packets: Vec::new(),
        }
    }
}

impl AcknowledgementHandler {
    /// Returns the bit mask that contains the packets who are acknowledged.
    pub fn bit_mask(&self) -> u32 {
        self.their_acks.field
    }

    /// Returns the last acknowledged sequence number by the other endpoint.
    pub fn last_seq(&self) -> u16 {
        self.their_acks.last_seq
    }

    /// Process the incoming sequence number.
    ///
    /// - Acknowledge the incoming sequence number
    /// - Update dropped packets
    pub fn process_incoming(&mut self, incoming_seq: u16) {
        self.their_acks.ack(incoming_seq);

        let dropped_packets = self.waiting_packets.ack(incoming_seq, self.bit_mask());
        self.dropped_packets
            .extend(dropped_packets.into_iter().map(|(_, p)| p));
    }

    /// Enqueue the outgoing packet for acknowledgement.
    pub fn process_outgoing(&mut self, payload: &[u8]) {
        println!("Dropped packets are: {:#?}", self.dropped_packets);
        self.waiting_packets.enqueue(self.seq_num, &payload);
    }
}

#[cfg(test)]
mod test {
    use crate::infrastructure::AcknowledgementHandler;

    #[test]
    fn packet_is_not_acket() {
        let mut handler = AcknowledgementHandler::new();

        handler.seq_num = 0;
        handler.process_outgoing(vec![1, 2, 3].as_slice());
        handler.seq_num = 40;
        handler.process_outgoing(vec![1, 2, 4].as_slice());

        handler.process_incoming(40);

        assert_eq!(
            handler.dropped_packets,
            vec![vec![1, 2, 3].into_boxed_slice()]
        );
    }

    #[test]
    fn acking_500_packets_without_packet_drop() {
        let mut handler = AcknowledgementHandler::new();

        for i in 0..500 {
            handler.seq_num = i;
            handler.process_outgoing(vec![1, 2, 3].as_slice());

            handler.process_incoming(i);
        }

        assert_eq!(handler.dropped_packets.len(), 0);
    }

    #[test]
    fn acking_500_packets_with_packet_drop() {
        let mut handler = AcknowledgementHandler::new();

        let mut count = 0;

        for i in 0..500 {
            handler.seq_num = i;
            handler.process_outgoing(vec![i as u8, 2, 3].as_slice());

            if i % 4 != 0 {
                handler.process_incoming(i);
            } else {
                count += 1;
            }
        }

        assert_eq!(handler.dropped_packets.len(), count);
    }

    #[test]
    fn last_seq_will_be_updated() {
        let mut handler = AcknowledgementHandler::new();
        assert_eq!(handler.last_seq(), 0);
        handler.process_incoming(1);
        assert_eq!(handler.last_seq(), 1);
        handler.process_incoming(2);
        assert_eq!(handler.last_seq(), 2);
    }
}
