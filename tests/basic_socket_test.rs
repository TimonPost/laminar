use std::{collections::HashSet, net::SocketAddr, time::Instant};

use laminar::{Config, Packet, Socket, SocketEvent};
#[cfg(feature = "tester")]
use laminar::LinkConditioner;

#[test]
fn binding_to_any() {
    // bind to 10 different addresses
    let sock_without_config = (0..5).map(|_| Socket::bind_any());
    let sock_with_config = (0..5).map(|_| Socket::bind_any_with_config(Config::default()));

    let valid_socks: Vec<_> = sock_without_config
        .chain(sock_with_config)
        .filter_map(|sock| sock.ok())
        .collect();
    assert_eq!(valid_socks.len(), 10);

    let unique_addresses: HashSet<_> = valid_socks
        .into_iter()
        .map(|sock| sock.local_addr().unwrap())
        .collect();
    assert_eq!(unique_addresses.len(), 10);
}

#[test]
fn blocking_sender_and_receiver() {
    let cfg = Config::default();

    let mut client = Socket::bind_any_with_config(cfg.clone()).unwrap();
    let mut server = Socket::bind_any_with_config(Config {
        blocking_mode: true,
        ..cfg
    })
    .unwrap();

    let server_addr = server.local_addr().unwrap();
    let client_addr = client.local_addr().unwrap();

    let time = Instant::now();

    client
        .send(Packet::unreliable(server_addr, b"Hello world!".to_vec()))
        .unwrap();

    client.manual_poll(time);
    server.manual_poll(time);

    assert_eq![SocketEvent::Connect(client_addr), server.recv().unwrap()];
    if let SocketEvent::Packet(packet) = server.recv().unwrap() {
        assert_eq![b"Hello world!", packet.payload()];
    } else {
        panic!["Did not receive a packet when it should"];
    }
}

#[test]
fn local_addr() {
    let port = 40000;
    let socket =
        Socket::bind(format!("127.0.0.1:{}", port).parse::<SocketAddr>().unwrap()).unwrap();
    assert_eq!(port, socket.local_addr().unwrap().port());
}

#[test]
#[cfg(feature = "tester")]
fn use_link_conditioner() {
    let mut client = Socket::bind_any().unwrap();
    let mut server = Socket::bind_any().unwrap();

    let server_addr = server.local_addr().unwrap();

    let link_conditioner = {
        let mut lc = LinkConditioner::new();
        lc.set_packet_loss(1.0);
        Some(lc)
    };

    client.set_link_conditioner(link_conditioner);
    client
        .send(Packet::unreliable(server_addr, b"Hello world!".to_vec()))
        .unwrap();

    let time = Instant::now();
    client.manual_poll(time);
    server.manual_poll(time);

    assert_eq!(server.recv().is_none(), true);
}

#[test]
#[cfg(feature = "tester")]
fn poll_in_thread() {
    use std::thread;
    let mut server = Socket::bind_any().unwrap();
    let mut client = Socket::bind_any().unwrap();
    let server_addr = server.local_addr().unwrap();

    // get sender and receiver from server, and start polling in separate thread
    let (sender, receiver) = (server.get_packet_sender(), server.get_event_receiver());
    let _thread = thread::spawn(move || server.start_polling());

    // server will responde to this
    client
        .send(Packet::reliable_unordered(server_addr, b"Hello!".to_vec()))
        .expect("This should send");
    // this will break the loop
    client
        .send(Packet::reliable_unordered(server_addr, b"Bye!".to_vec()))
        .expect("This should send");
    client.manual_poll(Instant::now());

    // listen for received server messages, and break when "Bye!" is received.
    loop {
        if let Ok(event) = receiver.recv() {
            if let SocketEvent::Packet(packet) = event {
                let msg = packet.payload();

                if msg == b"Bye!" {
                    break;
                }

                sender
                    .send(Packet::reliable_unordered(
                        packet.addr(),
                        b"Hi, there!".to_vec(),
                    ))
                    .expect("This should send");
            }
        }
    }
    // loop until we get response from server.
    loop {
        client.manual_poll(Instant::now());
        if let Some(packet) = client.recv() {
            assert_eq!(
                packet,
                SocketEvent::Packet(Packet::reliable_unordered(
                    server_addr,
                    b"Hi, there!".to_vec()
                ))
            );
            break;
        }
    }
}
