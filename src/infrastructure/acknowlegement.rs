use crate::net::ExternalAcks;
use crate::net::LocalAckRecord;

pub struct AcknowledgementHandler {
    // reliability control
    waiting_packets: LocalAckRecord,
    their_acks: ExternalAcks,
    pub(crate) seq_num: u16,
    pub(crate) dropped_packets: Vec<Box<[u8]>>,
}

impl AcknowledgementHandler {
    pub fn new() -> AcknowledgementHandler {
        AcknowledgementHandler {
            // reliability control
            seq_num: 0,
            waiting_packets: Default::default(),
            their_acks: Default::default(),
            dropped_packets: Vec::new(),
        }
    }
}

impl AcknowledgementHandler {
    pub fn bit_mask(&self) -> u32 {
        self.their_acks.field
    }

    pub fn last_seq(&self) -> u16 {
        self.their_acks.last_seq
    }

    pub fn incoming(&mut self, incoming_seq: u16) {
        self.their_acks.ack(incoming_seq);

        // Update dropped packets if there are any.
        let dropped_packets = self.waiting_packets.ack(incoming_seq, self.bit_mask());

        self.dropped_packets = dropped_packets.into_iter().map(|(_, p)| p).collect();
    }

    pub fn outgoing(&mut self, payload: &[u8]) {
        self.waiting_packets.enqueue(self.seq_num, &payload);
    }
}
