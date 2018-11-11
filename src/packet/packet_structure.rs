use infrastructure::DeliveryMethod;
use std::net::SocketAddr;

#[derive(Clone, PartialEq, Eq, Debug)]
/// This is a user friendly packet containing the payload from the packet and the endpoint from where it came.
pub struct Packet {
    /// the endpoint from where it came
    addr: SocketAddr,
    /// the raw payload of the packet
    payload: Box<[u8]>,
    /// defines on how the packet will be delivered.
    delivery_method: DeliveryMethod,
}

impl Packet {
    /// Create an new packet by passing the receiver, data and how this packet should be delivered.
    pub fn new(addr: SocketAddr, payload: Box<[u8]>, delivery_method: DeliveryMethod) -> Self {
        Packet {
            addr,
            payload,
            delivery_method,
        }
    }

    /// Unreliable. Packets can be dropped, duplicated or arrive without order.
    ///
    /// **Details**
    ///
    /// | Packet Drop     | Packet Duplication | Packet Order     | Packet Fragmentation | Packet Delivery |
    /// | :-------------: | :-------------:    | :-------------:  | :-------------:      | :-------------: |
    /// |       Yes       |        Yes         |      No          |      No              |       No        |
    ///
    /// Basically just bare UDP, free to be dropped, used for very unnecessary data, great for 'general' position updates.
    pub fn unreliable(addr: SocketAddr, payload: Vec<u8>) -> Packet {
        Packet::new(
            addr,
            payload.into_boxed_slice(),
            DeliveryMethod::UnreliableUnordered,
        )
    }

    /// Reliable. All packets will be sent and received, but without order.
    ///
    /// *Details*
    ///
    /// |   Packet Drop   | Packet Duplication | Packet Order     | Packet Fragmentation | Packet Delivery |
    /// | :-------------: | :-------------:    | :-------------:  | :-------------:      | :-------------: |
    /// |       No        |      No            |      No          |      Yes             |       Yes       |
    ///
    /// Basically this is almost TCP like without ordering of packets.
    /// Receive every packet and immediately give to application, order does not matter.
    pub fn reliable_unordered(addr: SocketAddr, payload: Vec<u8>) -> Packet {
        Packet::new(
            addr,
            payload.into_boxed_slice(),
            DeliveryMethod::ReliableUnordered,
        )
    }

    /// Get the payload (raw data) of this packet.
    pub fn payload(&self) -> &[u8] {
        &self.payload
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
