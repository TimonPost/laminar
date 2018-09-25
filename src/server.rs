use std;

use net::udp::UdpSocket;
use connection::Manager;

/// Highest level abstraction. This is the struct the developers will interface with, send messages to, and receive messages on
pub struct UdpServer {
    socket: UdpSocket,
    manager: Manager
}

impl UdpServer {
    pub fn new(addr: &str, port: &str) -> Result<UdpServer, std::io::Error> {
        let bind_string = addr.to_owned() + ":" + port;
        match UdpSocket::bind(bind_string) {
            Ok(socket) => {
                Ok(UdpServer{
                    socket: socket,
                    manager: Manager::new()
                })
            },
            Err(e) => {
                Err(e)
            }
        }
    }
}

mod test {
    use super::*;

    static TEST_HOST_IP: &'static str = "127.0.0.1";
    static TEST_PORT: &'static str = "21000";

    #[test]
    fn test_create_udp_server() {
        let new_server = UdpServer::new(TEST_HOST_IP, TEST_PORT);
        assert!(new_server.is_ok());
        let new_server = new_server.unwrap();
    }
}
