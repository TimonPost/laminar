use std::io;
use std::net::{self, ToSocketAddrs};

use super::{Packet, RawPacket, SocketState};
use bincode::deserialize;

use error::{NetworkError, Result};

/// Maximum amount of data that we can read from a datagram
const BUFFER_SIZE: usize = 1024;

/// Represents an <ip>:<port> combination listening for UDP traffic
pub struct UdpSocket {
    socket: net::UdpSocket,
    state: SocketState,
    recv_buffer: [u8; BUFFER_SIZE],
}

impl UdpSocket {
    /// Binds to the socket and then sets up the SocketState to manage the connections. Because UDP connections are not persistent, we can only infer the status of the remote endpoint by looking to see if they are sending packets or not
    pub fn bind<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        let socket = net::UdpSocket::bind(addr)?;
        let state = SocketState::new()?;

        Ok(UdpSocket {
            socket,
            state,
            recv_buffer: [0; BUFFER_SIZE],
        })
    }

    /// Attempts to receive any data we have pending from the socket. If the amount of data read exceeds the buffer size, the excess is discarded.
    pub fn recv(&mut self) -> Result<Option<Packet>> {
        let (len, _addr) = self.socket.recv_from(&mut self.recv_buffer)?;
        // If len is 0, then there was no data in the packet
        if len > 0 {
            // If there was data, deserialize it and hand it off for processing
            let raw_packet: RawPacket = deserialize(&self.recv_buffer[..len])?;
            let packet = self.state.process_received(_addr, &raw_packet)?;
            Ok(Some(packet))
        } else {
            Ok(None)
        }
    }

    /// Attempts to send a packet to a specific remote endpoint, i.e., an <ip>:<port> combination of a client
    pub fn send(&mut self, packet: Packet) -> Result<io::Result<usize>> {
        let (addr, payload) = self.state.pre_process_packet(packet)?;
        Ok(self.socket.send_to(&payload, addr))
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
    use super::UdpSocket;
    use bincode::{deserialize, serialize};
    use packet::Packet;
    use std::io;
    use std::net::{IpAddr, SocketAddr};
    use std::str::FromStr;
    use std::{thread, time};

    #[test]
    fn send_receive_1_pckt() {
        let mut send_socket = UdpSocket::bind("127.0.0.1:12347").unwrap();
        let mut recv_socket = UdpSocket::bind("127.0.0.1:12348").unwrap();

        let addr = SocketAddr::new(
            IpAddr::from_str("127.0.0.1").expect("Unreadable input IP."),
            12348,
        );

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
    pub fn send_receive_stress_test() {
        const TOTAL_PACKAGES: u16 = 1000;

        thread::spawn(|| {
            thread::sleep(time::Duration::from_millis(3));

            let mut send_socket = UdpSocket::bind("127.0.0.1:12357").unwrap();

            let addr = SocketAddr::new(
                IpAddr::from_str("127.0.0.1").expect("Unreadable input IP."),
                12358,
            );

            for packet_count in 0..TOTAL_PACKAGES {
                let stub = StubData {
                    id: packet_count,
                    b: 1,
                };
                let data = serialize(&stub).unwrap();
                let len = data.len();
                let dummy_packet = Packet::new(addr, data);
                let send_result = send_socket.send(dummy_packet);
                assert!(send_result.is_ok());
            }
        });

        thread::spawn(|| {
            let mut recv_socket = UdpSocket::bind("127.0.0.1:12358").unwrap();

            let mut received_packages_count = 0;

            loop {
                let packet = recv_socket.recv();
                assert!(packet.is_ok());
                let packet_payload: Option<Packet> = packet.unwrap();
                assert!(packet_payload.is_some());
                let received_packet = packet_payload.unwrap();

                let stub_data = deserialize::<StubData>(received_packet.payload()).unwrap();

                assert_eq!(received_packet.addr().to_string(), "127.0.0.1:12357");
                assert_eq!(stub_data.id, received_packages_count);
                assert_eq!(stub_data.b, 1);

                received_packages_count += 1;
                if received_packages_count >= TOTAL_PACKAGES {
                    break;
                }
            }
        }).join()
        .unwrap();
    }

    #[derive(Serialize, Deserialize, Clone, Copy)]
    struct StubData {
        pub id: u16,
        pub b: u16,
    }

    pub fn dummy_packet() -> Packet {
        let addr = SocketAddr::new(
            IpAddr::from_str("0.0.0.0").expect("Unreadable input IP."),
            12345,
        );

        Packet::new(addr, Vec::new())
    }
}
