#[cfg(feature = "tester")]
use std::net::SocketAddr;
#[cfg(feature = "tester")]
use std::{thread, time::Duration};

#[cfg(feature = "tester")]
use log::{debug, error, info};

#[cfg(feature = "tester")]
use common::{client_addr, Client, Server, ServerEvent};
#[cfg(feature = "tester")]
use laminar::{DeliveryGuarantee, OrderingGuarantee, Packet};

#[cfg(feature = "tester")]
mod common;

#[test]
#[cfg(feature = "tester")]
fn send_receive_unreliable_packets() {
    let client_addr = client_addr();
    let listen_addr: SocketAddr = "127.0.0.1:12346".parse().unwrap();
    let server = Server::new(listen_addr);

    let client = Client::new(Duration::from_millis(1), 5000);

    let assert_function = move |packet: Packet| {
        assert_eq!(packet.order_guarantee(), OrderingGuarantee::None);
        assert_eq!(packet.delivery_guarantee(), DeliveryGuarantee::Unreliable);
        assert_eq!(packet.payload(), payload().as_slice());
    };

    let packet_factory = move || -> Packet { Packet::unreliable(listen_addr, payload()) };

    let server_handle = server.start_receiving(assert_function);

    client
        .run_instance(packet_factory, client_addr)
        .wait_until_finished();

    // give the server time to process all packets.
    thread::sleep(Duration::from_millis(200));

    server_handle.shutdown();

    for event in server_handle.iter_events().collect::<Vec<ServerEvent>>() {
        match event {
            ServerEvent::Throughput(throughput) => {
                debug!("Throughput: {}", throughput);
            }
            ServerEvent::AverageThroughput(avg_throughput) => {
                debug!("Avg. Throughput: {}", avg_throughput);
            }
            ServerEvent::TotalSent(total) => {
                debug!("Total Packets Received {}", total);
            }
            ServerEvent::SocketEvent(event) => {
                info!("Socket Event: {:?}", event);
            }
        }
    }

    server_handle.wait_until_finished();
}

#[test]
#[cfg(feature = "tester")]
fn send_receive_unreliable_packets_muliple_clients() {
    let listen_addr: SocketAddr = "127.0.0.1:12345".parse().unwrap();
    let server = Server::new(listen_addr);
    let client = Client::new(Duration::from_millis(16), 500);

    let assert_function = move |packet: Packet| {
        assert_eq!(packet.order_guarantee(), OrderingGuarantee::None);
        assert_eq!(packet.delivery_guarantee(), DeliveryGuarantee::Unreliable);
        assert_eq!(packet.payload(), payload().as_slice());
    };

    let packet_factory = move || -> Packet { Packet::unreliable(listen_addr, payload()) };

    let server_handle = server.start_receiving(assert_function);

    let received = server_handle.event_receiver();

    let handle = thread::spawn(move || loop {
        match received.recv() {
            Ok(event) => {
                match event {
                    ServerEvent::Throughput(throughput) => {
                        info!("Throughput: {}", throughput);
                    }
                    ServerEvent::AverageThroughput(avg_throughput) => {
                        info!("Avg. Throughput: {}", avg_throughput);
                    }
                    ServerEvent::TotalSent(total) => {
                        info!("Total Received: {}", total);
                    }
                    ServerEvent::SocketEvent(event) => {
                        info!("Socket Event: {:?}", event);
                    }
                };
            }
            Err(_) => {
                error!("Stopped receiving events; closing event handler.");
                return;
            }
        }
    });

    let mut clients = Vec::new();

    for _ in 0..10 {
        clients.push(client.run_instance(packet_factory, client_addr()));
        info!("Client started.");
    }

    for client in clients {
        client.wait_until_finished();
        info!("Client finished.");
    }

    info!("Waiting 2 seconds");
    // give the server time to process all packets.
    thread::sleep(Duration::from_millis(2000));
    info!("Shutting down server!");
    server_handle.shutdown();
    server_handle.wait_until_finished();
    info!("Server is stopped");
    handle.join().unwrap();
}

pub fn payload() -> Vec<u8> {
    vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
}
