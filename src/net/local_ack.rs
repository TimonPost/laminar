use std::collections::HashMap;
use Packet;

/// Packets waiting for an ack
///
/// Holds up to 32 packets waiting for ack
///
/// Additionally, holds packets "forward" of the current ack packet
#[derive(Debug, Default)]
pub struct LocalAckRecord {
    // packets waiting for acknowledgement.
    packets: HashMap<u16, Packet>,
}

impl LocalAckRecord {
    /// Checks if there are packets in the queue to be aknowleged.
    pub fn is_empty(&mut self) -> bool {
        self.packets.is_empty()
    }

    /// Gets the total packages in the queue that could be aknowleged.
    pub fn len(&mut self) -> usize {
        self.packets.len()
    }

    /// Adds a packet to the queue awaiting for an aknowlegement.
    pub fn enqueue(&mut self, seq: u16, packet: Packet) {
        // TODO: Handle overwriting other packet?
        //   That really shouldn't happen, but it should be encoded here
        self.packets.insert(seq, packet);
    }

    /// Finds and removes acked packets, returning dropped packets
    #[allow(unused_parens)]
    pub fn ack(&mut self, seq: u16, seq_field: u32) -> Vec<(u16, Packet)> {
        let mut dropped_packets = Vec::new();
        let mut acked_packets = Vec::new();

        for key in self.packets.keys().into_iter() {
            let diff = seq.wrapping_sub(*key);
            if diff == 0 {
                acked_packets.push(*key);
            } else if diff <= 32 {
                let field_acked = (seq_field & (1 << diff - 1) != 0);
                if field_acked {
                    acked_packets.push(*key);
                }
            } else if diff < 32000 {
                dropped_packets.push(*key);
            }
        }

        for seq_number in acked_packets.iter() {
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
    use super::super::{LocalAckRecord, Packet};
    use std::net::{IpAddr, SocketAddr};
    use std::str::FromStr;

    #[test]
    fn acking_single_packet() {
        let mut record:LocalAckRecord = Default::default();
        record.enqueue(0, dummy_packet());
        let dropped = record.ack(0, 0);
        assert_eq!(dropped.len(), 0);
        assert!(record.is_empty());
    }

    #[test]
    fn acking_several_packets() {
        let mut record:LocalAckRecord = Default::default();
        record.enqueue(0, dummy_packet());
        record.enqueue(1, dummy_packet());
        record.enqueue(2, dummy_packet());
        let dropped = record.ack(2, 1 | (1 << 1));
        assert_eq!(dropped.len(), 0);
        assert!(record.is_empty());
    }

    #[test]
    fn acking_a_full_set_of_packets() {
        let mut record:LocalAckRecord = Default::default();

        for i in 0..33 {
            record.enqueue(i, dummy_packet())
        }

        let dropped = record.ack(32, !0);

        assert_eq!(dropped.len(), 0);
        assert!(record.is_empty());
    }

    #[test]
    fn dropping_one_packet() {
        let mut record:LocalAckRecord = Default::default();

        for i in 0..33 {
            record.enqueue(i, dummy_packet());
        }

        let dropped = record.ack(33, !0);

        assert_eq!(dropped, vec![(0, dummy_packet())]);
        assert!(record.is_empty());
    }

    #[test]
    fn acking_around_zero() {
        let mut record:LocalAckRecord = Default::default();

        for i in 0..33_u16 {
            record.enqueue(i.wrapping_sub(16), dummy_packet());
        }

        let dropped = record.ack(16, !0);

        assert_eq!(dropped.len(), 0);
        assert!(record.is_empty());
    }

    #[test]
    fn not_dropping_new_packets() {
        let mut record:LocalAckRecord = Default::default();
        record.enqueue(0, dummy_packet());
        record.enqueue(1, dummy_packet());
        record.enqueue(2, dummy_packet());
        record.enqueue(5, dummy_packet());
        record.enqueue(30000, dummy_packet());
        let dropped = record.ack(1, 1);
        assert_eq!(dropped.len(), 0);
        assert_eq!(record.len(), 3);
    }

    #[test]
    fn drops_old_packets() {
        let mut record:LocalAckRecord = Default::default();
        record.enqueue(0, dummy_packet());
        record.enqueue(40, dummy_packet());
        let dropped = record.ack(40, 0);
        assert_eq!(dropped, vec![(0, dummy_packet())]);
        assert!(record.is_empty());
    }

    #[test]
    fn drops_really_old_packets() {
        let mut record:LocalAckRecord = Default::default();
        record.enqueue(50000, dummy_packet());
        record.enqueue(0, dummy_packet());
        record.enqueue(1, dummy_packet());
        let dropped = record.ack(1, 1);
        assert_eq!(dropped, vec![(50000, dummy_packet())]);
        assert!(record.is_empty());
    }

    pub fn dummy_packet() -> Packet {
        let addr = SocketAddr::new(
            IpAddr::from_str("0.0.0.0").expect("Unreadable input IP."),
            12345,
        );

        Packet::new(addr, Vec::new())
    }
}
