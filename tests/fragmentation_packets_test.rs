#[cfg(feature = "tester")]
use std::{thread, time::Duration};
#[cfg(feature = "tester")]
use std::net::SocketAddr;

#[cfg(feature = "tester")]
use log::debug;

#[cfg(feature = "tester")]
use common::{Client, client_addr, Server, ServerEvent};
#[cfg(feature = "tester")]
use laminar::{DeliveryGuarantee, OrderingGuarantee, Packet};

#[cfg(feature = "tester")]
mod common;

#[test]
#[cfg(feature = "tester")]
fn send_receive_fragment_packets() {
    let listen_addr: SocketAddr = "127.0.0.1:12346".parse().unwrap();
    let client_addr = client_addr();

    let server = Server::new(listen_addr);

    let client = Client::new(Duration::from_millis(1), 5000);

    let assert_function = move |packet: Packet| {
        assert_eq!(packet.order_guarantee(), OrderingGuarantee::None);
        assert_eq!(packet.delivery_guarantee(), DeliveryGuarantee::Reliable);
        assert_eq!(packet.payload(), payload().as_slice());
    };

    let packet_factory = move || -> Packet { Packet::reliable_unordered(listen_addr, payload()) };

    let server_handle = server.start_receiving(assert_function);

    client
        .run_instance(packet_factory, client_addr)
        .wait_until_finished();

    // give the server time to process all packets.
    thread::sleep(Duration::from_millis(500));

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
            _ => debug!("Not handled!"),
        }
    }

    server_handle.wait_until_finished();
}

pub fn payload() -> Vec<u8> {
    vec![0; 4000]
}
