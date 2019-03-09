use laminar::{Packet, DeliveryMethod};
use super::{PacketFactory, PacketAsserting, client_addr};
use std::net::SocketAddr;
use byteorder::ReadBytesExt;
use byteorder::BigEndian;


pub struct Sequencing {
    addr: SocketAddr,
    delivery_guarantee: DeliveryMethod,
    last_received: u32,

}

impl Sequencing {
    pub fn new(addr: SocketAddr) -> Sequencing {
        Sequencing {
            addr,
            delivery_guarantee: DeliveryMethod::ReliableOrdered,
            last_received: 0
        }
    }
}

impl PacketAsserting  for Sequencing {
    fn assert_packet(&mut self, packet: Packet) {
        assert_eq!(packet.addr(), self.addr);
        assert_eq!(packet.delivery_method(), self.delivery_guarantee);

        let packet_identifier = packet.payload().read_u32::<BigEndian>().unwrap();

        if packet_identifier < self.last_received {
            panic!("Expected identifier: {} got {}; sequencing failed", self.last_received, packet_identifier);
        } else {
//            assert_eq!(packet.payload(), self.payoad);
            self.last_received = packet_identifier;
        }
    }
}

impl PacketFactory for Sequencing {
    fn new_packet(&self) -> Packet {
        let payload = vec![1,2,3];

        Packet::new(client_addr(), payload.into_boxed_slice(), self.delivery_guarantee)
    }
}