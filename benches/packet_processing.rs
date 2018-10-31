extern crate laminar;
#[macro_use]
extern crate criterion;

use self::laminar::net::{NetworkConfig, VirtualConnection};
use self::laminar::packet::header::{HeaderParser, HeaderReader, AckedPacketHeader, StandardHeader};
use self::laminar::packet::{Packet, PacketTypeId};
use laminar::infrastructure::DeliveryMethod;

use self::criterion::Criterion;
use std::net::SocketAddr;

const SERVER_ADDR: &str = "127.0.0.1:12345";
const CLIENT_ADDR: &str = "127.0.0.1:12346";

fn process_packet_before_send(connection: &mut VirtualConnection, config: &NetworkConfig, delivery_method: DeliveryMethod) {
    let payload = vec![1, 2, 3, 4, 5];

    let packet_data = connection.process_outgoing(&payload, delivery_method).unwrap();
}

fn send_unreliable_benchmark(c: &mut Criterion) {
    let config = NetworkConfig::default();
    let mut connection = VirtualConnection::new(SERVER_ADDR.parse().unwrap(), NetworkConfig::default());

    c.bench_function("process unreliable before send", move |b| {
        b.iter(|| process_packet_before_send(&mut connection, &config, DeliveryMethod::UnreliableUnordered))
    });
}

fn send_reliable_benchmark(c: &mut Criterion) {
    let config = NetworkConfig::default();
    let mut connection = VirtualConnection::new(SERVER_ADDR.parse().unwrap(), NetworkConfig::default());

    c.bench_function("process reliable before send", move |b| {
        b.iter(|| process_packet_before_send(&mut connection, &config, DeliveryMethod::ReliableUnordered))
    });
}

fn process_packet_when_received(
    connection: &mut VirtualConnection,
    data: &Vec<u8>,
) {
    let packet = connection
        .process_incoming(&data)
        .unwrap()
        .unwrap();
}

fn receive_unreliable_benchmark(c: &mut Criterion) {
    let mut connection = VirtualConnection::new(SERVER_ADDR.parse().unwrap(), NetworkConfig::default());

    // setup fake received bytes.
    let packet_header = StandardHeader::new(DeliveryMethod::UnreliableUnordered, PacketTypeId::Packet);

    let mut buffer = Vec::with_capacity(packet_header.size() as usize);
    packet_header.parse(&mut buffer).unwrap();
    buffer.append(&mut vec![1; 500]);

    c.bench_function("process unreliable packet on receive", move |b| {
        b.iter(|| process_packet_when_received(&mut connection, &buffer))
    });
}

fn receive_reliable_benchmark(c: &mut Criterion) {
    let mut connection = VirtualConnection::new(SERVER_ADDR.parse().unwrap(), NetworkConfig::default());

    // setup fake received bytes.
    let packet_header = AckedPacketHeader::new(StandardHeader::new(DeliveryMethod::ReliableUnordered, PacketTypeId::Packet),0, 1, 2);

    let mut buffer = Vec::with_capacity(packet_header.size() as usize);
    packet_header.parse(&mut buffer).unwrap();
    buffer.append(&mut vec![1; 500]);

    c.bench_function("process reliable packet on receive", move |b| {
        b.iter(|| process_packet_when_received(&mut connection, &buffer))
    });
}

criterion_group!(benches, send_reliable_benchmark, send_unreliable_benchmark, receive_reliable_benchmark, receive_unreliable_benchmark);
criterion_main!(benches);
