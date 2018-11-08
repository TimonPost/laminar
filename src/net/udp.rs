use std::net::{self, ToSocketAddrs, SocketAddr};
use net::connection::ConnectionPool;

use error::{NetworkError, NetworkErrorKind, NetworkResult};
use events::Event;
use net::link_conditioner::LinkConditioner;
use net::NetworkConfig;
use packet::Packet;

use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::error::Error;

/// Represents an <ip>:<port> combination listening for UDP traffic
pub struct UdpSocket {
    socket: net::UdpSocket,
    recv_buffer: Vec<u8>,
    config: Arc<NetworkConfig>,
    link_conditioner: Option<LinkConditioner>,
    timeout_check_thread: thread::JoinHandle<()>,
    events: (Sender<Event>, Receiver<Event>),
    connections: ConnectionPool
}

impl UdpSocket {
    /// Binds to the socket and then sets up the SocketState to manage the connections. Because UDP connections are not persistent, we can only infer the status of the remote endpoint by looking to see if they are sending packets or not
    pub fn bind<A: ToSocketAddrs>(addr: A, config: NetworkConfig) -> NetworkResult<Self> {
        let socket = net::UdpSocket::bind(addr)?;

        let config = &Arc::new(config);
        let (tx, rx) = mpsc::channel();

        let connection_pool = ConnectionPool::new(config);
        let join_handle = connection_pool.start_time_out_loop(tx.clone())?;

        Ok(UdpSocket {
            socket,
            recv_buffer: vec![0;config.receive_buffer_max_size],
            config: config.clone(),
            link_conditioner: None,
            connections: connection_pool,
            timeout_check_thread: join_handle,
            events: (tx, rx),
        })
    }

    /// Receives a single datagram message on the socket. On success, returns the packet containing origin and data.
    pub fn recv(&mut self) -> NetworkResult<Option<Packet>> {
        let (len, addr) = self
            .socket
            .recv_from(&mut self.recv_buffer)
            .map_err(|io| NetworkErrorKind::IOError(io))?;

        if len > 0 {
            let packet = &self.recv_buffer[..len];

            let connection = self.connections.get_connection_or_insert(&addr)?;
            let mut lock = connection
                .write()
                .map_err(|error| NetworkError::poisoned_connection_error(error.description()))?;

            return lock.process_incoming(&packet)

        } else {
            Err(NetworkErrorKind::ReceivedDataToShort)?
        }
    }

    /// Sends data on the socket to the given address. On success, returns the number of bytes written.
    pub fn send(&mut self, packet: Packet) -> NetworkResult<usize> {
        let connection = self.connections.get_connection_or_insert(&packet.addr())?;
        let mut lock = connection
            .write()
            .map_err(|error| NetworkError::poisoned_connection_error(error.description()))?;

        let mut packet_data = lock.process_outgoing(packet.payload(), packet.delivery_method())?;

        let mut bytes_sent = 0;

        if let Some(link_conditioner) = &self.link_conditioner {
            if link_conditioner.should_send() {
                for payload in packet_data.parts() {
                    bytes_sent += self.send_packet(&packet.addr(), &payload)?;
                }
            }
        } else {
            for payload in lock.gather_dropped_packets() {
                bytes_sent += self.send_packet(&packet.addr(), &payload)?;
            }

            for payload in packet_data.parts() {
                bytes_sent += self.send_packet(&packet.addr(), &payload)?;
            }
        }

        Ok(bytes_sent)
    }

    /// Send a single packet over the udp socket.
    fn send_packet(&self, addr: &SocketAddr, payload: &[u8]) -> NetworkResult<usize>  {
        let mut bytes_sent = 0;

        bytes_sent += self
            .socket
            .send_to(payload, addr)
            .map_err(|io| NetworkError::from(NetworkErrorKind::IOError(io)))?;

        Ok(bytes_sent)
    }

    /// Sets the blocking mode of the socket. In non-blocking mode, recv_from will not block if there is no data to be read. In blocking mode, it will. If using non-blocking mode, it is important to wait some amount of time between iterations, or it will quickly use all CPU available
    pub fn set_nonblocking(&mut self, nonblocking: bool) -> NetworkResult<()> {
        match self.socket.set_nonblocking(nonblocking) {
            Ok(_) => Ok(()),
            Err(_e) => Err(NetworkErrorKind::UnableToSetNonblocking.into()),
        }
    }

    /// This will return a `Vec` of events for processing.
    pub fn events(&self) -> Vec<Event> {
        let (_, ref rx) = self.events;

        rx.try_iter().collect()
    }

    /// Wrapper around getting the events sender
    /// This will cause a clone to be done, but this is low cost
    pub fn get_events_sender(&self) -> Sender<Event> {
        self.events.0.clone()
    }
}
