/// Third party's ack information.
///
/// So what does this mean?
///
/// Here we store information about the other side (virtual connection).
/// Like witch is the last sequence number from them.
#[derive(Debug, Default)]
pub struct ExternalAcks {
    /// the last sequence number we have received from the other side.
    pub last_seq: u16,
    pub field: u32,
    initialized: bool,
}

impl ExternalAcks {
    pub fn ack(&mut self, seq_num: u16) {
        if !self.initialized {
            self.last_seq = seq_num;
            self.initialized = true;
            return;
        }

        let pos_diff = seq_num.wrapping_sub(self.last_seq);
        let neg_diff = self.last_seq.wrapping_sub(seq_num);

        if pos_diff == 0 {
            return;
        }

        if pos_diff < 32000 {
            if pos_diff <= 32 {
                self.field = ((self.field << 1) | 1) << (pos_diff - 1);
            } else {
                self.field = 0;
            }
            self.last_seq = seq_num;
        } else if neg_diff <= 32 {
            self.field |= 1 << neg_diff - 1;
        }
    }
}

#[cfg(test)]
mod test {
    use super::ExternalAcks;

    #[test]
    fn acking_single_packet() {
        let mut acks:ExternalAcks = Default::default();
        acks.ack(0);

        assert_eq!(acks.last_seq, 0);
        assert_eq!(acks.field, 0);
    }

    #[test]
    fn acking_several_packets() {
        let mut acks:ExternalAcks = Default::default();
        acks.ack(0);
        acks.ack(1);
        acks.ack(2);

        assert_eq!(acks.last_seq, 2);
        assert_eq!(acks.field, 1 | (1 << 1));
    }

    #[test]
    fn acking_several_packets_out_of_order() {
        let mut acks:ExternalAcks = Default::default();
        acks.ack(1);
        acks.ack(0);
        acks.ack(2);

        assert_eq!(acks.last_seq, 2);
        assert_eq!(acks.field, 1 | (1 << 1));
    }

    #[test]
    fn acking_a_nearly_full_set_of_packets() {
        let mut acks:ExternalAcks = Default::default();

        for i in 0..32 {
            acks.ack(i);
        }

        assert_eq!(acks.last_seq, 31);
        assert_eq!(acks.field, !0 >> 1);
    }

    #[test]
    fn acking_a_full_set_of_packets() {
        let mut acks:ExternalAcks = Default::default();

        for i in 0..33 {
            acks.ack(i);
        }

        assert_eq!(acks.last_seq, 32);
        assert_eq!(acks.field, !0);
    }

    #[test]
    fn acking_to_the_edge_forward() {
        let mut acks:ExternalAcks = Default::default();
        acks.ack(0);
        acks.ack(32);

        assert_eq!(acks.last_seq, 32);
        assert_eq!(acks.field, 1 << 31);
    }

    #[test]
    fn acking_too_far_forward() {
        let mut acks:ExternalAcks = Default::default();
        acks.ack(0);
        acks.ack(1);
        acks.ack(34);

        assert_eq!(acks.last_seq, 34);
        assert_eq!(acks.field, 0);
    }

    #[test]
    fn acking_a_whole_buffer_too_far_forward() {
        let mut acks:ExternalAcks = Default::default();
        acks.ack(0);
        acks.ack(60);

        assert_eq!(acks.last_seq, 60);
        assert_eq!(acks.field, 0);
    }

    #[test]
    fn acking_too_far_backward() {
        let mut acks:ExternalAcks = Default::default();
        acks.ack(33);
        acks.ack(0);

        assert_eq!(acks.last_seq, 33);
        assert_eq!(acks.field, 0);
    }

    #[test]
    fn acking_around_zero() {
        let mut acks:ExternalAcks = Default::default();

        for i in 0..33_u16 {
            acks.ack(i.wrapping_sub(16));
        }
        assert_eq!(acks.last_seq, 16);
        assert_eq!(acks.field, !0);
    }

    #[test]
    fn ignores_old_packets() {
        let mut acks:ExternalAcks = Default::default();
        acks.ack(40);
        acks.ack(0);
        assert_eq!(acks.last_seq, 40);
        assert_eq!(acks.field, 0);
    }

    #[test]
    fn ignores_really_old_packets() {
        let mut acks:ExternalAcks = Default::default();
        acks.ack(30000);
        acks.ack(0);
        assert_eq!(acks.last_seq, 30000);
        assert_eq!(acks.field, 0);
    }

    #[test]
    fn skips_missing_acks_correctly() {
        let mut acks:ExternalAcks = Default::default();
        acks.ack(0);
        acks.ack(1);
        acks.ack(6);
        acks.ack(4);
        assert_eq!(acks.last_seq, 6);
        assert_eq!(
            acks.field,
            0        | // 5 (missing)
                       (1 << 1) | // 4 (present)
                       (0 << 2) | // 3 (missing)
                       (0 << 3) | // 2 (missing)
                       (1 << 4) | // 1 (present)
                       (1 << 5) // 0 (present)
        );
    }
}
