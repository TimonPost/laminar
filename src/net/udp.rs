use std::net::{self, ToSocketAddrs};

use error::{NetworkResult, NetworkError, NetworkErrorKind};
use events::Event;
use net::{NetworkConfig, SocketState};
use packet::{Packet, PacketProcessor};

/// Maximum amount of data that we can read from a datagram
const BUFFER_SIZE: usize = 1024;

/// Represents an <ip>:<port> combination listening for UDP traffic
pub struct UdpSocket {
    socket: net::UdpSocket,
    state: SocketState,
    recv_buffer: Vec<u8>,
    config: NetworkConfig,
    packet_processor: PacketProcessor,
}

impl UdpSocket {
    /// Binds to the socket and then sets up the SocketState to manage the connections. Because UDP connections are not persistent, we can only infer the status of the remote endpoint by looking to see if they are sending packets or not
    pub fn bind<A: ToSocketAddrs>(addr: A, config: NetworkConfig) -> NetworkResult<Self> {
        let socket = net::UdpSocket::bind(addr)?;
        let state = SocketState::new(&config)?;

        Ok(UdpSocket {
            socket,
            state,
            recv_buffer: vec![0; config.receive_buffer_max_size],
            packet_processor: PacketProcessor::new(&config),
            config,
        })
    }

    /// Receives a single datagram message on the socket. On success, returns the packet containing origin and data.
    pub fn recv(&mut self) -> NetworkResult<Option<Packet>> {
        let (len, addr) = self
            .socket
            .recv_from(&mut self.recv_buffer)
            .map_err(|io| NetworkErrorKind::IOError { inner: io })?;

        if len > 0 {
            let packet = self.recv_buffer[..len].to_owned();

            self.packet_processor
                .process_data(packet, addr, &mut self.state)
                .map_err(|err| err.into())
        } else {
            Err(NetworkErrorKind::ReceivedDataToShort)?
        }
    }

    /// Sends data on the socket to the given address. On success, returns the number of bytes written.
    pub fn send(&mut self, packet: Packet) -> NetworkResult<usize> {
        let (addr, mut packet_data) = self.state.pre_process_packet(packet, &self.config)?;

        let mut bytes_sent = 0;

        for payload in packet_data.parts() {
            bytes_sent += self
                .socket
                .send_to(&payload, addr)
                .map_err(|io| NetworkError::from(NetworkErrorKind::IOError { inner: io }))?;
        }

        Ok(bytes_sent)
    }

    pub fn events(&self) -> Vec<Event> {
        self.state.events()
    }

    /// Sets the blocking mode of the socket. In non-blocking mode, recv_from will not block if there is no data to be read. In blocking mode, it will. If using non-blocking mode, it is important to wait some amount of time between iterations, or it will quickly use all CPU available
    pub fn set_nonblocking(&mut self, nonblocking: bool) -> NetworkResult<()> {
        match self.socket.set_nonblocking(nonblocking) {
            Ok(_) => Ok(()),
            Err(_e) => Err(NetworkErrorKind::UnableToSetNonblocking.into()),
        }
    }
}
