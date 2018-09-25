use std::collections::HashMap;

use bincode::{deserialize, serialize};

use super::{Connection, Packet, SocketAddr, RawPacket};

/// This holds the 'virtual connections' currently (connected) to the udp socket.
pub struct SocketState  {
    connections: HashMap<SocketAddr, Connection>
}

impl SocketState {
    pub fn new() -> SocketState {
        SocketState { connections: HashMap::new() }
    }

    /// This will initialize the seq number, ack number and give back the raw data of the packet with the updated information.
    pub fn pre_process_packet(&mut self, packet: Packet) -> (SocketAddr, Vec<u8>) {
        let connection = self.create_connection_if_not_exists(&packet.addr);

        // queue new packet
        connection.waiting_packets.enqueue(connection.seq_num, packet.clone());

        // initialize packet data, seq, acked_seq etc.
        let raw_packet = RawPacket::new(connection.seq_num, &packet, connection);

        // increase sequence number
        connection.seq_num = connection.seq_num.wrapping_add(1);

        // TODO: remove unwrap
        let buffer = serialize(&raw_packet).unwrap();

        (packet.addr, buffer)
    }

    /// This will return all dropped packets from this connection.
    pub fn dropped_packets(&mut self, addr: SocketAddr) -> Vec<Packet> {
        let connection = self.create_connection_if_not_exists(&addr);
        connection.dropped_packets.drain(..).collect()
    }

    /// This will process an incoming packet and update acknowledgement information.
    pub fn process_received(&mut self, addr: SocketAddr, packet: &RawPacket) -> Packet {
        let mut connection = self.create_connection_if_not_exists(&addr);
        connection.their_acks.ack(packet.seq);

        // Update dropped packets if there are any.
        let dropped_packets = connection.waiting_packets.ack(packet.ack_seq, packet.ack_field);
        connection.dropped_packets = dropped_packets.into_iter().map(|(_, p)| p).collect();

        Packet { addr, payload: packet.payload.clone() }
    }

    #[inline]
    /// If there is no connection with the given socket address an new connection will be made.
    fn create_connection_if_not_exists(&mut self, addr: &SocketAddr) -> &mut Connection
    {
        self.connections.entry(*addr).or_insert(Connection::new())
    }
}