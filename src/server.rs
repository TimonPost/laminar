use net::udp::UdpSocket;
use connection::Manager;

pub struct Server {
    socket: UdpSocket,
    manager: Manager
}

// impl Server {
//     pub fn new(addr: &str, port: &str) -> Server {
//         let bind_string = addr.to_string() + ":" port;
//         Server{
//             socket: UdpSocket::bind(),
//             manager: Manager::new()
//         }
//     }
// }
