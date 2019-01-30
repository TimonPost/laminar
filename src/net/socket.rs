use crate::config::NetworkConfig;
use crate::error::{NetworkError, NetworkErrorKind, NetworkResult};
use crate::net::link_conditioner::LinkConditioner;
use crate::net::{connection::ActiveConnections, events::SocketEvent};
use crate::packet::Packet;
use log::error;
use mio::{Evented, Events, Poll, PollOpt, Ready, Token};
use std::io;
use std::mem;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::{mpsc, Arc};

const SOCKET: Token = Token(0);

/// A reliable UDP socket implementation with configurable reliability and ordering guarantees.
pub struct LaminarSocket {
    socket: mio::net::UdpSocket,
    config: Arc<NetworkConfig>,
    connections: ActiveConnections,
    recv_buffer: Vec<u8>,
    link_conditioner: Option<LinkConditioner>,
    event_sender: mpsc::Sender<SocketEvent>,
    packet_receiver: mpsc::Receiver<Packet>,
}

impl LaminarSocket {
    /// Binds to the socket and then sets up `ActiveConnections` to manage the "connections".
    /// Because UDP connections are not persistent, we can only infer the status of the remote
    /// endpoint by looking to see if they are still sending packets or not
    pub fn bind<A: ToSocketAddrs>(
        addresses: A,
        config: NetworkConfig,
    ) -> NetworkResult<(Self, mpsc::Sender<Packet>, mpsc::Receiver<SocketEvent>)> {
        let socket = std::net::UdpSocket::bind(addresses)?;
        let socket = mio::net::UdpSocket::from_socket(socket)?;
        Ok(Self::new(socket, config))
    }

    /// Entry point to the run loop. This should run in a spawned thread since calls to `poll.poll`
    /// are blocking.
    pub fn start_polling(&mut self) -> NetworkResult<()> {
        let poll = Poll::new()?;

        poll.register(self, SOCKET, Ready::readable(), PollOpt::edge())?;

        let mut events = Events::with_capacity(self.config.socket_event_buffer_size);
        let events_ref = &mut events;
        // Packet receiver MUST only be used in this method.
        let packet_receiver = mem::replace(&mut self.packet_receiver, mpsc::channel().1);
        // Nothing should break out of this loop!
        loop {
            self.handle_idle_clients();
            if let Err(e) = poll.poll(events_ref, self.config.socket_polling_timeout) {
                error!("Error polling the socket: {:?}", e);
            }
            if let Err(e) = self.process_events(events_ref) {
                error!("Error processing events: {:?}", e);
            }
            // XXX: I'm fairly certain this isn't exactly safe. I'll likely need to add some
            // handling for when the socket is blocked on send. Worth some more research.
            // Alternatively, I'm sure the Tokio single threaded runtime does handle this for us
            // so maybe it's work switching to that while providing the same interface?
            for packet in packet_receiver.try_iter() {
                if let Err(e) = self.send_to(packet) {
                    error!("Error sending packet: {:?}", e);
                }
            }
        }
    }

    /// Iterate through all of the idle connections based on `idle_connection_timeout` config and
    /// remove them from the active connections. For each connection removed, we will send a
    /// `SocketEvent::TimeOut` event to the `event_sender` channel.
    fn handle_idle_clients(&mut self) {
        let idle_addresses = self
            .connections
            .idle_connections(self.config.idle_connection_timeout);

        for address in idle_addresses {
            self.connections.remove_connection(&address);
            if let Err(err) = self.event_sender.send(SocketEvent::TimeOut(address)) {
                error!("{:?}", err);
            }
        }
    }

    /// Process events received from the mio socket.
    fn process_events(&mut self, events: &mut Events) -> NetworkResult<()> {
        for event in events.iter() {
            match event.token() {
                SOCKET => {
                    if event.readiness().is_readable() {
                        loop {
                            match self.recv_from() {
                                Ok(Some(packet)) => {
                                    if let Err(err) =
                                        self.event_sender.send(SocketEvent::Packet(packet))
                                    {
                                        error!("{:?}", err);
                                    }
                                }
                                Ok(None) => continue,
                                Err(ref err) => match err.kind() {
                                    NetworkErrorKind::IOError(io_err)
                                        if io_err.kind() == io::ErrorKind::WouldBlock =>
                                    {
                                        break;
                                    }
                                    _ => error!("{:?}", err),
                                },
                            };
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
        Ok(())
    }

    /// Serializes and sends a `Packet` on the socket. On success, returns the number of bytes written.
    fn send_to(&mut self, packet: Packet) -> NetworkResult<usize> {
        let connection = self
            .connections
            .get_or_insert_connection(&packet.addr(), self.config.clone());
        let mut packet_data =
            connection.process_outgoing(packet.payload(), packet.delivery_method())?;
        let mut bytes_sent = 0;

        if let Some(link_conditioner) = &self.link_conditioner {
            if link_conditioner.should_send() {
                for payload in packet_data.parts() {
                    bytes_sent += self.send_packet(&packet.addr(), &payload)?;
                }
            }
        } else {
            for payload in connection.gather_dropped_packets() {
                bytes_sent += self.send_packet(&packet.addr(), &payload)?;
            }

            for payload in packet_data.parts() {
                bytes_sent += self.send_packet(&packet.addr(), &payload)?;
            }
        }

        Ok(bytes_sent)
    }

    /// Receives a single message from the socket. On success, returns the packet containing origin and data.
    fn recv_from(&mut self) -> NetworkResult<Option<Packet>> {
        let (recv_len, address) = self.socket.recv_from(&mut self.recv_buffer)?;
        if recv_len <= 0 {
            return Err(NetworkErrorKind::ReceivedDataToShort)?;
        }

        let received_payload = &self.recv_buffer[..recv_len];
        let connection = self
            .connections
            .get_or_insert_connection(&address, self.config.clone());
        connection.process_incoming(received_payload)
    }

    /// Send a single packet over the udp socket.
    fn send_packet(&self, addr: &SocketAddr, payload: &[u8]) -> NetworkResult<usize> {
        let mut bytes_sent = 0;

        bytes_sent += self
            .socket
            .send_to(payload, addr)
            .map_err(|io| NetworkError::from(NetworkErrorKind::IOError(io)))?;

        Ok(bytes_sent)
    }

    fn new(
        socket: mio::net::UdpSocket,
        config: NetworkConfig,
    ) -> (Self, mpsc::Sender<Packet>, mpsc::Receiver<SocketEvent>) {
        let (event_sender, event_receiver) = mpsc::channel();
        let (packet_sender, packet_receiver) = mpsc::channel();
        let buffer_size = config.receive_buffer_max_size;
        (
            Self {
                socket,
                config: Arc::new(config),
                connections: ActiveConnections::new(),
                recv_buffer: vec![0; buffer_size],
                link_conditioner: None,
                event_sender,
                packet_receiver,
            },
            packet_sender,
            event_receiver,
        )
    }
}

impl Evented for LaminarSocket {
    fn register(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        self.socket.register(poll, token, interest, opts)
    }

    fn reregister(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        self.socket.reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        self.socket.deregister(poll)
    }
}
