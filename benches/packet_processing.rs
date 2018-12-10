use std::sync::Arc;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use laminar::{net::VirtualConnection, DeliveryMethod, NetworkConfig, ProtocolVersion};

use criterion::{criterion_group, criterion_main, Criterion};

const SERVER_ADDR: &str = "127.0.0.1:12345";
const CLIENT_ADDR: &str = "127.0.0.1:12346";

fn process_packet_before_send(
    connection: &mut VirtualConnection,
    _config: &NetworkConfig,
    delivery_method: DeliveryMethod,
) {
    let payload = vec![1, 2, 3, 4, 5];

    let _packet_data = connection
        .process_outgoing(&payload, delivery_method)
        .unwrap();
}

fn send_unreliable_benchmark(c: &mut Criterion) {
    let config = NetworkConfig::default();
    let mut connection = VirtualConnection::new(
        SERVER_ADDR.parse().unwrap(),
        Arc::new(NetworkConfig::default()),
    );

    c.bench_function("process unreliable before send", move |b| {
        b.iter(|| {
            process_packet_before_send(
                &mut connection,
                &config,
                DeliveryMethod::UnreliableUnordered,
            )
        })
    });
}

fn send_reliable_benchmark(c: &mut Criterion) {
    let config = NetworkConfig::default();
    let mut connection = VirtualConnection::new(
        SERVER_ADDR.parse().unwrap(),
        Arc::new(NetworkConfig::default()),
    );

    c.bench_function("process reliable before send", move |b| {
        b.iter(|| {
            process_packet_before_send(&mut connection, &config, DeliveryMethod::ReliableUnordered)
        })
    });
}

fn process_packet_when_received(connection: &mut VirtualConnection, data: &[u8]) {
    connection.process_incoming(&data).unwrap().unwrap();
}

/// This is mimicking the `HeaderParser for StandardHeader` implementation which is no longer
/// visible externally
fn standard_header_bytes(delivery_method: DeliveryMethod) -> Vec<u8> {
    let mut buffer = Vec::new();
    buffer.write_u32::<BigEndian>(ProtocolVersion::get_crc32());
    // Represents a standard `Packet`
    buffer.write_u8(0);
    buffer.write_u8(delivery_method as u8);
    buffer
}

/// This is mimicking the `HeaderParser for AckedPacketHeader` implementation which is no longer
/// visible externally
fn acked_header_bytes(
    delivery_method: DeliveryMethod,
    seq: u16,
    ack_seq: u16,
    ack_field: u32,
) -> Vec<u8> {
    let mut buffer = standard_header_bytes(delivery_method);
    buffer.write_u16::<BigEndian>(seq);
    buffer.write_u16::<BigEndian>(ack_seq);
    buffer.write_u32::<BigEndian>(ack_field);
    buffer
}

fn receive_unreliable_benchmark(c: &mut Criterion) {
    let mut connection = VirtualConnection::new(
        SERVER_ADDR.parse().unwrap(),
        Arc::new(NetworkConfig::default()),
    );

    // setup fake received bytes.
    let mut buffer = standard_header_bytes(DeliveryMethod::UnreliableUnordered);
    buffer.append(&mut vec![1; 500]);

    c.bench_function("process unreliable packet on receive", move |b| {
        b.iter(|| process_packet_when_received(&mut connection, &buffer))
    });
}

fn receive_reliable_benchmark(c: &mut Criterion) {
    let mut connection = VirtualConnection::new(
        SERVER_ADDR.parse().unwrap(),
        Arc::new(NetworkConfig::default()),
    );

    // setup fake received bytes.
    let mut buffer = acked_header_bytes(DeliveryMethod::ReliableUnordered, 0, 1, 2);
    buffer.append(&mut vec![1; 500]);

    c.bench_function("process reliable packet on receive", move |b| {
        b.iter(|| process_packet_when_received(&mut connection, &buffer))
    });
}

criterion_group!(
    benches,
    send_reliable_benchmark,
    send_unreliable_benchmark,
    receive_unreliable_benchmark,
    receive_reliable_benchmark
);
criterion_main!(benches);
