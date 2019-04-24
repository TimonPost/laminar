use crate::{
    config::Config,
    error::{ErrorKind, Result},
    net::{connection::ActiveConnections, events::SocketEvent, link_conditioner::LinkConditioner},
    packet::{DeliveryGuarantee, Outgoing, Packet},
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
        Socket::bind_with_config(addresses, Config::default())
    }

    /// Binds to the socket and then sets up `ActiveConnections` to manage the "connections".
    /// Because UDP connections are not persistent, we can only infer the status of the remote
    /// endpoint by looking to see if they are still sending packets or not
    ///
    /// This function allows you to configure laminar with the passed configuration.
    pub fn bind_with_config<A: ToSocketAddrs>(
        addresses: A,
        config: Config,
    ) -> Result<(Self, Sender<Packet>, Receiver<SocketEvent>)> {
        let socket = UdpSocket::bind(addresses)?;
        socket.set_nonblocking(true)?;
        let (event_sender, event_receiver) = unbounded();
        let (packet_sender, packet_receiver) = unbounded();
        Ok((
            Socket {
                recv_buffer: vec![0; config.receive_buffer_max_size],
                socket,
                config,
                connections: ActiveConnections::new(),
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
            if let Err(e) = self.recv_from() {
                error!("Encountered an error receiving data: {:?}", e);
            };

            // Now grab all the packets waiting to be sent and send them
            while let Ok(p) = self.packet_receiver.try_recv() {
                if let Err(e) = self.send_to(p) {
                    error!("There was an error sending packet: {:?}", e);
                }
            }

            // Finally check for idle clients
            if let Err(e) = self.handle_idle_clients() {
                error!("Encountered an error when sending TimeoutEvent: {:?}", e);
            }
        }
    }

    /// Iterate through all of the idle connections based on `idle_connection_timeout` config and
    /// remove them from the active connections. For each connection removed, we will send a
    /// `SocketEvent::TimeOut` event to the `event_sender` channel.
    fn handle_idle_clients(&mut self) -> Result<()> {
        let idle_addresses = self
            .connections
            .idle_connections(self.config.idle_connection_timeout);
        for address in idle_addresses {
            self.connections.remove_connection(&address);
            self.event_sender.send(SocketEvent::Timeout(address))?;
        }

        Ok(())
    }

    // Serializes and sends a `Packet` on the socket. On success, returns the number of bytes written.
    fn send_to(&mut self, packet: Packet) -> Result<usize> {
        let connection = self
            .connections
            .get_or_insert_connection(packet.addr(), &self.config);

        let processed_packet = connection.process_outgoing(
            packet.payload(),
            packet.delivery_guarantee(),
            packet.order_guarantee(),
        )?;

        let dropped = connection.gather_dropped_packets();
        let mut processed_packets: Vec<Outgoing> = dropped
            .iter()
            .flat_map(|waiting_packet| {
                connection.process_outgoing(
                    &waiting_packet.payload,
                    // Because a delivery guarantee is only sent with reliable packets
                    DeliveryGuarantee::Reliable,
                    // This is stored with the dropped packet because they could be mixed
                    waiting_packet.ordering_guarantee,
                )
            })
            .collect();

        processed_packets.push(processed_packet);

        let mut bytes_sent = 0;

        for processed_packet in processed_packets {
            if self.should_send_packet() {
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
            }
        }
        Ok(bytes_sent)
    }

    // On success the packet will be send on the `event_sender`
    fn recv_from(&mut self) -> Result<()> {
        match self.socket.recv_from(&mut self.recv_buffer) {
            Ok((recv_len, address)) => {
                if recv_len == 0 {
                    return Err(ErrorKind::ReceivedDataToShort)?;
                }
                let received_payload = &self.recv_buffer[..recv_len];

                if !self.connections.exists(&address) {
                    self.event_sender.send(SocketEvent::Connect(address))?;
                }

                let connection = self
                    .connections
                    .get_or_insert_connection(address, &self.config);

                connection.process_incoming(received_payload, &self.event_sender)?;
            }
            Err(e) => {
                if e.kind() != io::ErrorKind::WouldBlock {
                    error!("Encountered an error receiving data: {:?}", e);
                    return Err(e.into());
                }
            }
        }
        Ok(())
    }

    // Send a single packet over the UDP socket.
    fn send_packet(&self, addr: &SocketAddr, payload: &[u8]) -> Result<usize> {
        let bytes_sent = self.socket.send_to(payload, addr)?;
        Ok(bytes_sent)
    }

    // In the presence of a link conditioner, we may not want to send a packet each time.
    fn should_send_packet(&self) -> bool {
        if let Some(link_conditioner) = &self.link_conditioner {
            link_conditioner.should_send()
        } else {
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        net::constants::{ACKED_PACKET_HEADER, FRAGMENT_HEADER_SIZE, STANDARD_HEADER_SIZE},
        Packet, Socket,
    };
    use std::net::SocketAddr;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn can_send_and_receive() {
        let (mut server, _, packet_receiver) =
            Socket::bind("127.0.0.1:12342".parse::<SocketAddr>().unwrap()).unwrap();
        let (mut client, packet_sender, _) =
            Socket::bind("127.0.0.1:12341".parse::<SocketAddr>().unwrap()).unwrap();

        thread::spawn(move || client.start_polling());
        thread::spawn(move || server.start_polling());

        for _ in 0..3 {
            packet_sender
                .send(Packet::unreliable(
                    "127.0.0.1:12342".parse::<SocketAddr>().unwrap(),
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
        let (mut server, _, _) =
            Socket::bind("127.0.0.1:12370".parse::<SocketAddr>().unwrap()).unwrap();

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
        let (mut server, _, _) =
            Socket::bind("127.0.0.1:12371".parse::<SocketAddr>().unwrap()).unwrap();

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
        let (mut server, _, _) =
            Socket::bind("127.0.0.1:12372".parse::<SocketAddr>().unwrap()).unwrap();

        let fragment_packet_size = STANDARD_HEADER_SIZE + FRAGMENT_HEADER_SIZE;

        // the first fragment of an sequence of fragments contains also the acknowledgment header.
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

    #[test]
    fn connect_event_occurs() {
        let (mut server, _, packet_receiver) =
            Socket::bind("127.0.0.1:12345".parse::<SocketAddr>().unwrap()).unwrap();
        let (mut client, packet_sender, _) =
            Socket::bind("127.0.0.1:12344".parse::<SocketAddr>().unwrap()).unwrap();

        thread::spawn(move || client.start_polling());
        thread::spawn(move || server.start_polling());

        packet_sender
            .send(Packet::unreliable(
                "127.0.0.1:12345".parse().unwrap(),
                vec![0, 1, 2],
            ))
            .unwrap();
        assert_eq!(
            packet_receiver.recv().unwrap(),
            SocketEvent::Connect("127.0.0.1:12344".parse().unwrap())
        );
    }

    #[test]
    fn disconnect_event_occurs() {
        let mut config = Config::default();
        config.idle_connection_timeout = Duration::from_millis(1);

        let (mut server, _, packet_receiver) =
            Socket::bind("127.0.0.1:12347".parse::<SocketAddr>().unwrap()).unwrap();
        let (mut client, packet_sender, _) =
            Socket::bind("127.0.0.1:12346".parse::<SocketAddr>().unwrap()).unwrap();

        thread::spawn(move || client.start_polling());
        thread::spawn(move || server.start_polling());

        packet_sender
            .send(Packet::unreliable(
                "127.0.0.1:12347".parse().unwrap(),
                vec![0, 1, 2],
            ))
            .unwrap();

        assert_eq!(
            packet_receiver.recv().unwrap(),
            SocketEvent::Connect("127.0.0.1:12346".parse().unwrap())
        );
        assert_eq!(
            packet_receiver.recv().unwrap(),
            SocketEvent::Packet(Packet::unreliable(
                "127.0.0.1:12346".parse().unwrap(),
                vec![0, 1, 2]
            ))
        );
        assert_eq!(
            packet_receiver.recv().unwrap(),
            SocketEvent::Timeout("127.0.0.1:12346".parse().unwrap())
        );
    }

    const LOCAL_ADDR: &str = "127.0.0.1:13000";
    const REMOTE_ADDR: &str = "127.0.0.1:14000";

    fn create_test_packet(id: u8, addr: &str) -> Packet {
        let payload = vec![id];
        Packet::reliable_unordered(addr.parse().unwrap(), payload)
    }

    #[test]
    fn multiple_sends_should_start_sending_dropped() {
        // Start up a server and a client.
        let (mut server, server_sender, server_receiver) =
            Socket::bind(REMOTE_ADDR.parse::<SocketAddr>().unwrap()).unwrap();
        thread::spawn(move || server.start_polling());

        let (mut client, client_sender, client_receiver) =
            Socket::bind(LOCAL_ADDR.parse::<SocketAddr>().unwrap()).unwrap();
        thread::spawn(move || client.start_polling());

        // Send enough packets to ensure that we must have dropped packets.
        for i in 0..35 {
            client_sender
                .send(create_test_packet(i, REMOTE_ADDR))
                .unwrap();
        }

        let mut events = Vec::new();

        loop {
            if let Ok(event) = server_receiver.recv_timeout(Duration::from_millis(500)) {
                events.push(event);
            } else {
                break;
            }
        }

        // Ensure that we get the correct number of events to the server.
        // 1 connect event plus the 35 messages
        assert_eq!(events.len(), 36);

        // Finally the server decides to send us a message back. This necessarily will include
        // the ack information for 33 of the sent 35 packets.
        server_sender
            .send(create_test_packet(0, LOCAL_ADDR))
            .unwrap();

        // Block to ensure that the client gets the server message before moving on.
        client_receiver.recv().unwrap();

        // This next sent message should end up sending the 2 unacked messages plus the new messages
        // with payload 35
        events.clear();
        client_sender
            .send(create_test_packet(35, REMOTE_ADDR))
            .unwrap();
        loop {
            if let Ok(event) = server_receiver.recv_timeout(Duration::from_millis(500)) {
                events.push(event);
            } else {
                break;
            }
        }

        let sent_events: Vec<u8> = events
            .iter()
            .flat_map(|e| match e {
                SocketEvent::Packet(p) => Some(p.payload()[0]),
                _ => None,
            })
            .collect();
        assert_eq!(sent_events.len(), 3);
        // The order will be guaranteed in a future PR.
        // assert_eq!(sent_events, vec![0, 1, 35]);
    }
}
