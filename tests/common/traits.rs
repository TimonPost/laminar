use laminar::Packet;

pub trait PacketFactory {
    fn new_packet(&self) -> Packet;
}

pub trait PacketAsserting {
    fn assert_packet(&mut self, packet: Packet);
}