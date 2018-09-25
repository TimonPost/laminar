use std::net::SocketAddr;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Packet {
    addr: SocketAddr,
    seq: Option<u16>,
    payload: Box<[u8]>,
}

impl Packet {
    pub fn new(addr: SocketAddr, payload: Vec<u8>) -> Self {
        Packet {
            addr,
            seq: None,
            payload: payload.into_boxed_slice(),
        }
    }

    pub fn with_seq(mut self, seq: u16) -> Self {
        self.seq = Some(seq);
        self
    }

    pub fn addr(&self) -> SocketAddr {
        self.addr
    }
}
