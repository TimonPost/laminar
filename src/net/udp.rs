use std::io;
use std::net::{self, ToSocketAddrs, SocketAddr};
use std::collections::HashMap;

use bincode::{deserialize, serialize};
use super::{Packet, RawPacket, SocketState};

const BUFFER_SIZE: usize = 1024;

pub struct UdpSocket {
    socket: net::UdpSocket,
    state: SocketState,
    recv_buffer: [u8; BUFFER_SIZE],
}

impl UdpSocket {
    pub fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        let socket = net::UdpSocket::bind(addr)?;
        let state = SocketState::new();

        Ok(UdpSocket {
            socket,
            state,
            recv_buffer: [0; BUFFER_SIZE],
        })
    }

    pub fn recv(&mut self) -> io::Result<Option<Packet>> {
        let (len, _addr) = self.socket.recv_from(&mut self.recv_buffer)?;

        if len > 0 {
            // TODO: Remove unwrap and funnel result error types
            let raw_packet: RawPacket = deserialize(&self.recv_buffer[..len]).unwrap();
            let packet = self.state.process_received(_addr, &raw_packet);
            Ok(Some(packet))
        } else {
            Ok(None)
        }
    }

    pub fn send(&mut self, mut packet: Packet) -> io::Result<usize> {
        let (addr, payload) = self.state.pre_process_packet(packet);
        self.socket.send_to(&payload, addr)
    }

    pub fn set_nonblocking(&mut self, nonblocking: bool) -> io::Result<()> {
        self.socket.set_nonblocking(nonblocking)
    }
}

mod test
{
    use super::UdpSocket;
    use packet::Packet;
    use std::io;
    use std::net::{SocketAddr, IpAddr};
    use std::str::FromStr;
    use std::{thread, time};
    use bincode::{serialize, deserialize};

    #[test]
    fn send_receive_1_pckt()
    {
        let mut send_socket = UdpSocket::bind("127.0.0.1:12345").unwrap();
        let mut recv_socket = UdpSocket::bind("127.0.0.1:12346").unwrap();

        let addr = SocketAddr::new(
            IpAddr::from_str("0.0.0.0").expect("Unreadable input IP."),
            12345,
        );

        let dummy_packet = Packet::new(addr, vec![1,2,3]);

        let send_result: io::Result<usize> = send_socket.send(dummy_packet);
        assert!(send_result.is_ok());

        let packet: io::Result<Option<Packet>> = recv_socket.recv();
        assert!(packet.is_ok());
        let packet_payload: Option<Packet> = packet.unwrap();
        assert!(packet_payload.is_some());
        let received_packet = packet_payload.unwrap();

        assert_eq!(received_packet.addr().to_string(), "127.0.0.1:12345");
        assert_eq!(received_packet.payload(), &[1,2,3]);
    }

    #[test]
    pub fn send_receive_stress_test()
    {
        const TOTAL_PACKAGES: u16 = 10000;

        thread::spawn(|| {
            thread::sleep(time::Duration::from_millis(3));

            let mut send_socket = UdpSocket::bind("127.0.0.1:12345").unwrap();

            let addr = SocketAddr::new(
                IpAddr::from_str("127.0.0.1").expect("Unreadable input IP."),
                12346,
            );

            for packet_count in 0..TOTAL_PACKAGES
            {
                let stub = StubData { id: packet_count, b: 1};
                let data = serialize(&stub).unwrap();
                let len = data.len();
                let dummy_packet = Packet::new(addr, data);

                let send_result: io::Result<usize> = send_socket.send(dummy_packet);

                assert!(send_result.is_ok());
                println!("sending packet_count: {} packet_id: {}", packet_count, stub.id);
//                assert_eq!(send_result.unwrap(), len);
            }
        });

        thread::spawn(|| {
            let mut recv_socket = UdpSocket::bind("127.0.0.1:12346").unwrap();

            let mut received_packages_count = 0;

            loop {

                let packet: io::Result<Option<Packet>> = recv_socket.recv();
                assert!(packet.is_ok());
                let packet_payload: Option<Packet> = packet.unwrap();
                assert!(packet_payload.is_some());
                let received_packet = packet_payload.unwrap();

                let stub_data = deserialize::<StubData>(received_packet.payload()).unwrap();

                assert_eq!(received_packet.addr().to_string(), "127.0.0.1:12345");
                assert_eq!(stub_data.id, received_packages_count);
                assert_eq!(stub_data.b, 1);

                println!("receiving packet_count: {} packet_id: {}", received_packages_count, stub_data.id);

                received_packages_count += 1;

                if received_packages_count == TOTAL_PACKAGES {
                    break;
                }
            }
        }).join();
    }

    #[derive(Serialize, Deserialize, Clone, Copy)]
    struct StubData
    {
        pub id: u16,
        pub b: u16
    }

    pub fn dummy_packet() -> Packet {
        let addr = SocketAddr::new(
            IpAddr::from_str("0.0.0.0").expect("Unreadable input IP."),
            12345,
        );

        Packet::new(addr, Vec::new())
    }
}
