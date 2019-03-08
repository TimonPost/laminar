use crate::{
    config::Config,
    error::{ErrorKind, Result},
    net::{connection::ActiveConnections, events::SocketEvent, link_conditioner::LinkConditioner},
    packet::Packet,
};
use crossbeam_channel::{self, unbounded, Receiver, Sender};
use log::error;
use std::{
    self, io,
    net::{SocketAddr, ToSocketAddrs, UdpSocket},
};

/// A reliable UDP socket implementation with configurable reliability and ordering guarantees.
pub struct Socket {
    socket: UdpSocket,
    config: Config,
    connections: ActiveConnections,
    recv_buffer: Vec<u8>,
    link_conditioner: Option<LinkConditioner>,
    event_sender: Sender<SocketEvent>,
    packet_receiver: Receiver<Packet>,
}

impl Socket {
    /// Binds to the socket and then sets up `ActiveConnections` to manage the "connections".
    /// Because UDP connections are not persistent, we can only infer the status of the remote
    /// endpoint by looking to see if they are still sending packets or not
    pub fn bind<A: ToSocketAddrs>(
        addresses: A,
        config: Config,
    ) -> Result<(Self, Sender<Packet>, Receiver<SocketEvent>)> {
        let socket = UdpSocket::bind(addresses)?;
        socket.set_nonblocking(true)?;
        let (event_sender, event_receiver) = unbounded();
        let (packet_sender, packet_receiver) = unbounded();
        Ok((
            Socket {
                socket,
                config,
                connections: ActiveConnections::new(),
                recv_buffer: Vec::new(),
                link_conditioner: None,
                event_sender,
                packet_receiver,
            },
            packet_sender,
            event_receiver,
        ))
    }

    /// Entry point to the run loop. This should run in a spawned thread since calls to `poll.poll`
    /// are blocking.
    pub fn start_polling(&mut self) -> Result<()> {
        // Nothing should break out of this loop!
        loop {
            // First we pull any newly arrived packets and handle them
            match self.recv_from() {
                Ok(result) => match result {
                    Some(packet) => {
                        match self.event_sender.send(SocketEvent::Packet(packet)) {
                            Ok(_) => {}
                            Err(e) => {
                                error!("Error sending SocketEvent: {:?}", e);
                            }
                        };
                    }
                    None => {
                        error!("Empty packet received");
                    }
                },
                Err(e) => {
                    error!("Error receiving packet: {:?}", e);
                }
            };

            // Now grab all the packets waiting to be sent and send them
            let to_send: Vec<Packet> = self.packet_receiver.try_iter().collect();
            for p in to_send {
                match self.send_to(p) {
                    Ok(_) => {}
                    Err(e) => {
                        error!("Error sending packet: {:?}", e);
                    }
                }
            }

            // Finally check for idle clients
            self.handle_idle_clients();
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
        }
    }

    // Serializes and sends a `Packet` on the socket. On success, returns the number of bytes written.
    fn send_to(&mut self, packet: Packet) -> Result<usize> {
        let connection = self
            .connections
            .get_or_insert_connection(packet.addr(), &self.config);
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

    // Receives a single message from the socket. On success, returns the packet containing origin and data.
    fn recv_from(&mut self) -> Result<Option<Packet>> {
        match self.socket.recv_from(&mut self.recv_buffer) {
            Ok((recv_len, address)) => {
                if recv_len == 0 {
                    return Err(ErrorKind::ReceivedDataToShort)?;
                }
                let received_payload = &self.recv_buffer[..recv_len];
                let connection = self
                    .connections
                    .get_or_insert_connection(address, &self.config);
                connection.process_incoming(received_payload)
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                error!("Encountered a WouldBlock: {:?}", e);
                Ok(None)
            }
            Err(e) => {
                error!("Encountered an error receiving data: {:?}", e);
                Ok(None)
            }
        }
    }

    // Send a single packet over the UDP socket.
    fn send_packet(&self, addr: &SocketAddr, payload: &[u8]) -> Result<usize> {
        let bytes_sent = self.socket.send_to(payload, addr)?;
        Ok(bytes_sent)
    }

    #[allow(dead_code)]
    fn new(socket: UdpSocket, config: Config) -> (Self, Sender<Packet>, Receiver<SocketEvent>) {
        let _ = socket.set_nonblocking(true);
        let (event_sender, event_receiver) = unbounded();
        let (packet_sender, packet_receiver) = unbounded();
        let buffer_size = config.receive_buffer_max_size;
        (
            Self {
                socket,
                config,
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
