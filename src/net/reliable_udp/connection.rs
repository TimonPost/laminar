use super::{AckRecord, ExternalAcks, Packet};

/// Contains the information about a certain 'virtual connection' over udp.
/// This stores information about the last sequence number, dropped packages, packages waiting for acknowledgement and acknowledgements gotten from the other side.
pub struct Connection
{
    pub seq_num: u16,
    pub dropped_packets: Vec<Packet>,
    pub waiting_packets: AckRecord,
    pub their_acks: ExternalAcks,
}

impl Connection {
    pub fn new() -> Connection {
        Connection {
            seq_num: 0,
            dropped_packets: Vec::new(),
            waiting_packets: AckRecord::new(),
            their_acks: ExternalAcks::new()
        }
    }
}

// TODO:
/// This defines whether the connection is good or bad.
/// We should use this for handling Congestion Avoidance so that when the network of the client is bad we do not flood the router with small packets.
///
/// When network conditions are `Good` we send 30 packets per-second, and when network conditions are `Bad` we drop to 10 packets per-second.
pub enum ConnectionQuality
{
    Good,
    Bad
}