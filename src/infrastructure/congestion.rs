use crate::net::{NetworkQuality, RttMeasurer};
use crate::sequence_buffer::{CongestionData, SequenceBuffer};
use crate::Config;
use std::time::Instant;

pub struct CongestionHandler {
    rtt_measurer: RttMeasurer,
    rtt: f32,
    congestion_data: SequenceBuffer<CongestionData>,
    _quality: NetworkQuality,
}

impl CongestionHandler {
    pub fn new(config: &Config) -> CongestionHandler {
        CongestionHandler {
            rtt_measurer: RttMeasurer::new(config),
            congestion_data: SequenceBuffer::with_capacity(<u16>::max_value() as usize),
            _quality: NetworkQuality::Good,
            rtt: 0.0,
        }
    }

    pub fn incoming(&mut self, incoming_seq: u16) {
        let congestion_data = self.congestion_data.get_mut(incoming_seq);
        self.rtt = self.rtt_measurer.get_rtt(congestion_data);
    }

    pub fn outgoing(&mut self, seq: u16) {
        self.congestion_data
            .insert(CongestionData::new(seq, Instant::now()), seq);
    }
}
