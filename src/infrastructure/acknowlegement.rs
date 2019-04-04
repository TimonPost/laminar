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

        self.dropped_packets = dropped_packets.into_iter().map(|(_, p)| p).collect();
    }

    /// Enqueue the outgoing packet for acknowledgement.
    pub fn process_outgoing(&mut self, payload: &[u8]) {
        self.waiting_packets.enqueue(self.seq_num, &payload);
    }
}
