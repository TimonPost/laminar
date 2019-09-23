use crate::net::constants::MAX_FRAGMENTS_DEFAULT;
use crate::packet::header::AckedPacketHeader;
use crate::packet::SequenceNumber;

#[derive(Clone)]
/// This contains the information required to reassemble fragments.
pub struct ReassemblyData {
    pub sequence: SequenceNumber,
    pub num_fragments_received: u8,
    pub num_fragments_total: u8,
    pub buffer: Vec<u8>,
    pub fragments_received: [bool; MAX_FRAGMENTS_DEFAULT as usize],
    pub acked_header: Option<AckedPacketHeader>,
}

impl ReassemblyData {
    pub fn new(sequence: SequenceNumber, num_fragments_total: u8, prealloc: usize) -> Self {
        Self {
            sequence,
            num_fragments_received: 0,
            num_fragments_total,
            buffer: Vec::with_capacity(prealloc),
            fragments_received: [false; MAX_FRAGMENTS_DEFAULT as usize],
            acked_header: None,
        }
    }
}

impl Default for ReassemblyData {
    fn default() -> Self {
        Self {
            sequence: 0,
            num_fragments_received: 0,
            num_fragments_total: 0,
            buffer: Vec::with_capacity(1024),
            fragments_received: [false; MAX_FRAGMENTS_DEFAULT as usize],
            acked_header: None,
        }
    }
}
