#![feature(test)]
extern crate laminar;
extern crate test;
#[macro_use]
extern crate criterion;

use self::laminar::net::{NetworkConfig, SocketState};
use self::laminar::packet::header::{FragmentHeader, HeaderParser, HeaderReader, PacketHeader};
use self::laminar::packet::{Packet, PacketProcessor};

use self::criterion::Criterion;
use test::Bencher;

use std::net::SocketAddr;
use std::thread;

const SERVER_ADDR: &str = "127.0.0.1:12345";
const CLIENT_ADDR: &str = "127.0.0.1:12346";

fn process_packet_before_send(socket_state: &mut SocketState, config: &NetworkConfig) {
    let payload = vec![1, 2, 3, 4, 5];
    let packet = Packet::new("127.0.0.1:12346".parse().unwrap(), payload);

    let (addr, mut packet_data) = socket_state.pre_process_packet(packet, &config).unwrap();
}

fn send_benchmark(c: &mut Criterion) {
    let config = NetworkConfig::default();
    let mut socket_state = SocketState::new().unwrap();

    c.bench_function("process send", move |b| {
        b.iter(|| process_packet_before_send(&mut socket_state, &config))
    });
}

fn process_packet_when_received(
    packet_processor: &mut PacketProcessor,
    data: &Vec<u8>,
    socket_state: &mut SocketState,
) {
    let packet = packet_processor
        .process_data(data.clone(), CLIENT_ADDR.parse().unwrap(), socket_state)
        .unwrap()
        .unwrap();
}

fn receive_benchmark(c: &mut Criterion) {
    let mut socket_state = SocketState::new().unwrap();
    let mut packet_processor = PacketProcessor::new(NetworkConfig::default());

    // setup fake received bytes.
    let packet_header = PacketHeader::new(0, 1, 2);
    let mut parsed = packet_header.parse().unwrap();
    parsed.append(&mut vec![1; 500]);

    c.bench_function("process received", move |b| {
        b.iter(|| process_packet_when_received(&mut packet_processor, &parsed, &mut socket_state))
    });
}

criterion_group!(benches, send_benchmark, receive_benchmark);
criterion_main!(benches);
