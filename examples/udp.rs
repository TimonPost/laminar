//! This module provides examples for the TCP api.
//! 1. sending data
//! 2. receiving data
//! 3. constructing the packet for sending.

extern crate laminar;

use laminar::net::{NetworkConfig, SocketAddr, UdpSocket};
use laminar::packet::Packet;

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
pub fn send_data() {
    // you can change the config but if you want just go for the default.
    let config = NetworkConfig::default();

    // setup an udp socket and bind it to the client address.
    let mut udp_socket = UdpSocket::bind(client_address(), config).unwrap();

    let packet = construct_packet();

    // next send or packet to the endpoint we earlier putted into the packet.
    udp_socket.send(packet);
}

/// This is an example of how to receive data over udp on an specific socket address with blocking the current thread.
pub fn receive_data_with_blocking() {
    // you can change the config but if you want just go for the default.
    let config = NetworkConfig::default();

    // setup an udp socket and bind it to the client address.
    let mut udp_socket: UdpSocket = UdpSocket::bind(server_address(), config).unwrap();

    // next we could specify if or socket should block the current thread when receiving data or not (default = false)
    udp_socket.set_nonblocking(false);

    // Next start receiving.
    let result = udp_socket.recv();

    match result {
        Ok(Some(packet)) => {
            let endpoint: SocketAddr = packet.addr();
            let received_data: &[u8] = packet.payload();

            // you can here deserialize your bytes into the data you have passed it when sending.

            println!(
                "Received packet from: {:?} with length {}",
                endpoint,
                received_data.len()
            );
        }
        Ok(None) => {
            println!("This could happen when we have'n received all data from this packet yet");
        }
        Err(e) => {
            println!("Something went wrong when receiving, error: {:?}", e);
        }
    }
}

/// This is an example of how to receive data over udp on an specific socket address without blocking the current thread.
pub fn receive_data_without_blocking() {
    // you can change the config but if you want just go for the default.
    let config = NetworkConfig::default();

    // setup an udp socket and bind it to the client address.
    let mut udp_socket: UdpSocket = UdpSocket::bind(client_address(), config).unwrap();

    // next we could specify if or socket should block the current thread when receiving data or not (default = false)
    udp_socket.set_nonblocking(false);

    // setup a thread to do the receiving
    // Next start receiving.
    let result = udp_socket.recv();

    match result {
        Ok(Some(packet)) => {
            let endpoint: SocketAddr = packet.addr();
            let received_data: &[u8] = packet.payload();

            // you can here deserialize your bytes into the data you have passed it when sending.

            println!(
                "Received packet from: {:?} with length {}",
                endpoint,
                received_data.len()
            );
        }
        Ok(None) => {
            println!("This could happen when we have'n received all data from this packet yet");
        }
        Err(e) => {
            // We get an error if receiving would block the thread.
            println!("Something went wrong when receiving, error: {:?}", e);
        }
    }
}

/// This is an example of how to construct an packet.
pub fn construct_packet() -> Packet {
    // this is the destination address of the packet.
    let destination: SocketAddr = server_address();

    // lets construct some payload (raw data) for or packet.
    let raw_data = "example data".as_bytes();

    // lets construct or packet by passing in the destination for this packet and the bytes needed to be send..
    let packet: Packet = Packet::sequenced_unordered(destination, raw_data.to_owned());

    packet
}

// TODO: Use functions in example
fn main() {}
