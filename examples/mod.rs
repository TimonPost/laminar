#[macro_use]
extern crate serde_derive;

mod simple_udp;
mod tcp;
mod udp;

use std::net::SocketAddr;

// Can be used to play around with the library run with cargo run --example playground.
pub fn main()
{
    simple_udp::run_simple_example();
//    udp::receive_data_with_blocking();
//    udp::receive_data_without_blocking();
}

/// The socket address of where the server is located.
const SERVER_ADDR: &'static str = "127.0.0.1:12345";
// The client address from where the data is sent.
const CLIENT_ADDR: &'static str = "127.0.0.1:12346";

fn client_address() -> SocketAddr
{
    CLIENT_ADDR.parse().unwrap()
}

fn server_address() -> SocketAddr
{
    SERVER_ADDR.parse().unwrap()
}