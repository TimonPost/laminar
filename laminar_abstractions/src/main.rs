extern crate laminar;
#[macro_use]
extern crate log;

mod server;
mod client;
mod net_events;
mod packet;

use client::Client;
use server::{Server, ServerConfig};
use std::time::Duration;
use std::net::SocketAddr;
use laminar::infrastructure::DeliveryMethod;

fn main() {
    let server_addr: SocketAddr =  "127.0.0.1:12345".parse().unwrap();
    let config = ServerConfig::default();

    let mut server = Server::new(config);
    server.run();

    let mut client = Client::new();
    client.send_tcp(b"Test data", server_addr);
    client.send_udp(b"Test data", server_addr, DeliveryMethod::Unreliable);
    client.send_udp(b"Test data", server_addr, DeliveryMethod::ReliableUnordered);
    client.send_udp(b"Test data", server_addr, DeliveryMethod::ReliableOrdered);
    client.send_udp(b"Test data", server_addr, DeliveryMethod::Unreliable);

    server.broad_cast_tcp(b"Test data", server_addr);
    server.broad_cast_upd(b"Test data", server_addr, DeliveryMethod::Unreliable);

    server.shutdown();
}
