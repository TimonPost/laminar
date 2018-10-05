use std::net::SocketAddr;

#[derive(Clone, PartialEq, Eq, Debug)]
/// This is a user friendly packet containing the payload from the packet and the enpoint from where it came.
pub struct Packet {
    // the address to witch the packet will be send
    addr: SocketAddr,
    // the raw payload of the packet
    payload: Box<[u8]>,
}

impl Packet {
    pub fn new(addr: SocketAddr, payload: Vec<u8>) -> Self {
        Packet {
            addr,
            payload: payload.into_boxed_slice(),
        }
    }

    /// Get the payload (raw data) of this packet.
    pub fn payload(&self) -> &[u8] {
        return &self.payload;
    }

    /// Get the endpoint from this packet.
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }
}