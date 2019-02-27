use std::net::SocketAddr;

/// Represents a completed fragmented packet that has been reconstructed during the fragmentation process.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Fragment {
    payload: Box<[u8]>,
    addr: SocketAddr,
}

impl Fragment {
    /// Construct a new instance of `Fragment`.
    pub fn new(payload: Box<[u8]>, addr: SocketAddr) -> Fragment {
        Fragment { payload, addr }
    }

    /// Returns the payload of this fragmented packet.
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    /// Returns the endpoint from which the fragmented packet came from.
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }
}

#[cfg(test)]
mod tests {
    use crate::packet::Fragment;
    use std::net::SocketAddr;

    #[test]
    fn create_fragment() {
        let header = Fragment::new(test_payload(), test_addr());

        assert_eq!(header.addr(), test_addr());
        assert_eq!(header.payload(), &test_payload());
    }

    fn test_payload() -> Box<[u8]> {
        return "test".as_bytes().to_vec().into_boxed_slice();
    }

    fn test_addr() -> SocketAddr {
        "127.0.0.1:12345".parse().unwrap()
    }
}
