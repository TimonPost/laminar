use laminar::{Packet, DeliveryMethod};
use byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};

use super::{PacketFactory, PacketAsserting, client_addr};
use std::net::SocketAddr;

#[derive(Clone, Copy)]
pub struct Ordering {
    addr: SocketAddr,
    delivery_guarantee: DeliveryMethod,
    last_received: u32,
    current_packet_identifier: u32
}

impl Ordering {
    pub fn new(addr: SocketAddr) -> Ordering {
        Ordering {
            addr,
            delivery_guarantee: DeliveryMethod::ReliableOrdered,
            last_received: 0,
            current_packet_identifier: 0
        }
    }
}

impl PacketAsserting  for Ordering {
    fn assert_packet(&mut self, packet: Packet) {
        assert_eq!(packet.addr(), self.addr);
        assert_eq!(packet.delivery_method(), self.delivery_guarantee);

        let packet_identifier = packet.payload().read_u32::<BigEndian>().unwrap();

        if packet_identifier != self.last_received + 1 {
            panic!("Expected identifier: {} got {}; ordering failed", self.last_received, packet_identifier);
        } else {
//            assert_eq!(packet.payload(), self.payoad);
        }
    }
}

impl PacketFactory for Ordering {
    fn new_packet(&self) -> Packet {
        let mut payload = Vec::new();
        payload.write_u32::<BigEndian>(self.current_packet_identifier);
        payload.write_u16::<BigEndian>(2);
        payload.write_u16::<BigEndian>(3);

        Packet::new(client_addr(), payload.into_boxed_slice(), self.delivery_guarantee)
    }
}