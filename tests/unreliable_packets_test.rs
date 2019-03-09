#[cfg(feature = "tester")]
mod common;

#[cfg(feature = "tester")]
use common::{client_addr, server_addr, Client, Server, ServerEvent};

use laminar::{DeliveryMethod, Packet};
use log::{debug, error};
use std::{thread, time::Duration};

#[test]
#[cfg(feature = "tester")]
fn send_receive_unreliable_packets() {
    let server = Server::new();
    let client_addr = client_addr();
    let client = Client::new(Duration::from_millis(1), client_addr, 5000);

    let assert_function = move |packet: Packet| {
        assert_eq!(packet.addr(), client_addr);
        assert_eq!(
            packet.delivery_method(),
            DeliveryMethod::UnreliableUnordered
        );
        assert_eq!(packet.payload(), payload().as_slice());
    };

    let packet_factory = || -> Packet { Packet::unreliable(server_addr(), payload()) };

    let server_handle = server.start_receiving(assert_function);

    client.run_instance(packet_factory).wait_until_finished();

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
            _ => error!("Not handled!"),
        }
    }

    server_handle.wait_until_finished();
}

pub fn payload() -> Vec<u8> {
    vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
}
