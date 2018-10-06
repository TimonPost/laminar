use std::default::Default;
use net::constants::{FRAGMENT_SIZE_DEFAULT, MAX_FRAGMENTS_DEFAULT,};


#[derive(Clone)]
pub struct NetworkConfig
{
    /// This is the maximal size an packet can get with all its fragments.
    ///
    /// Recommended value: 16384
    pub max_packet_size: usize,
    /// These are the maximal fragments a packet could be divided into.
    ///
    /// Why can't I have more than 255 (u8)?
    /// This is because you don't want to send more then 256 fragments over UDP, with high amounts of fragments the change for an invalid packet is very high.
    /// Use TCP instead (later we will probably support larger ranges but every fragment packet then needs to be resent if it doesn't get an acknowledgement).
    ///
    /// Recommended value: 16 but keep in mind to keep this as low as possible.
    pub max_fragments: u8,
    /// This is the size of an fragment.
    /// If an packet is to large in needs to be spit in fragments.
    ///
    /// Recommended value: +- 1450 (1500 is the default MTU)
    pub fragment_size: u16,
    /// This is the size of the buffer that queues up fragments ready to be reassembled once the whole packet arrives.
    pub fragment_reassembly_buffer_size: usize,
    /// This is the size of the buffer the UDP socket reads it data into.
    pub receive_buffer_max_size: usize,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            max_packet_size: (MAX_FRAGMENTS_DEFAULT * FRAGMENT_SIZE_DEFAULT) as usize,
            max_fragments: MAX_FRAGMENTS_DEFAULT as u8,
            fragment_size: FRAGMENT_SIZE_DEFAULT,
            fragment_reassembly_buffer_size: 64,
            receive_buffer_max_size: 1500,
        }
    }
}