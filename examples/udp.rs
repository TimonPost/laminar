extern crate laminar;

use self::laminar::net::{UdpSocket, SocketAddr};
use self::laminar::packet::Packet;

use super::{client_address, server_address};

/// This is an example of how to send data to an specific address.
pub fn send_data()
{
    // setup an udp socket and bind it to the client address.
    let mut udp_socket = UdpSocket::bind(client_address()).unwrap();

    let packet = construct_packet();

    // next send or packet to the endpoint we earlier putted into the packet.
    udp_socket.send(packet);
}

/// This is an example of how to receive data over udp on an specific socket address with blocking the current thread.
pub fn receive_data_with_blocking()
{
    // setup an udp socket and bind it to the client address.
    let mut udp_socket: UdpSocket = UdpSocket::bind(server_address()).unwrap();

    // next we could specify if or socket should block the current thread when receiving data or not (default = true)
    udp_socket.set_blocking(true);

    // Next start receiving.
    let result= udp_socket.recv();

    match result {
        Ok(Some(packet)) => {
            let endpoint: SocketAddr = packet.addr;
            let received_data: Box<[u8]> = packet.payload;

            // you can here deserialize your bytes into the data you have passed it when sending.

            println!("Received packet from: {:?} with length {}", endpoint, received_data.len());
        },
        Ok(None) => {
            println!("This could happen when we have'n received all data from this packet yet");
        },
        Err(e) => {
            println!("Something went wrong when receiving, error: {:?}", e);
        }
    }
}

/// This is an example of how to receive data over udp on an specific socket address without blocking the current thread.
pub fn receive_data_without_blocking()
{
    // setup an udp socket and bind it to the client address.
    let mut udp_socket: UdpSocket = UdpSocket::bind(client_address()).unwrap();

    // next we could specify if or socket should block the current thread when receiving data or not (default = true)
    udp_socket.set_blocking(false);

    // setup a thread to do the receiving
    // Next start receiving.
    let result= udp_socket.recv();

    match result {
        Ok(Some(packet)) => {
            let endpoint: SocketAddr = packet.addr;
            let received_data: Box<[u8]> = packet.payload;

            // you can here deserialize your bytes into the data you have passed it when sending.

            println!("Received packet from: {:?} with length {}", endpoint, received_data.len());
        },
        Ok(None) => {
            println!("This could happen when we have'n received all data from this packet yet");
        },
        Err(e) => {
            // We get an error if receiving would block the thread.
            println!("Something went wrong when receiving, error: {:?}", e);
        }
    }
}

/// This is an example of how to construct an packet.
pub fn construct_packet() -> Packet
{
    // this is the destination address of the packet.
    let destination: SocketAddr = server_address();

    // lets construct some payload (raw data) for or packet.
    let raw_data = "example data".as_bytes();

    // lets construct or packet by passing in the destination for this packet and the bytes needed to be send..
    let packet: Packet = Packet::new(destination, raw_data.to_owned());

    packet
}
