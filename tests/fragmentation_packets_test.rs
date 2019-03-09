#[cfg(feature = "tester")]
mod common;
#[cfg(feature = "tester")]
use common::{client_addr, server_addr, Client, Server, ServerEvent};

use laminar::{DeliveryMethod, Packet};
use log::debug;
use std::{thread, time::Duration};

#[test]
#[cfg(feature = "tester")]
fn send_receive_fragment_packets() {
    let server = Server::new();
    let client_addr = client_addr();
    let client = Client::new(Duration::from_millis(1), client_addr, 10000);

    let assert_function = move |packet: Packet| {
        assert_eq!(packet.addr(), client_addr);
        assert_eq!(packet.delivery_method(), DeliveryMethod::ReliableUnordered);
        assert_eq!(packet.payload(), payload().as_slice());
    };

    let packet_factory = || -> Packet { Packet::reliable_unordered(server_addr(), payload()) };

    let server_handle = server.start_receiving(assert_function);

    client.run_instance(packet_factory).wait_until_finished();

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
