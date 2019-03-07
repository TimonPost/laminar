use crate::{
    net::{NetworkQuality, RttMeasurer},
    sequence_buffer::{CongestionData, SequenceBuffer},
    Config,
};

use std::time::Instant;

/// Type that is responsible for keeping track of congestion information.
pub struct CongestionHandler {
    rtt_measurer: RttMeasurer,
    congestion_data: SequenceBuffer<CongestionData>,
    _quality: NetworkQuality,
}

impl CongestionHandler {
    /// Constructs a new `CongestionHandler` which you can use for keeping track of congestion information.
    pub fn new(config: &Config) -> CongestionHandler {
        CongestionHandler {
            rtt_measurer: RttMeasurer::new(config),
            congestion_data: SequenceBuffer::with_capacity(<u16>::max_value() as usize),
            _quality: NetworkQuality::Good,
        }
    }

    /// Process incoming sequence number.
    ///
    /// This will calculate the RTT-time and smooth down the RTT-value to prevent uge RTT-spikes.
    pub fn process_incoming(&mut self, incoming_seq: u16) {
        let congestion_data = self.congestion_data.get_mut(incoming_seq);
        self.rtt_measurer.calculate_rrt(congestion_data);
    }

    /// Process outgoing sequence number.
    ///
    /// This will insert an entry which is used for keeping track of the sending time.
    /// Once we process incoming sequence numbers we can calculate the `RTT` time.
    pub fn process_outgoing(&mut self, seq: u16) {
        self.congestion_data
            .insert(CongestionData::new(seq, Instant::now()), seq);
    }
}
