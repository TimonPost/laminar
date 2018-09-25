use std::net::SocketAddr;
use net::Connection;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Packet {
    // the address to witch the packet will be send
    pub addr: SocketAddr,
    // the raw payload of the packet
    pub payload: Box<[u8]>,
}

impl Packet {
    pub fn new(addr: SocketAddr, payload: Vec<u8>) -> Self {
        Packet {
            addr,
            payload: payload.into_boxed_slice(),
        }
    }

    pub fn payload(&self) -> &[u8]
    {
        return &self.payload
    }

    pub fn addr(&self) -> SocketAddr {
        self.addr
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
/// packet that will be send over the network witch contains:
/// 1. the sequence number
/// 2. the last acknowledged sequence number
/// 3. last 32 acknowledged packages.
pub struct RawPacket {
    // this is the sequence number so that we can know where in the sequence of packages this packet belongs.
    pub seq: u16,
    // this is the last acknowledged sequence number.
    pub ack_seq: u16,
    // this is an bitfield of all last 32 acknowledged packages
    pub ack_field: u32,
    // this is the payload in witch the packet data is stored.
    pub payload: Box<[u8]>
}

impl RawPacket
{
    pub fn new( seq_num: u16, p: &Packet, connection: &Connection) -> RawPacket {
        RawPacket {
            seq: seq_num,
            ack_seq: connection.their_acks.last_seq,
            ack_field: connection.their_acks.field,
            payload: p.payload.clone()
        }
    }
}
