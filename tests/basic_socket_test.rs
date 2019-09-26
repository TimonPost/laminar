use laminar::{Config, Packet, Socket, SocketEvent};

use std::{collections::HashSet, net::SocketAddr, time::Instant};

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
