use std::net::SocketAddr;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Packet {
    // the address to witch the packet will be send
    pub addr: SocketAddr,
    // this is the last acknowledged sequence number.
    pub ack_seq: Option<u16>,
    // this is an bitfield of all last 32 acknowledged packages
    pub ack_field: Option<u32>,
    // this is the payload in witch the packet data is stored.
    pub seq: Option<u16>,
    // the raw payload of the packet
    payload: Box<[u8]>,
}

impl Packet {
    pub fn new(addr: SocketAddr, payload: Vec<u8>) -> Self {
        Packet {
            addr,
            ack_seq: None,
            ack_field: None,
            seq: None,
            payload: payload.into_boxed_slice(),
        }
    }

    pub fn with_data(mut self, seq: u16, ack_seq: u16, ack_field: u32) -> Self
    {
        self.seq = Some(seq);
        self.ack_seq = Some(ack_seq);
        self.ack_field = Some(ack_field);
        self
    }

    pub fn addr(&self) -> SocketAddr {
        self.addr
    }
}
