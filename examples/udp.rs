//! This module provides examples for the UDP api.
//! 1. sending data
//! 2. receiving data
//! 3. constructing the packet for sending.
use laminar::{Packet, Result, Socket, SocketEvent};

use std::net::SocketAddr;

/// The socket address of where the server is located.
const SERVER_ADDR: &'static str = "127.0.0.1:12345";
// The client address from where the data is sent.
const CLIENT_ADDR: &'static str = "127.0.0.1:12346";

fn client_address() -> SocketAddr {
    CLIENT_ADDR.parse().unwrap()
}

fn server_address() -> SocketAddr {
    SERVER_ADDR.parse().unwrap()
}

/// This is an example of how to send data to an specific address.
pub fn send_data() -> Result<()> {
    // Setup a udp socket and bind it to the client address.
    let socket = Socket::bind(client_address()).unwrap();

    let packet = construct_packet();

    // next send or packet to the endpoint we earlier putted into the packet.
    socket.send(packet)
}

/// This is an example of how to receive data over udp.
pub fn receive_data() {
    // setup an udp socket and bind it to the client address.
    let socket = Socket::bind(server_address()).unwrap();

    // Next start receiving.
    loop {
        if let Some(result) = socket.recv() {
            match result {
                SocketEvent::Packet(packet) => {
                    let endpoint: SocketAddr = packet.addr();
                    let received_data: &[u8] = packet.payload();

                    // you can here deserialize your bytes into the data you have passed it when sending.

                    println!(
                        "Received packet from: {:?} with length {}",
                        endpoint,
                        received_data.len()
                    );
                }
                _ => {}
            }
            break;
        }
    }
}

/// This is an example of how to construct a packet.
pub fn construct_packet() -> Packet {
    // this is the destination address of the packet.
    let destination: SocketAddr = server_address();

    // lets construct some payload (raw data) for or packet.
    let raw_data = "example data".as_bytes();

    // lets construct or packet by passing in the destination for this packet and the bytes needed to be send..
    let packet: Packet = Packet::reliable_unordered(destination, raw_data.to_owned());

    packet
}

// TODO: Use functions in example
fn main() {}
