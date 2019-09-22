//! This module provides examples for the UDP api.
//! 1. sending data
//! 2. receiving data
//! 3. constructing the packet for sending.
use laminar::{
    managers::SimpleConnectionManagerFactory, ConnectionEvent, Packet, ReceiveEvent, Result,
    SendEvent, Socket,
};

use std::net::SocketAddr;
use std::time::Instant;

/// The socket address of where the server is located.
const SERVER_ADDR: &str = "127.0.0.1:12345";
// The client address from where the data is sent.
const CLIENT_ADDR: &str = "127.0.0.1:12346";

fn client_address() -> SocketAddr {
    CLIENT_ADDR.parse().unwrap()
}

fn server_address() -> SocketAddr {
    SERVER_ADDR.parse().unwrap()
}

/// This is an example of how to send data to an specific address.
pub fn send_data(socket: &mut Socket) -> Result<()> {
    let (to_address, packet) = construct_packet();

    // next send or packet to the endpoint we earlier putted into the packet.
    socket.send(ConnectionEvent(to_address, SendEvent::Packet(packet)))?;

    // this function processes all events and actually sends or receives packets
    socket.manual_poll(Instant::now());
    Ok(())
}

/// This is an example of how to receive data over udp.
pub fn receive_data(socket: &mut Socket) {
    // this function processes all events and actually sends or receives packets
    socket.manual_poll(Instant::now());
    // Next start receiving.
    loop {
        if let Some(ConnectionEvent(_addr, ReceiveEvent::Packet(packet))) = socket.recv() {
            let endpoint: SocketAddr = packet.addr();
            let received_data: &[u8] = packet.payload();

            // you can here deserialize your bytes into the data you have passed it when sending.

            println!(
                "Received packet from: {:?} with length {}",
                endpoint,
                received_data.len()
            );
            break;
        }
    }
}

/// This is an example of how to construct a packet.
pub fn construct_packet() -> (SocketAddr, Packet) {
    // this is the destination address of the packet.
    let destination: SocketAddr = server_address();

    // lets construct some payload (raw data) for or packet.
    let raw_data = b"example data";

    // lets construct or packet by passing in the destination for this packet and the bytes needed to be send..
    let packet: Packet = Packet::reliable_unordered(destination, raw_data.to_vec());

    (destination, packet)
}

fn main() -> Result<()> {
    // Setup a udp socket and bind it to the client address.
    // SimpleSocketManager(true) provides connection that immediatelly goes to connected state, after any socket event is received
    let mut client = Socket::bind(
        client_address(),
        Box::new(SimpleConnectionManagerFactory(true)),
    )
    .unwrap();

    // setup an udp socket and bind it to the server address.
    let mut server = Socket::bind(
        server_address(),
        Box::new(SimpleConnectionManagerFactory(true)),
    )
    .unwrap();

    send_data(&mut client)?;
    receive_data(&mut server);
    Ok(())
}
