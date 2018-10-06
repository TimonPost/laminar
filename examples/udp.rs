extern crate laminar;

use self::laminar::net::{UdpSocket, SocketAddr};
use self::laminar::packet::Packet;

pub fn test()
{
    let socket_addr = "127.0.0.1:12345".parse().unwrap();

    let mut udp_socket = UdpSocket::bind(socket_addr).unwrap();
    udp_socket.recv();
    udp_socket.send()
}

pub fn constucting_packet()
{
    // this is the destination address of the packet.
    let destination = "127.0.0.1:12345".parse().unwrap();

    // lets construct some payload (raw data) for or packet.
    let raw_data = "example data".as_bytes();

    // lets construct or packet.
    let packet = Packet::new(destination, raw_data.to_owned());
}

fn
