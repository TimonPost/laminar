use std::time::Instant;

use crate::packet::SequenceNumber;

#[derive(Clone)]
/// This contains the information required to reassemble fragments.
pub struct CongestionData {
    pub sequence: SequenceNumber,
    pub sending_time: Instant,
}

impl CongestionData {
    pub fn new(sequence: SequenceNumber, sending_time: Instant) -> Self {
        CongestionData {
            sequence,
            sending_time,
        }
    }
}

impl Default for CongestionData {
    fn default() -> Self {
        CongestionData {
            sequence: 0,
            sending_time: Instant::now(),
        }
    }
}
