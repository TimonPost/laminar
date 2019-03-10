mod common;

use crate::common::{ClientStub, ServerMoq};
use laminar::{Config, DeliveryMethod};
#[allow(unused_imports)]
use log::{debug, error, info};
use std::{
    sync::mpsc,
    time::{Duration, Instant},
};

const TOTAL_PACKETS_TO_SEND: u32 = 10_000;
const CLIENT_ADDR: &str = "127.0.0.1:12346";
const SERVER_ADDR: &str = "127.0.0.1:12345";

/// Test description:
/// 1. Setup receiving server.
/// 2. Send large packets so they need to be fragmented.
/// 3. Check if received data is correct.
#[test]
pub fn fragment_packet_integration_test() {
    let (tx, rx) = mpsc::channel();

    let test_data = vec![1; 4000];
    debug!("Test data created");
    let mut server = ServerMoq::new(Config::default(), SERVER_ADDR.parse().unwrap());
    debug!("ServerMoq created");
    let server_thread = server.start_receiving(rx, test_data.clone());
    debug!("Server thread has started receiving");
    let client = ClientStub::new(
        Duration::from_millis(0),
        CLIENT_ADDR.parse().unwrap(),
        TOTAL_PACKETS_TO_SEND,
        DeliveryMethod::ReliableUnordered,
    );
    debug!("Beginning stopwatch");
    let stopwatch = Instant::now();
    debug!("Adding client");
    server
        .add_client(test_data.to_vec(), client)
        .join()
        .unwrap();
    // notify server to stop receiving.
    debug!("Sending cancellation message");
    match tx.send(true) {
        Ok(_) => {
            debug!("Server stopped");
            let _total_received = server_thread.join().unwrap();
            let _elapsed_time = stopwatch.elapsed();
        },
        Err(e) => {
            error!("Error sending cancellation message: {:?}", e);
        }
    }
}
