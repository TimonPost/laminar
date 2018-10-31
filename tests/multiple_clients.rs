extern crate laminar;

mod common;

use std::sync::mpsc;
use std::time::{Duration, Instant};

use laminar::infrastructure::DeliveryMethod;
use laminar::net::NetworkConfig;

use common::{ClientStub, ServerMoq};

const TOTAL_PACKETS_TO_SEND: u32 = 500;
const SERVER_ADDR: &str = "127.0.0.1:12345";

/// Test description:
/// 1. Setup receiving server.
/// 2. Setup multiple clients.
/// 3. Send x packets to server with clients.
/// 3. Validate received data.
#[test]
pub fn multiple_client_integration_test() {
    let (tx, rx) = mpsc::channel();

    let test_data = "Test Data!".as_bytes();

    // setup the server and start receiving.
    let mut server = ServerMoq::new(NetworkConfig::default(), true, SERVER_ADDR.parse().unwrap());
    let server_thread = server.start_receiving(rx, test_data.to_vec());

    // the packet rate at which clients send data.
    let sixteenth_a_second = Duration::from_millis(16);

    // create client stubs.
    let mut clients = Vec::new();
    clients.push(ClientStub::new(
        sixteenth_a_second,
        "127.0.0.1:12346".parse().unwrap(),
        TOTAL_PACKETS_TO_SEND,
        DeliveryMethod::UnreliableUnordered
    ));
    clients.push(ClientStub::new(
        sixteenth_a_second,
        "127.0.0.1:12347".parse().unwrap(),
        TOTAL_PACKETS_TO_SEND,
        DeliveryMethod::UnreliableUnordered
    ));
    clients.push(ClientStub::new(
        sixteenth_a_second,
        "127.0.0.1:12348".parse().unwrap(),
        TOTAL_PACKETS_TO_SEND,
        DeliveryMethod::UnreliableUnordered
    ));
    clients.push(ClientStub::new(
        sixteenth_a_second,
        "127.0.0.1:12349".parse().unwrap(),
        TOTAL_PACKETS_TO_SEND,
        DeliveryMethod::UnreliableUnordered
    ));
    clients.push(ClientStub::new(
        sixteenth_a_second,
        "127.0.0.1:12350".parse().unwrap(),
        TOTAL_PACKETS_TO_SEND,
        DeliveryMethod::UnreliableUnordered
    ));
    clients.push(ClientStub::new(
        sixteenth_a_second,
        "127.0.0.1:12351".parse().unwrap(),
        TOTAL_PACKETS_TO_SEND,
        DeliveryMethod::UnreliableUnordered
    ));

    let stopwatch = Instant::now();

    let mut handles = Vec::new();

    // start all clients
    for client in clients {
        handles.push(server.add_client(test_data.to_vec(), client));
    }

    // wait for clients to send data
    for handle in handles {
        handle.join().unwrap();
    }

    // notify server to stop receiving.
    tx.send(true).unwrap();

    let _total_received = server_thread.join().unwrap();
    let _elapsed_time = stopwatch.elapsed();
}
