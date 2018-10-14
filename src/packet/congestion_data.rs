use std::time::Instant;

#[derive(Clone)]
/// This contains the information needed to know for reassembling fragments.
pub struct CongestionData {
    pub sequence: u16,
    pub sending_time: Instant,
}

impl CongestionData {
    pub fn new(sequence: u16, sending_time: Instant) -> Self {
        CongestionData {
            sequence,
            sending_time
        }
    }
}

impl Default for CongestionData
{
    fn default() -> Self {
        CongestionData { sequence: 0, sending_time: Instant::now() }
    }
}