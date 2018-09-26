use std;
use std::net::{ToSocketAddrs, SocketAddr};

use net::udp::UdpSocket;
use connection::Manager;

/// Highest level abstraction. This is the struct the developers will interface with, send messages to, and receive messages on
pub struct UdpServer {
    socket: UdpSocket,
    manager: Manager
}

impl UdpServer {
    pub fn new<A: ToSocketAddrs>(addr: A) -> Result<UdpServer, std::io::Error> {
        addr.to_socket_addrs().and_then(|mut i| UdpSocket::bind(i.next().unwrap())).and_then(|mut bound| {
            Ok(UdpServer {
                socket: bound,
                manager: Manager::new(),
            })
        })

            // UdpSocket::bind(i.next()
            //.and_then(|socket| UdpServer{socket: socket, manager: Manager::new()})
    }
}

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
