use std::{default::Default, time::Duration};

use crate::net::constants::{DEFAULT_MTU, FRAGMENT_SIZE_DEFAULT, MAX_FRAGMENTS_DEFAULT};

#[derive(Clone, Debug)]
/// Contains the configuration options to configure laminar for special use-cases.
pub struct Config {
    /// Make the underlying UDP socket block when true, otherwise non-blocking.
    pub blocking_mode: bool,
    /// Value which can specify the amount of time that can pass without hearing from a client before considering them disconnected
    pub idle_connection_timeout: Duration,
    /// Value which specifies at which interval (if at all) a heartbeat should be sent, if no other packet was sent in the meantime.
    /// If None, no heartbeats will be sent (the default).
    pub heartbeat_interval: Option<Duration>,
    /// Value which can specify the maximum size a packet can be in bytes. This value is inclusive of fragmenting; if a packet is fragmented, the total size of the fragments cannot exceed this value.
    ///
    /// Recommended value: 16384
    pub max_packet_size: usize,
    /// Value which can specify the maximal allowed fragments.
    ///
    /// Why can't I have more than 255 (u8)?
    /// This is because you don't want to send more then 256 fragments over UDP, with high amounts of fragments the chance for an invalid packet is very high.
    /// Use TCP instead (later we will probably support larger ranges but every fragment packet then needs to be resent if it doesn't get an acknowledgment).
    ///
    /// default: 16 but keep in mind that lower is better.
    pub max_fragments: u8,
    /// Value which can specify the size of a fragment.
    ///
    /// This is the maximum size of each fragment. It defaults to `1450` bytes, due to the default MTU on most network devices being `1500`.
    pub fragment_size: u16,
    /// Value which can specify the size of the buffer that queues up fragments ready to be reassembled once all fragments have arrived.```
    pub fragment_reassembly_buffer_size: u16,
    /// Value that specifies the size of the buffer the UDP data will be read into. Defaults to `1450` bytes.
    pub receive_buffer_max_size: usize,
    /// Value which can specify the factor which will smooth out network jitter.
    ///
    /// use-case: If one packet hast not arrived we don't directly want to transform to a bad network state.
    /// Value that specifies the factor used to smooth out network jitter. It defaults to 10% of the round-trip time. It is expressed as a ratio, with 0 equal to 0% and 1 equal to 100%. This helps prevent flapping of `VirtualConnections`
    pub rtt_smoothing_factor: f32,
    /// Value which can specify the maximal round trip time (rtt) for packet.
    ///
    /// Value which specifies the maximum round trip time before we consider it a problem. This is expressed in milliseconds.
    pub rtt_max_value: u16,
    /// Value which can specify the event buffer we read socket events into.
    ///
    /// Value that specifies the size of the event buffer into which we receive socket events, in bytes. Defaults to 1024.
    pub socket_event_buffer_size: usize,
    /// Value which can specify how long we should block polling for socket events.
    ///
    /// Value that specifies how long we should block polling for socket events, in milliseconds. Defaults to `1ms`.
    pub socket_polling_timeout: Option<Duration>,
    /// The maximum amount of reliable packets in flight on this connection before we drop the
    /// connection.
    ///
    /// When we send a reliable packet, it is stored locally until an acknowledgement comes back to
    /// us, if that store grows to a size
    pub max_packets_in_flight: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            blocking_mode: false,
            idle_connection_timeout: Duration::from_secs(5),
            heartbeat_interval: None,
            max_packet_size: (MAX_FRAGMENTS_DEFAULT * FRAGMENT_SIZE_DEFAULT) as usize,
            max_fragments: MAX_FRAGMENTS_DEFAULT as u8,
            fragment_size: FRAGMENT_SIZE_DEFAULT,
            fragment_reassembly_buffer_size: 64,
            receive_buffer_max_size: DEFAULT_MTU as usize,
            rtt_smoothing_factor: 0.10,
            rtt_max_value: 250,
            socket_event_buffer_size: 1024,
            socket_polling_timeout: Some(Duration::from_millis(1)),
            max_packets_in_flight: 512,
        }
    }
}
