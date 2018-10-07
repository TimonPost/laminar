use std::net::{self, ToSocketAddrs};


use packet::{Packet, PacketProcessor};
use net::{NetworkConfig, SocketState};
use error::{NetworkError, Result};

/// Maximum amount of data that we can read from a datagram
const BUFFER_SIZE: usize = 1024;

/// Represents an <ip>:<port> combination listening for UDP traffic
pub struct UdpSocket {
    socket: net::UdpSocket,
    state: SocketState,
    recv_buffer: Vec<u8>,
    config: NetworkConfig,
    packet_processor: PacketProcessor
}

impl UdpSocket {
    /// Binds to the socket and then sets up the SocketState to manage the connections. Because UDP connections are not persistent, we can only infer the status of the remote endpoint by looking to see if they are sending packets or not
    pub fn bind<A: ToSocketAddrs>(addr: A, config: NetworkConfig) -> Result<Self> {
        let socket = net::UdpSocket::bind(addr)?;
        let state = SocketState::new()?;

        Ok(UdpSocket {
            socket,
            state,
            recv_buffer: vec![0;config.receive_buffer_max_size],
            packet_processor: PacketProcessor::new(config.clone()),
            config,
        })
    }

    /// Receives a single datagram message on the socket. On success, returns the packet containing origin and data.
    pub fn recv(&mut self) -> Result<Option<Packet>> {
        let (len, addr) = self.socket.recv_from(&mut self.recv_buffer).map_err(|_| NetworkError::ReceiveFailed)?;

        if len > 0 {
            let packet = self.recv_buffer[..len].to_owned();

            self.packet_processor.process_data(packet, addr, &mut self.state)
        }else {
            return Err (NetworkError::ReceiveFailed.into());
        }
    }

    /// Sends data on the socket to the given address. On success, returns the number of bytes written.
    pub fn send(&mut self, mut packet: Packet) -> Result<usize> {
        let (addr, mut packet_data) = self.state.pre_process_packet(packet, &self.config)?;

        let mut bytes_sent = 0;

        for payload in packet_data.parts() {
            bytes_sent += self.socket.send_to(&payload, addr).map_err(|_| NetworkError::SendFailed)?;
        }

        Ok(bytes_sent)
    }

    /// Sets the blocking mode of the socket. In non-blocking mode, recv_from will not block if there is no data to be read. In blocking mode, it will. If using non-blocking mode, it is important to wait some amount of time between iterations, or it will quickly use all CPU available
    pub fn set_nonblocking(&mut self, nonblocking: bool) -> Result<()> {
        match self.socket.set_nonblocking(nonblocking) {
            Ok(_) => {
                Ok(())
            }
            Err(e) => {
                Err(NetworkError::UnableToSetNonblocking.into())
            }
        }
    }
}

#[cfg(test)]
mod test {
    use net::{UdpSocket, NetworkConfig, constants};
    use bincode::{deserialize, serialize};
    use packet::Packet;
    use std::io;
    use std::net::{IpAddr, SocketAddr};
    use std::str::FromStr;
    use std::{thread, time};

    #[test]
    #[ignore]
    fn send_receive_1_pckt() {
        let mut send_socket = UdpSocket::bind("127.0.0.1:12347",NetworkConfig::default()).unwrap();
        let mut recv_socket = UdpSocket::bind("127.0.0.1:12348",NetworkConfig::default()).unwrap();

        let addr = "127.0.0.1:12348".parse().unwrap();

        let dummy_packet = Packet::new(addr, vec![1, 2, 3]);

        let send_result = send_socket.send(dummy_packet);
        assert!(send_result.is_ok());

        let packet = recv_socket.recv();
        assert!(packet.is_ok());
        let packet_payload: Option<Packet> = packet.unwrap();
        assert!(packet_payload.is_some());
        let received_packet = packet_payload.unwrap();

        assert_eq!(received_packet.addr().to_string(), "127.0.0.1:12347");
        assert_eq!(received_packet.payload(), &[1, 2, 3]);
    }

    #[test]
    #[ignore]
    fn send_receive_fragment_packet() {
        let mut send_socket = UdpSocket::bind("127.0.0.1:12347",NetworkConfig::default()).unwrap();
        let mut recv_socket = UdpSocket::bind("127.0.0.1:12348",NetworkConfig::default()).unwrap();

        let addr = "127.0.0.1:12348".parse().unwrap();

        let handle = thread::spawn(move || {
            loop {
                let packet = recv_socket.recv();

                match packet {
                    Ok(Some(packet)) => {
                        assert_eq!(packet.addr().to_string(), "127.0.0.1:12347");
                        assert_eq!(packet.payload(), vec![123; 4000].as_slice());
                        break;
                    }
                    Err(e) => { panic!("{:?}",e); },
                    _ => { }
                };
            }
        });

        let dummy_packet = Packet::new(addr, vec![123;4000]);
        let send_result = send_socket.send(dummy_packet);
        assert!(send_result.is_ok());

        handle.join();
    }

    #[test]
    #[ignore]
    pub fn send_receive_stress_test() {
        const TOTAL_PACKAGES: u64 = 10000;

        thread::spawn(|| {
            thread::sleep(time::Duration::from_millis(3));

            let mut send_socket = UdpSocket::bind("127.0.0.1:12357",NetworkConfig::default()).unwrap();

            let addr = "127.0.0.1:12358".parse().unwrap();

            for packet_count in 0..TOTAL_PACKAGES {

                let stub = StubData {
                    id: packet_count,
                    b: 400,
                };

                let data = serialize(&stub).unwrap();
                let len = data.len();

                let dummy_packet = Packet::new(addr, data);
                let send_result = send_socket.send(dummy_packet);
                assert!(send_result.is_ok());
                assert_eq!(send_result.unwrap(), len + constants::PACKET_HEADER_SIZE as usize);
            }
        });

        thread::spawn(|| {
            let mut recv_socket = UdpSocket::bind("127.0.0.1:12358", NetworkConfig::default()).unwrap();
            let mut received_packages_count = 0;
            loop {
                let packet= recv_socket.recv();
                assert!(packet.is_ok());

                let packet_payload: Option<Packet> = packet.unwrap();
                assert!(packet_payload.is_some());
                let received_packet = packet_payload.unwrap();

                let stub_data = deserialize::<StubData>(received_packet.payload()).unwrap();

                assert_eq!(received_packet.addr().to_string(), "127.0.0.1:12357");
                assert_eq!(stub_data.b, 400);
                received_packages_count += 1;
            }
        }).join()
        .unwrap();
    }

    #[derive(Serialize, Deserialize, Clone, Copy)]
    struct StubData {
        pub id: u64,
        pub b: u16,
    }

    pub fn dummy_packet() -> Packet {
        let addr = "127.0.0.1:12345".parse().unwrap();
        Packet::new(addr, Vec::new())
    }
}
