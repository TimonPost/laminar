use net::{ExternalAcks, LocalAckRecord, NetworkQuality};
use packet::{CongestionData, FragmentBuffer, Packet};
use std::fmt;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

/// Contains the information about a certain 'virtual connection' over udp.
/// This stores information about the last sequence number, dropped packages, packages waiting for acknowledgement and acknowledgements gotten from the other side.
pub struct VirtualConnection {
    pub seq_num: u16,
    pub dropped_packets: Vec<Packet>,
    pub waiting_packets: LocalAckRecord,
    pub their_acks: ExternalAcks,
    pub last_heard: Instant,
    pub remote_address: SocketAddr,
    pub quality: NetworkQuality,
    pub congestion_avoidance_buffer: FragmentBuffer<CongestionData>,
    pub rtt: f32,
}

impl VirtualConnection {
    /// Creates and returns a new Connection that wraps the provided socket address
    pub fn new(addr: SocketAddr) -> VirtualConnection {
        VirtualConnection {
            seq_num: 0,
            dropped_packets: Vec::new(),
            waiting_packets: Default::default(),
            their_acks: Default::default(),
            last_heard: Instant::now(),
            quality: NetworkQuality::Good,
            remote_address: addr,
            congestion_avoidance_buffer: FragmentBuffer::with_capacity(<u16>::max_value() as usize),
            rtt: 0.0,
        }
    }

    /// Returns a Duration representing since we last heard from the client
    pub fn last_heard(&self) -> Duration {
        let now = Instant::now();
        now.duration_since(self.last_heard)
    }
}

impl fmt::Debug for VirtualConnection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}:{}",
            self.remote_address.ip(),
            self.remote_address.port()
        )
    }
}
