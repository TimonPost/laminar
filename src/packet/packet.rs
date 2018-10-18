use std::net::SocketAddr;
use infrastructure::DeliveryMethod;

#[derive(Clone, PartialEq, Eq, Debug)]
/// This is a user friendly packet containing the payload from the packet and the endpoint from where it came.
pub struct Packet {
    // the endpoint from where it came
    addr: SocketAddr,
    // the raw payload of the packet
    payload: Box<[u8]>,
    // defines on how the packet will be delivered.
    delivery_method: DeliveryMethod,
}

impl Packet {
    /// Create an new packet by passing the receiver, data and how this packet should be delivered.
    pub fn new(addr: SocketAddr, payload: Vec<u8>, delivery_method: DeliveryMethod) -> Self {
        Packet {
            addr,
            payload: payload.into_boxed_slice(),
            delivery_method,
        }
    }

    /// This will create an unreliable packet.
    ///
    /// *Details*
    ///
    ///  1. Unreliable
    ///  2. No guarantee for delivery.
    ///  3. No guarantee for order.
    ///  4. No way of getting dropped packet
    ///  5. Duplication possible
    ///
    /// Basically just bare UDP
    pub fn unreliable(addr: SocketAddr, payload: Vec<u8>) -> Packet {
        Packet::new(addr,payload,DeliveryMethod::Unreliable)
    }

    /// This will create an reliable unordered packet.
    ///
    /// *Details*
    ///
    ///  1. Reliable.
    ///  2. Guarantee of delivery.
    ///  3. No guarantee for order.
    ///  4. Packets will not be dropped.
    ///  5. Duplication not possible
    ///
    /// Basically this is almost TCP like without ordering of packets.
    pub fn reliable_unordered(addr: SocketAddr, payload: Vec<u8>) -> Packet {
        Packet::new(addr,payload,DeliveryMethod::ReliableUnordered)
    }

    /// This will create an reliable unordered packet.
    ///
    /// *Details*
    ///
    ///  1. Reliable.
    ///  2. Guarantee of delivery.
    ///  3. Guarantee for order.
    ///  4. Packets will not be dropped.
    ///  5. Duplication not possible
    ///
    /// Basically this is almost TCP like with ordering of packets.
    pub fn reliable_ordered(addr: SocketAddr, payload: Vec<u8>) -> Packet {
        Packet::new(addr,payload,DeliveryMethod::ReliableOrdered)
    }

    /// This will create an reliable ordered packet.
    ///
    /// *Details*
    ///
    ///  1. Unreliable.
    ///  2. No guarantee of delivery.
    ///  3. Guarantee for order.
    ///  4. Packets can be dropped but you will be able to retrieve dropped packets.
    ///  5. Duplication not possible
    ///
    /// Basically this is UDP with the ability to retrieve dropped packets by acknowledgements.
    pub fn sequenced_ordered(addr: SocketAddr, payload: Vec<u8>) -> Packet {
        Packet::new(addr,payload,DeliveryMethod::SequencedOrdered)
    }

    /// This will create an reliable ordered packet.
    ///
    /// *Details*
    ///
    ///  1. Unreliable.
    ///  2. No guarantee of delivery.
    ///  3. No Guarantee for order.
    ///  4. Packets can be dropped but you will be able to retrieve dropped packets.
    ///  5. Duplication not possible
    ///
    /// Basically this is UDP with the ability to retrieve dropped packets by acknowledgements.
    pub fn sequenced_unordered(addr: SocketAddr, payload: Vec<u8>) -> Packet {
        Packet::new(addr,payload,DeliveryMethod::SequencedUnordered)
    }

    /// Get the payload (raw data) of this packet.
    pub fn payload(&self) -> &[u8] {
        return &self.payload;
    }

    /// Get the endpoint from this packet.
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// Get the type representing on how this packet will be delivered.
    pub fn delivery_method(&self) -> DeliveryMethod {
        self.delivery_method
    }
}
