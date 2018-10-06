#[macro_use]
extern crate serde_derive;

/// This is an simple example to demonstrate how to send some data over the network with the udp.
mod simple_udp;

mod tcp;

/// Some examples of how to use the UDPSocket Api.
/// 1. sending data
/// 2. receiving data
/// 3. constructing the packet for sending.
mod udp;

use std::net::SocketAddr;

/// The socket address of where the server is located.
const SERVER_ADDR: &'static str = "127.0.0.1:12345";
// The client address from where the data is sent.
const CLIENT_ADDR: &'static str = "127.0.0.1:12346";

// Can be used to play around with the library run with cargo run --example playground.
pub fn main()
{
    simple_udp::run_simple_example();
}

fn client_address() -> SocketAddr
{
    CLIENT_ADDR.parse().unwrap()
}

fn server_address() -> SocketAddr
{
    SERVER_ADDR.parse().unwrap()
}