use crate::net::constants::{FRAGMENT_SIZE_DEFAULT, MAX_FRAGMENTS_DEFAULT};
use std::{default::Default, time::Duration};

#[derive(Clone)]
/// Struct that contains config values for various aspects of the network
pub struct Config {
    /// The maximal amount of time to keep `VirtualConnection`s around before cleaning them up.
    pub idle_connection_timeout: Duration,
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
    /// This is the size of the event buffer we read socket events from `mio::Poll` into.
    pub socket_event_buffer_size: usize,
    /// Optional duration specifying how long we should block polling for socket events.
    pub socket_polling_timeout: Option<Duration>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            idle_connection_timeout: Duration::from_secs(5),
            max_packet_size: (MAX_FRAGMENTS_DEFAULT * FRAGMENT_SIZE_DEFAULT) as usize,
            max_fragments: MAX_FRAGMENTS_DEFAULT as u8,
            fragment_size: FRAGMENT_SIZE_DEFAULT,
            fragment_reassembly_buffer_size: 64,
            receive_buffer_max_size: 1500,
            rtt_smoothing_factor: 0.10,
            rtt_max_value: 250,
            socket_event_buffer_size: 1024,
            socket_polling_timeout: Some(Duration::from_millis(100)),
        }
    }
}
