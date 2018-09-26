use std::io::Error;
use std::net::ToSocketAddrs;

use connection::Manager;
use net::udp::UdpSocket;

/// Highest level abstraction. This is the struct the developers will interface with, send messages to, and receive messages on
pub struct UdpServer {
    socket: UdpSocket,
    manager: Manager,
}

impl UdpServer {
    pub fn new<A: ToSocketAddrs>(addr: A) -> Result<UdpServer, Error> {
        let socket = UdpSocket::bind(addr)?;
        Ok(UdpServer {
            socket,
            manager: Manager::default(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    static TEST_HOST_IP: &'static str = "127.0.0.1";
    static TEST_PORT: &'static str = "21000";

    #[test]
    fn test_create_udp_server() {
        let new_server = UdpServer::new(format!("{}:{}", TEST_HOST_IP, TEST_PORT));
        assert!(new_server.is_ok());
        let new_server = new_server.unwrap();
    }
}
