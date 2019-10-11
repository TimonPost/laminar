use std::{self, collections::HashMap, fmt::Debug, io::Result, net::SocketAddr, time::Instant};

use crossbeam_channel::{self, unbounded, Receiver, Sender};
use log::error;

use crate::{
    config::Config, net::Connection, net::ConnectionEventAddress, net::ConnectionMessenger,
};

// TODO: maybe we can make a breaking change and use this instead of `ConnectionEventAddress` trait?
// #[derive(Debug)]
// pub struct ConnectionEvent<Event: Debug>(pub SocketAddr, pub Event);

/// A datagram socket is a type of network socket which provides a connectionless point for sending or receiving data packets.
pub trait DatagramSocket: Debug {
    /// Sends a single packet to the socket.
    fn send_packet(&mut self, addr: &SocketAddr, payload: &[u8]) -> Result<usize>;

    /// Receives a single packet from the socket.
    fn receive_packet<'a>(&mut self, buffer: &'a mut [u8]) -> Result<(&'a [u8], SocketAddr)>;

    /// Returns the socket address that this socket was created from.
    fn local_addr(&self) -> Result<SocketAddr>;

    /// Returns whether socket operates in blocking or nonblocking mode.
    fn is_blocking_mode(&self) -> bool;
}

// This will be used by a `Connection`.
#[derive(Debug)]
struct SocketEventSenderAndConfig<TSocket: DatagramSocket, ReceiveEvent: Debug> {
    config: Config,
    socket: TSocket,
    event_sender: Sender<ReceiveEvent>,
}

impl<TSocket: DatagramSocket, ReceiveEvent: Debug>
    SocketEventSenderAndConfig<TSocket, ReceiveEvent>
{
    fn new(config: Config, socket: TSocket, event_sender: Sender<ReceiveEvent>) -> Self {
        Self {
            config,
            socket,
            event_sender,
        }
    }
}

impl<TSocket: DatagramSocket, ReceiveEvent: Debug> ConnectionMessenger<ReceiveEvent>
    for SocketEventSenderAndConfig<TSocket, ReceiveEvent>
{
    fn config(&self) -> &Config {
        &self.config
    }

    fn send_event(&mut self, _address: &SocketAddr, event: ReceiveEvent) {
        self.event_sender.send(event).expect("Receiver must exists");
    }

    fn send_packet(&mut self, address: &SocketAddr, payload: &[u8]) {
        if let Err(err) = self.socket.send_packet(address, payload) {
            error!("Error occured sending a packet (to {}): {}", address, err)
        }
    }
}

/// Implements a concept of connections on top of datagram socket.
/// Connection capabilities depends on what is an actual `Connection` type.
/// Connection type also defines a type of sending and receiving events.
#[derive(Debug)]
pub struct ConnectionManager<TSocket: DatagramSocket, TConnection: Connection> {
    connections: HashMap<SocketAddr, TConnection>,
    receive_buffer: Vec<u8>,
    user_event_receiver: Receiver<TConnection::SendEvent>,
    messenger: SocketEventSenderAndConfig<TSocket, TConnection::ReceiveEvent>,
    event_receiver: Receiver<TConnection::ReceiveEvent>,
    user_event_sender: Sender<TConnection::SendEvent>,
}

impl<TSocket: DatagramSocket, TConnection: Connection> ConnectionManager<TSocket, TConnection> {
    /// Creates an instance of `ConnectionManager` by passing a socket and config.
    pub fn new(socket: TSocket, config: Config) -> Self {
        let (event_sender, event_receiver) = unbounded();
        let (user_event_sender, user_event_receiver) = unbounded();
        ConnectionManager {
            receive_buffer: vec![0; config.receive_buffer_max_size],
            connections: Default::default(),
            user_event_receiver,
            messenger: SocketEventSenderAndConfig::new(config, socket, event_sender),
            user_event_sender,
            event_receiver,
        }
    }

    /// Processes any inbound/outbound packets and events.
    /// Processes connection specific logic for active connections.
    /// Removes dropped connections from active connections list.
    pub fn manual_poll(&mut self, time: Instant) {
        let messenger = &mut self.messenger;
        // first we pull all newly arrived packets and handle them
        loop {
            match messenger
                .socket
                .receive_packet(self.receive_buffer.as_mut())
            {
                Ok((payload, address)) => {
                    if let Some(conn) = self.connections.get_mut(&address) {
                        conn.process_packet(messenger, payload, time);
                    } else {
                        // create connection, but do not add to active connections list
                        let mut conn =
                            TConnection::create_connection(messenger, address, time, Some(payload));
                        conn.process_packet(messenger, payload, time);
                    }
                }
                Err(e) => {
                    if e.kind() != std::io::ErrorKind::WouldBlock {
                        error!("Encountered an error receiving data: {:?}", e);
                    }
                    break;
                }
            }
            // prevent from blocking, break after receiving first packet
            if messenger.socket.is_blocking_mode() {
                break;
            }
        }

        // now grab all the waiting packets and send them
        while let Ok(event) = self.user_event_receiver.try_recv() {
            // get or create connection
            let conn = self.connections.entry(event.address()).or_insert_with(|| {
                TConnection::create_connection(messenger, event.address(), time, None)
            });
            conn.process_event(messenger, event, time);
        }

        // update all connections
        for conn in self.connections.values_mut() {
            conn.update(messenger, time);
        }

        // iterate through all connections and remove those that should be dropped
        self.connections
            .retain(|_, conn| !conn.should_drop(messenger, time));
    }

    /// Returns a handle to the event sender which provides a thread-safe way to enqueue user events
    /// to be processed. This should be used when the socket is busy running its polling loop in a
    /// separate thread.
    pub fn event_sender(&self) -> &Sender<TConnection::SendEvent> {
        &self.user_event_sender
    }

    /// Returns a handle to the event receiver which provides a thread-safe way to retrieve events
    /// from the connections. This should be used when the socket is busy running its polling loop in
    /// a separate thread.
    pub fn event_receiver(&self) -> &Receiver<TConnection::ReceiveEvent> {
        &self.event_receiver
    }

    /// Returns socket reference.
    pub fn socket(&self) -> &TSocket {
        &self.messenger.socket
    }

    /// Returns socket mutable reference.
    #[allow(dead_code)]
    pub fn socket_mut(&mut self) -> &mut TSocket {
        &mut self.messenger.socket
    }

    /// Returns a number of active connections.
    #[cfg(test)]
    pub fn connections_count(&self) -> usize {
        self.connections.len()
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashSet,
        net::SocketAddr,
        time::{Duration, Instant},
    };

    use crate::net::LinkConditioner;
    use crate::test_utils::*;
    use crate::{Config, Packet, SocketEvent};

    /// The socket address of where the server is located.
    const SERVER_ADDR: &str = "127.0.0.1:10001";
    // The client address from where the data is sent.
    const CLIENT_ADDR: &str = "127.0.0.1:10002";

    fn client_address() -> SocketAddr {
        CLIENT_ADDR.parse().unwrap()
    }

    fn server_address() -> SocketAddr {
        SERVER_ADDR.parse().unwrap()
    }

    fn create_server_client_network() -> (FakeSocket, FakeSocket, NetworkEmulator) {
        let network = NetworkEmulator::default();
        let server = FakeSocket::bind(&network, server_address(), Config::default()).unwrap();
        let client = FakeSocket::bind(&network, client_address(), Config::default()).unwrap();
        (server, client, network)
    }

    fn create_server_client(config: Config) -> (FakeSocket, FakeSocket) {
        let network = NetworkEmulator::default();
        let server = FakeSocket::bind(&network, server_address(), config.clone()).unwrap();
        let client = FakeSocket::bind(&network, client_address(), config).unwrap();
        (server, client)
    }

    #[test]
    fn using_sender_and_receiver() {
        let (mut server, mut client, _) = create_server_client_network();

        let sender = client.get_packet_sender();
        let receiver = server.get_event_receiver();

        sender
            .send(Packet::reliable_unordered(
                server_address(),
                b"Hello world!".to_vec(),
            ))
            .unwrap();

        let time = Instant::now();
        client.manual_poll(time);
        server.manual_poll(time);

        assert_eq![Ok(SocketEvent::Connect(client_address())), receiver.recv()];
        if let SocketEvent::Packet(packet) = receiver.recv().unwrap() {
            assert_eq![b"Hello world!", packet.payload()];
        } else {
            panic!["Did not receive a packet when it should"];
        }
    }

    #[test]
    fn initial_packet_is_resent() {
        let (mut server, mut client, network) = create_server_client_network();
        let time = Instant::now();

        // send a packet that the server ignores/drops
        client
            .send(Packet::reliable_unordered(
                server_address(),
                b"Do not arrive".to_vec(),
            ))
            .unwrap();
        client.manual_poll(time);

        // drop the inbound packet, this simulates a network error
        network.clear_packets(server_address());

        // send a packet that the server receives
        for id in 0..u8::max_value() {
            client
                .send(Packet::reliable_unordered(server_address(), vec![id]))
                .unwrap();

            server
                .send(Packet::reliable_unordered(client_address(), vec![id]))
                .unwrap();

            client.manual_poll(time);
            server.manual_poll(time);

            while let Some(SocketEvent::Packet(pkt)) = server.recv() {
                if pkt.payload() == b"Do not arrive" {
                    return;
                }
            }
            while let Some(_) = client.recv() {}
        }

        panic!["Did not receive the ignored packet"];
    }

    #[test]
    fn receiving_does_not_allow_denial_of_service() {
        let (mut server, mut client, _) = create_server_client_network();
        // send a bunch of packets to a server
        for _ in 0..3 {
            client
                .send(Packet::unreliable(
                    server_address(),
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                ))
                .unwrap();
        }

        let time = Instant::now();

        client.manual_poll(time);
        server.manual_poll(time);

        for _ in 0..6 {
            assert![server.recv().is_some()];
        }
        assert![server.recv().is_none()];

        // the server shall not have any connection in its connection table even though it received
        // packets
        assert_eq![0, server.connection_count()];

        server
            .send(Packet::unreliable(client_address(), vec![1]))
            .unwrap();

        server.manual_poll(time);

        // the server only adds to its table after having sent explicitly
        assert_eq![1, server.connection_count()];
    }

    #[test]
    fn initial_sequenced_is_resent() {
        let (mut server, mut client, network) = create_server_client_network();
        let time = Instant::now();

        // send a packet that the server ignores/drops
        client
            .send(Packet::reliable_sequenced(
                server_address(),
                b"Do not arrive".to_vec(),
                None,
            ))
            .unwrap();
        client.manual_poll(time);

        // drop the inbound packet, this simulates a network error
        network.clear_packets(server_address());

        // send a packet that the server receives
        for id in 0..36 {
            client
                .send(Packet::reliable_sequenced(server_address(), vec![id], None))
                .unwrap();

            server
                .send(Packet::reliable_sequenced(client_address(), vec![id], None))
                .unwrap();

            client.manual_poll(time);
            server.manual_poll(time);

            while let Some(SocketEvent::Packet(pkt)) = server.recv() {
                if pkt.payload() == b"Do not arrive" {
                    panic!["Sequenced packet arrived while it should not"];
                }
            }
            while let Some(_) = client.recv() {}
        }
    }

    #[test]
    fn initial_ordered_is_resent() {
        let (mut server, mut client, network) = create_server_client_network();
        let time = Instant::now();

        // send a packet that the server ignores/drops
        client
            .send(Packet::reliable_ordered(
                server_address(),
                b"Do not arrive".to_vec(),
                None,
            ))
            .unwrap();
        client.manual_poll(time);

        // drop the inbound packet, this simulates a network error
        network.clear_packets(server_address());

        // send a packet that the server receives
        for id in 0..35 {
            client
                .send(Packet::reliable_ordered(server_address(), vec![id], None))
                .unwrap();

            server
                .send(Packet::reliable_ordered(client_address(), vec![id], None))
                .unwrap();

            client.manual_poll(time);
            server.manual_poll(time);

            while let Some(SocketEvent::Packet(pkt)) = server.recv() {
                if pkt.payload() == b"Do not arrive" {
                    return;
                }
            }
            while let Some(_) = client.recv() {}
        }

        panic!["Did not receive the ignored packet"];
    }

    #[test]
    fn do_not_duplicate_sequenced_packets_when_received() {
        let (mut server, mut client, _) = create_server_client_network();
        let time = Instant::now();

        for id in 0..100 {
            client
                .send(Packet::reliable_sequenced(server_address(), vec![id], None))
                .unwrap();
            client.manual_poll(time);
            server.manual_poll(time);
        }

        let mut seen = HashSet::new();

        while let Some(message) = server.recv() {
            match message {
                SocketEvent::Connect(_) => {}
                SocketEvent::Packet(packet) => {
                    let byte = packet.payload()[0];
                    assert![!seen.contains(&byte)];
                    seen.insert(byte);
                }
                SocketEvent::Timeout(_) => {
                    panic!["This should not happen, as we've not advanced time"];
                }
            }
        }

        assert_eq![100, seen.len()];
    }

    #[test]
    fn more_than_65536_sequenced_packets() {
        let (mut server, mut client, _) = create_server_client_network();
        // acknowledge the client
        server
            .send(Packet::unreliable(client_address(), vec![0]))
            .unwrap();

        let time = Instant::now();

        for id in 0..65536 + 100 {
            client
                .send(Packet::unreliable_sequenced(
                    server_address(),
                    id.to_string().as_bytes().to_vec(),
                    None,
                ))
                .unwrap();
            client.manual_poll(time);
            server.manual_poll(time);
        }

        let mut cnt = 0;
        while let Some(message) = server.recv() {
            match message {
                SocketEvent::Connect(_) => {}
                SocketEvent::Packet(_) => {
                    cnt += 1;
                }
                SocketEvent::Timeout(_) => {
                    panic!["This should not happen, as we've not advanced time"];
                }
            }
        }
        assert_eq![65536 + 100, cnt];
    }

    #[test]
    fn sequenced_packets_pathological_case() {
        let mut config = Config::default();
        config.max_packets_in_flight = 100;
        let (_, mut client) = create_server_client(config.clone());

        let time = Instant::now();

        for id in 0..101 {
            client
                .send(Packet::reliable_sequenced(
                    server_address(),
                    id.to_string().as_bytes().to_vec(),
                    None,
                ))
                .unwrap();
            client.manual_poll(time);

            while let Some(event) = client.recv() {
                match event {
                    SocketEvent::Timeout(remote_addr) => {
                        assert_eq![100, id];
                        assert_eq![remote_addr, server_address()];
                        return;
                    }
                    _ => {
                        panic!["No other event possible"];
                    }
                }
            }
        }

        panic!["Should have received a timeout event"];
    }

    #[test]
    fn manual_polling_socket() {
        let (mut server, mut client, _) = create_server_client_network();
        for _ in 0..3 {
            client
                .send(Packet::unreliable(
                    server_address(),
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                ))
                .unwrap();
        }

        let time = Instant::now();

        client.manual_poll(time);
        server.manual_poll(time);

        assert!(server.recv().is_some());
        assert!(server.recv().is_some());
        assert!(server.recv().is_some());
    }

    #[test]
    fn can_send_and_receive() {
        let (mut server, mut client, _) = create_server_client_network();
        for _ in 0..3 {
            client
                .send(Packet::unreliable(
                    server_address(),
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                ))
                .unwrap();
        }

        let now = Instant::now();
        client.manual_poll(now);
        server.manual_poll(now);

        assert!(server.recv().is_some());
        assert!(server.recv().is_some());
        assert!(server.recv().is_some());
    }

    #[test]
    fn connect_event_occurs() {
        let (mut server, mut client, _) = create_server_client_network();

        client
            .send(Packet::unreliable(server_address(), vec![0, 1, 2]))
            .unwrap();

        let now = Instant::now();
        client.manual_poll(now);
        server.manual_poll(now);

        assert_eq!(
            server.recv().unwrap(),
            SocketEvent::Connect(client_address())
        );
    }

    #[test]
    fn disconnect_event_occurs() {
        let mut config = Config::default();
        config.idle_connection_timeout = Duration::from_millis(1);
        let (mut server, mut client) = create_server_client(config.clone());

        client
            .send(Packet::unreliable(server_address(), vec![0, 1, 2]))
            .unwrap();

        let now = Instant::now();
        client.manual_poll(now);
        server.manual_poll(now);

        assert_eq!(
            server.recv().unwrap(),
            SocketEvent::Connect(client_address())
        );
        assert_eq!(
            server.recv().unwrap(),
            SocketEvent::Packet(Packet::unreliable(client_address(), vec![0, 1, 2]))
        );

        // acknowledge the client
        server
            .send(Packet::unreliable(client_address(), vec![]))
            .unwrap();

        server.manual_poll(now);
        client.manual_poll(now);

        // make sure the connection was successful on the client side
        assert_eq!(
            client.recv().unwrap(),
            SocketEvent::Packet(Packet::unreliable(server_address(), vec![]))
        );

        // give just enough time for no timeout events to occur (yet)
        server.manual_poll(now + config.idle_connection_timeout - Duration::from_millis(1));
        client.manual_poll(now + config.idle_connection_timeout - Duration::from_millis(1));

        assert_eq!(server.recv(), None);
        assert_eq!(client.recv(), None);

        // give enough time for timeouts to be detected
        server.manual_poll(now + config.idle_connection_timeout);
        client.manual_poll(now + config.idle_connection_timeout);

        assert_eq!(
            server.recv().unwrap(),
            SocketEvent::Timeout(client_address())
        );
        assert_eq!(
            client.recv().unwrap(),
            SocketEvent::Timeout(server_address())
        );
    }

    #[test]
    fn heartbeats_work() {
        let mut config = Config::default();
        config.idle_connection_timeout = Duration::from_millis(10);
        config.heartbeat_interval = Some(Duration::from_millis(4));
        let (mut server, mut client) = create_server_client(config.clone());
        // initiate a connection
        client
            .send(Packet::unreliable(server_address(), vec![0, 1, 2]))
            .unwrap();

        let now = Instant::now();
        client.manual_poll(now);
        server.manual_poll(now);

        // make sure the connection was successful on the server side
        assert_eq!(
            server.recv().unwrap(),
            SocketEvent::Connect(client_address())
        );
        assert_eq!(
            server.recv().unwrap(),
            SocketEvent::Packet(Packet::unreliable(client_address(), vec![0, 1, 2]))
        );

        // acknowledge the client
        // this way, the server also knows about the connection and sends heartbeats
        server
            .send(Packet::unreliable(client_address(), vec![]))
            .unwrap();

        server.manual_poll(now);
        client.manual_poll(now);

        // make sure the connection was successful on the client side
        assert_eq!(
            client.recv().unwrap(),
            SocketEvent::Packet(Packet::unreliable(server_address(), vec![]))
        );

        // give time to send heartbeats
        client.manual_poll(now + config.heartbeat_interval.unwrap());
        server.manual_poll(now + config.heartbeat_interval.unwrap());

        // give time for timeouts to occur if no heartbeats were sent
        client.manual_poll(now + config.idle_connection_timeout);
        server.manual_poll(now + config.idle_connection_timeout);

        // assert that no disconnection events occurred
        assert_eq!(client.recv(), None);
        assert_eq!(server.recv(), None);
    }

    #[test]
    fn multiple_sends_should_start_sending_dropped() {
        let (mut server, mut client, _) = create_server_client_network();

        let now = Instant::now();

        // send enough packets to ensure that we must have dropped packets.
        for i in 0..35 {
            client
                .send(Packet::unreliable(server_address(), vec![i]))
                .unwrap();
            client.manual_poll(now);
        }

        let mut events = Vec::new();

        loop {
            server.manual_poll(now);
            if let Some(event) = server.recv() {
                events.push(event);
            } else {
                break;
            }
        }

        // ensure that we get the correct number of events to the server.
        // 35 connect events plus the 35 messages
        assert_eq!(events.len(), 70);

        // finally the server decides to send us a message back. This necessarily will include
        // the ack information for 33 of the sent 35 packets.
        server
            .send(Packet::unreliable(client_address(), vec![0]))
            .unwrap();
        server.manual_poll(now);

        // loop to ensure that the client gets the server message before moving on
        loop {
            client.manual_poll(now);
            if client.recv().is_some() {
                break;
            }
        }

        // this next sent message should end up sending the 2 unacked messages plus the new messages
        // with payload 35
        events.clear();
        client
            .send(Packet::unreliable(server_address(), vec![35]))
            .unwrap();
        client.manual_poll(now);

        loop {
            server.manual_poll(now);
            if let Some(event) = server.recv() {
                events.push(event);
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
        assert_eq!(sent_events, vec![35]);
    }

    #[test]
    fn really_bad_network_keeps_chugging_along() {
        let (mut server, mut client, _) = create_server_client_network();

        let time = Instant::now();

        // we give both the server and the client a really bad bidirectional link
        let link_conditioner = {
            let mut lc = LinkConditioner::new();
            lc.set_packet_loss(0.9);
            Some(lc)
        };

        client.set_link_conditioner(link_conditioner.clone());
        server.set_link_conditioner(link_conditioner);

        let mut set = HashSet::new();

        // we chat 100 packets between the client and server, which will re-send any non-acked
        // packets
        let mut send_many_packets = |dummy: Option<u8>| {
            for id in 0..100 {
                client
                    .send(Packet::reliable_unordered(
                        server_address(),
                        vec![dummy.unwrap_or(id)],
                    ))
                    .unwrap();

                server
                    .send(Packet::reliable_unordered(client_address(), vec![255]))
                    .unwrap();

                client.manual_poll(time);
                server.manual_poll(time);

                while let Some(_) = client.recv() {}
                while let Some(event) = server.recv() {
                    match event {
                        SocketEvent::Packet(pkt) => {
                            set.insert(pkt.payload()[0]);
                        }
                        SocketEvent::Timeout(_) => {
                            panic!["Unable to time out, time has not advanced"]
                        }
                        SocketEvent::Connect(_) => {}
                    }
                }
            }

            set.len()
        };

        // the first chatting sequence sends packets 0..100 from the client to the server. After
        // this we just chat with a value of 255 so we don't accidentally overlap those chatting
        // packets with the packets we want to ack.
        send_many_packets(None);
        send_many_packets(Some(255));
        send_many_packets(Some(255));
        send_many_packets(Some(255));

        // 101 because we have 0..100 and 255 from the dummies
        assert_eq![101, send_many_packets(Some(255))];
    }

    #[test]
    fn fragmented_ordered_gets_acked() {
        let mut config = Config::default();
        config.fragment_size = 10;
        let (mut server, mut client) = create_server_client(config.clone());

        let time = Instant::now();
        let dummy = vec![0];

        // ---

        client
            .send(Packet::unreliable(server_address(), dummy.clone()))
            .unwrap();
        client.manual_poll(time);
        server
            .send(Packet::unreliable(client_address(), dummy.clone()))
            .unwrap();
        server.manual_poll(time);

        // ---

        let exceeds = b"Fragmented string".to_vec();
        client
            .send(Packet::reliable_ordered(server_address(), exceeds, None))
            .unwrap();
        client.manual_poll(time);

        server.manual_poll(time);
        server.manual_poll(time);
        server
            .send(Packet::reliable_ordered(
                client_address(),
                dummy.clone(),
                None,
            ))
            .unwrap();

        client
            .send(Packet::unreliable(server_address(), dummy.clone()))
            .unwrap();
        client.manual_poll(time);
        server.manual_poll(time);

        for _ in 0..4 {
            assert![server.recv().is_some()];
        }
        assert![server.recv().is_none()];

        for _ in 0..34 {
            client
                .send(Packet::reliable_ordered(
                    server_address(),
                    dummy.clone(),
                    None,
                ))
                .unwrap();
            client.manual_poll(time);
            server
                .send(Packet::reliable_ordered(
                    client_address(),
                    dummy.clone(),
                    None,
                ))
                .unwrap();
            server.manual_poll(time);
            assert![client.recv().is_some()];
            // If the last iteration returns `None` here, it indicates we just received a re-sent
            // fragment, because `manual_poll` only processes a single incoming UDP packet per
            // `manual_poll` if and only if the socket is in blocking mode.
            //
            // If that functionality is changed, we will receive something unexpected here
            match server.recv() {
                Some(SocketEvent::Packet(pkt)) => {
                    assert_eq![dummy, pkt.payload()];
                }
                _ => {
                    panic!["Did not receive expected dummy packet"];
                }
            }
        }
    }

    #[quickcheck_macros::quickcheck]
    fn do_not_panic_on_arbitrary_packets(bytes: Vec<u8>) {
        use crate::net::DatagramSocket;
        let network = NetworkEmulator::default();
        let mut server = FakeSocket::bind(&network, server_address(), Config::default()).unwrap();
        let mut client_socket = network.new_socket(client_address()).unwrap();

        client_socket
            .send_packet(&server_address(), &bytes)
            .unwrap();

        let time = Instant::now();
        server.manual_poll(time);
    }
}
