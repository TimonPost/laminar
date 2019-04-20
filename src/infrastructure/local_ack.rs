use crate::infrastructure::WaitingPacket;
use std::collections::HashMap;

/// Packets waiting for an ack
///
/// Holds up to 32 packets waiting for ack
///
/// Additionally, holds packets "forward" of the current ack packet
#[derive(Debug, Default)]
pub struct LocalAckRecord {
    // packets waiting for acknowledgement.
    packets: HashMap<u16, WaitingPacket>,
}

impl LocalAckRecord {
    /// Checks if there are packets in the queue to be acknowledged.
    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.packets.is_empty()
    }

    /// Gets the total packages in the queue that could be acknowledged.
    #[cfg(test)]
    pub fn len(&mut self) -> usize {
        self.packets.len()
    }

    /// Adds a packet to the queue awaiting for an acknowledgement.
    pub fn enqueue(&mut self, seq: u16, packet: WaitingPacket) {
        self.packets.insert(seq, packet);
    }

    /// Finds and removes acked packets, returning dropped packets
    #[allow(unused_parens)]
    pub fn ack(&mut self, seq: u16, seq_field: u32) -> Vec<(u16, WaitingPacket)> {
        let mut dropped_packets = Vec::new();
        let mut acked_packets = Vec::new();

        for key in self.packets.keys() {
            let diff = seq.wrapping_sub(*key);
            if diff == 0 {
                // This packet (ack)
                acked_packets.push(*key);
            } else if diff <= 32 {
                // seq > key, so it's old packet
                // within last 32 so checkable within bitfield
                let field_acked = (seq_field & (1 << (diff - 1)) != 0);
                if field_acked {
                    acked_packets.push(*key);
                } else {
                    dropped_packets.push(*key);
                }
            } else if diff < 32000 {
                // Old packet that's not within 32, so can't be acked
                // ASSUME DROPPED
                dropped_packets.push(*key);
            }
            // otherwise, wrapped around, so key > seq (new)
            // New packet (don't drop or ack)
        }

        for seq_number in &acked_packets {
            self.packets.remove(seq_number);
        }

        dropped_packets
            .into_iter()
            .map(|seq| (seq, self.packets.remove(&seq).unwrap()))
            .collect()
    }
}

#[cfg(test)]
mod test {
    use crate::infrastructure::{LocalAckRecord, WaitingPacket};
    use crate::packet::OrderingGuarantee;

    fn get_empty() -> WaitingPacket {
        WaitingPacket {
            payload: Box::new([]),
            ordering_guarantee: OrderingGuarantee::None,
        }
    }

    #[test]
    fn acking_single_packet() {
        let mut record: LocalAckRecord = Default::default();
        record.enqueue(0, get_empty());
        let dropped = record.ack(0, 0);
        assert_eq!(dropped.len(), 0);
        assert!(record.is_empty());
    }

    #[test]
    fn acking_several_packets() {
        let mut record: LocalAckRecord = Default::default();
        record.enqueue(0, get_empty());
        record.enqueue(1, get_empty());
        record.enqueue(2, get_empty());
        let dropped = record.ack(2, 1 | (1 << 1));
        assert_eq!(dropped.len(), 0);
        assert!(record.is_empty());
    }

    #[test]
    fn acking_a_full_set_of_packets() {
        let mut record: LocalAckRecord = Default::default();

        for i in 0..33 {
            record.enqueue(i, get_empty())
        }

        let dropped = record.ack(32, !0);

        assert_eq!(dropped.len(), 0);
        assert!(record.is_empty());
    }

    #[test]
    fn dropping_one_packet() {
        let mut record: LocalAckRecord = Default::default();

        for i in 0..33 {
            record.enqueue(i, get_empty());
        }

        let dropped = record.ack(33, !0);

        assert_eq!(dropped, vec![(0, get_empty())]);
        assert!(record.is_empty());
    }

    #[test]
    fn acking_around_zero() {
        let mut record: LocalAckRecord = Default::default();

        for i in 0..33_u16 {
            record.enqueue(i.wrapping_sub(16), get_empty());
        }

        let dropped = record.ack(16, !0);

        assert_eq!(dropped.len(), 0);
        assert_eq!(record.len(), 0);
    }

    #[test]
    fn not_dropping_new_packets() {
        let mut record: LocalAckRecord = Default::default();
        record.enqueue(0, get_empty());
        record.enqueue(1, get_empty());
        record.enqueue(2, get_empty());
        record.enqueue(5, get_empty());
        record.enqueue(30000, get_empty());
        let dropped = record.ack(1, 1);
        assert_eq!(dropped.len(), 0);
        assert_eq!(record.len(), 3);
    }

    #[test]
    fn drops_old_packets() {
        let mut record: LocalAckRecord = Default::default();
        record.enqueue(0, get_empty());
        record.enqueue(40, get_empty());
        let dropped = record.ack(40, 0);
        assert_eq!(dropped, vec![(0, get_empty())]);
        assert!(record.is_empty());
    }

    #[test]
    fn drops_really_old_packets() {
        let mut record: LocalAckRecord = Default::default();
        record.enqueue(50000, get_empty());
        record.enqueue(0, get_empty());
        record.enqueue(1, get_empty());
        let dropped = record.ack(1, 1);
        assert_eq!(dropped, vec![(50000, get_empty())]);
        assert!(record.is_empty());
    }
}
