use crate::{
    config::Config,
    error::{ErrorKind, Result},
    net::{connection::ActiveConnections, events::SocketEvent, link_conditioner::LinkConditioner, constants::DEFAULT_MTU},
    packet::{Outgoing, Packet},
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
    ) -> Result<(Self, Sender<Packet>, Receiver<SocketEvent>)> {
        let socket = UdpSocket::bind(addresses)?;
        socket.set_nonblocking(true)?;
        let (event_sender, event_receiver) = unbounded();
        let (packet_sender, packet_receiver) = unbounded();
        Ok((
            Socket {
                recv_buffer: vec![0; DEFAULT_MTU as usize],
                socket,
                config: Config::default(),
                connections: ActiveConnections::new(),
                link_conditioner: None,
                event_sender,
                packet_receiver,
            },
            packet_sender,
            event_receiver,
        ))
    }

    /// Configure the socket with the passed configuration.
    pub fn with_config(mut self, config: Config) -> Socket {
        self.recv_buffer = vec![0; config.receive_buffer_max_size];
        self.config = config;
        self
    }

    /// Entry point to the run loop. This should run in a spawned thread since calls to `poll.poll`
    /// are blocking.
    pub fn start_polling(&mut self) -> Result<()> {
        // Nothing should break out of this loop!
        loop {
            // First we pull any newly arrived packets and handle them
            if let Err(e) = self.recv_from() {
                error!("Error receiving packet: {:?}", e);
            };

            // Now grab all the packets waiting to be sent and send them
            while let Ok(p) = self.packet_receiver.try_recv() {
                if let Err(e) = self.send_to(p) {
                    error!("There was an error sending packet: {:?}", e);
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
        let (dropped_packets, processed_packet) = {
            let connection = self
                .connections
                .get_or_insert_connection(packet.addr(), &self.config);

            let processed_packet = connection.process_outgoing(
                packet.payload(),
                packet.delivery_guarantee(),
                packet.order_guarantee(),
            )?;

            (connection.gather_dropped_packets(), processed_packet)
        };

        let mut bytes_sent = 0;

        let should_send = if let Some(link_conditioner) = &self.link_conditioner {
            link_conditioner.should_send()
        } else {
            true
        };

        if should_send {
            match processed_packet {
                Outgoing::Packet(outgoing) => {
                    bytes_sent += self.send_packet(&packet.addr(), &outgoing.contents())?;
                }
                Outgoing::Fragments(packets) => {
                    for outgoing in packets {
                        bytes_sent += self.send_packet(&packet.addr(), &outgoing.contents())?;
                    }
                }
            }

            for payload in dropped_packets {
                bytes_sent += self.send_packet(&packet.addr(), &payload)?;
            }

            return Ok(bytes_sent);
        }

        Ok(0)
    }

    // On success the packet will be send on the `event_sender`
    fn recv_from(&mut self) -> Result<()> {
        match self.socket.recv_from(&mut self.recv_buffer) {
            Ok((recv_len, address)) => {
                if recv_len == 0 {
                    return Err(ErrorKind::ReceivedDataToShort)?;
                }
                let received_payload = &self.recv_buffer[..recv_len];
                let connection = self
                    .connections
                    .get_or_insert_connection(address, &self.config);
                connection.process_incoming(received_payload, &self.event_sender)?;
            }
            Err(e) => {
                if e.kind() == io::ErrorKind::WouldBlock {
                    error!("Encountered a WouldBlock error: {:?}", e);
                } else {
                    error!("Encountered an error receiving data: {:?}", e);
                }
                return Err(e.into());
            }
        }
        Ok(())
    }

    // Send a single packet over the UDP socket.
    fn send_packet(&self, addr: &SocketAddr, payload: &[u8]) -> Result<usize> {
        let bytes_sent = self.socket.send_to(payload, addr)?;
        Ok(bytes_sent)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        net::constants::{ACKED_PACKET_HEADER, FRAGMENT_HEADER_SIZE, STANDARD_HEADER_SIZE},
        Config, Packet, Socket,
    };
    use std::net::SocketAddr;
    use std::thread;

    #[test]
    fn can_send_and_receive() {
        let (mut server, _, packet_receiver) = Socket::bind(
            "127.0.0.1:12345".parse::<SocketAddr>().unwrap()
        )
        .unwrap();
        let (mut client, packet_sender, _) = Socket::bind(
            "127.0.0.1:12344".parse::<SocketAddr>().unwrap()
        )
        .unwrap();

        thread::spawn(move || client.start_polling());
        thread::spawn(move || server.start_polling());

        for _ in 0..3 {
            packet_sender
                .send(Packet::unreliable(
                    "127.0.0.1:12345".parse::<SocketAddr>().unwrap(),
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                ))
                .unwrap();
        }

        let mut iter = packet_receiver.iter();

        assert!(iter.next().is_some());
        assert!(iter.next().is_some());
        assert!(iter.next().is_some());
    }

    #[test]
    fn sending_large_unreliable_packet_should_fail() {
        let (mut server, _, packet_receiver) = Socket::bind(
            "127.0.0.1:12370".parse::<SocketAddr>().unwrap()
        )
        .unwrap();

        assert_eq!(
            server
                .send_to(Packet::unreliable(
                    "127.0.0.1:12360".parse().unwrap(),
                    vec![1; 5000]
                ))
                .is_err(),
            true
        );
    }

    #[test]
    fn send_returns_right_size() {
        let (mut server, _, packet_receiver) = Socket::bind(
            "127.0.0.1:12371".parse::<SocketAddr>().unwrap()
        )
        .unwrap();

        assert_eq!(
            server
                .send_to(Packet::unreliable(
                    "127.0.0.1:12361".parse().unwrap(),
                    vec![1; 1024]
                ))
                .unwrap(),
            1024 + STANDARD_HEADER_SIZE as usize
        );
    }

    #[test]
    fn fragmentation_send_returns_right_size() {
        let (mut server, _, packet_receiver) = Socket::bind(
            "127.0.0.1:12372".parse::<SocketAddr>().unwrap()
        )
        .unwrap();

        let fragment_packet_size = STANDARD_HEADER_SIZE + FRAGMENT_HEADER_SIZE;

        // the first fragment of an sequence of fragments contains also the acknowledgement header.
        assert_eq!(
            server
                .send_to(Packet::reliable_unordered(
                    "127.0.0.1:12362".parse().unwrap(),
                    vec![1; 4000]
                ))
                .unwrap(),
            4000 + (fragment_packet_size * 4 + ACKED_PACKET_HEADER) as usize
        );
    }
}
