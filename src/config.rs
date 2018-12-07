use crate::net::constants::{FRAGMENT_SIZE_DEFAULT, MAX_FRAGMENTS_DEFAULT};
use std::default::Default;

#[derive(Clone)]
/// Struct that contains config values for various aspects of the network
pub struct NetworkConfig {
    /// This is the maximal size a packet can get with all its fragments.
    ///
    /// Recommended value: 16384
    pub max_packet_size: usize,
    /// These are the maximal fragments a packet could be divided into.
    ///
    /// Why can't I have more than 255 (u8)?
    /// This is because you don't want to send more then 256 fragments over UDP, with high amounts of fragments the chance for an invalid packet is very high.
    /// Use TCP instead (later we will probably support larger ranges but every fragment packet then needs to be resent if it doesn't get an acknowledgement).
    ///
    /// Recommended value: 16 but keep in mind that lower is better.
    pub max_fragments: u8,
    /// This is the size of a fragment.
    /// If a packet is too large it needs to be split in fragments.
    ///
    /// Recommended value: +- 1450 (1500 is the default MTU)
    pub fragment_size: u16,
    /// This is the size of the buffer that queues up fragments ready to be reassembled once the whole packet arrives.
    pub fragment_reassembly_buffer_size: usize,
    /// This is the size of the buffer the UDP socket reads it data into.
    pub receive_buffer_max_size: usize,
    /// This is the factor which will smooth out network jitter. So that if one packet is not arrived fast we don't wan't to directly transform to an bad network.
    ///
    /// Recommended value: 10% of the rtt time.
    /// Value is a ratio (0 = 0% and 1 = 100%)
    pub rtt_smoothing_factor: f32,
    /// This is the maximal round trip time (rtt) for packet.
    ///
    /// Recommend value: 250 ms
    /// Value is represented in milliseconds.
    pub rtt_max_value: u16,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            max_packet_size: (MAX_FRAGMENTS_DEFAULT * FRAGMENT_SIZE_DEFAULT) as usize,
            max_fragments: MAX_FRAGMENTS_DEFAULT as u8,
            fragment_size: FRAGMENT_SIZE_DEFAULT,
            fragment_reassembly_buffer_size: 64,
            receive_buffer_max_size: 1500,
            rtt_smoothing_factor: 0.10,
            rtt_max_value: 250,
        }
    }
}
