use crate::packet::{DeliveryGuarantee, OrderingGuarantee};
use std::net::SocketAddr;

#[derive(Clone, PartialEq, Eq, Debug)]
/// This is a user friendly packet containing the payload, endpoint, and reliability guarantees.
/// A packet could have reliability guarantees to specify how it should be delivered and processed.
///
/// | Reliability Type             | Packet Drop     | Packet Duplication | Packet Order     | Packet Fragmentation |Packet Delivery|
/// | :-------------:              | :-------------: | :-------------:    | :-------------:  | :-------------:      | :-------------:
/// |   **Unreliable Unordered**   |       Any       |      Yes           |     No           |      No              |   No
/// |   **Unreliable Sequenced**   |    Any + old    |      No            |     Sequenced    |      No              |   No
/// |   **Reliable Unordered**     |       No        |      No            |     No           |      Yes             |   Yes
/// |   **Reliable Ordered**       |       No        |      No            |     Ordered      |      Yes             |   Yes
/// |   **Reliable Sequenced**     |    Only old     |      No            |     Sequenced    |      Yes             |   Only newest
///
/// You are able to send packets with the above reliability types.
pub struct Packet {
    /// the endpoint from where it came
    addr: SocketAddr,
    /// the raw payload of the packet
    payload: Box<[u8]>,
    /// defines on how the packet will be delivered.
    delivery: DeliveryGuarantee,
    /// defines on how the packet will be ordered.
    ordering: OrderingGuarantee,
}

impl Packet {
    /// Create a new packet by passing the receiver, data, and guarantees on how this packet should be delivered.
    pub(crate) fn new(
        addr: SocketAddr,
        payload: Box<[u8]>,
        delivery: DeliveryGuarantee,
        ordering: OrderingGuarantee,
    ) -> Packet {
        Packet {
            addr,
            payload,
            delivery,
            ordering,
        }
    }

    /// Create a new unreliable packet by passing the receiver, data.
    ///
    /// Unreliable: Packets can be dropped, duplicated or arrive without order.
    ///
    /// **Details**
    ///
    /// | Packet Drop     | Packet Duplication | Packet Order     | Packet Fragmentation | Packet Delivery |
    /// | :-------------: | :-------------:    | :-------------:  | :-------------:      | :-------------: |
    /// |       Any       |        Yes         |      No          |      No              |       No        |
    ///
    /// Basically just bare UDP. The packet may or may not be delivered.
    pub fn unreliable(addr: SocketAddr, payload: Vec<u8>) -> Packet {
        Packet {
            addr,
            payload: payload.into_boxed_slice(),
            delivery: DeliveryGuarantee::Unreliable,
            ordering: OrderingGuarantee::None,
        }
    }

    /// Create a new unreliable sequenced packet by passing the receiver, data.
    ///
    /// Unreliable Sequenced; Packets can be dropped, but could not be duplicated and arrive in sequence.
    ///
    /// *Details*
    ///
    /// | Packet Drop     | Packet Duplication | Packet Order     | Packet Fragmentation | Packet Delivery |
    /// | :-------------: | :-------------:    | :-------------:  | :-------------:      | :-------------: |
    /// |    Any + old    |        No          |      Sequenced   |      No              |       No        |
    ///
    /// Basically just bare UDP, free to be dropped, but has some sequencing to it so that only the newest packets are kept.
    pub fn unreliable_sequenced(
        addr: SocketAddr,
        payload: Vec<u8>,
        stream_id: Option<u8>,
    ) -> Packet {
        Packet {
            addr,
            payload: payload.into_boxed_slice(),
            delivery: DeliveryGuarantee::Unreliable,
            ordering: OrderingGuarantee::Sequenced(stream_id),
        }
    }

    /// Create a new packet by passing the receiver, data.
    /// Reliable; All packets will be sent and received, but without order.
    ///
    /// *Details*
    ///
    /// |   Packet Drop   | Packet Duplication | Packet Order     | Packet Fragmentation | Packet Delivery |
    /// | :-------------: | :-------------:    | :-------------:  | :-------------:      | :-------------: |
    /// |       No        |      No            |      No          |      Yes             |       Yes       |
    ///
    /// Basically this is almost TCP without ordering of packets.
    pub fn reliable_unordered(addr: SocketAddr, payload: Vec<u8>) -> Packet {
        Packet {
            addr,
            payload: payload.into_boxed_slice(),
            delivery: DeliveryGuarantee::Reliable,
            ordering: OrderingGuarantee::None,
        }
    }

    /// Create a new packet by passing the receiver, data and a optional stream on which the ordering will be done.
    ///
    /// Reliable; All packets will be sent and received, with order.
    ///
    /// *Details*
    ///
    /// |   Packet Drop   | Packet Duplication | Packet Order     | Packet Fragmentation | Packet Delivery |
    /// | :-------------: | :-------------:    | :-------------:  | :-------------:      | :-------------: |
    /// |       No        |      No            |      Ordered     |      Yes             |       Yes       |
    ///
    /// Basically this is almost TCP-like with ordering of packets.
    ///
    /// # Remark
    /// - When `stream_id` is specified as `None` the default stream will be used; if you are not sure what this is you can leave it at `None`.
    pub fn reliable_ordered(addr: SocketAddr, payload: Vec<u8>, stream_id: Option<u8>) -> Packet {
        Packet {
            addr,
            payload: payload.into_boxed_slice(),
            delivery: DeliveryGuarantee::Reliable,
            ordering: OrderingGuarantee::Ordered(stream_id),
        }
    }

    /// Create a new packet by passing the receiver, data and a optional stream on which the sequencing will be done.
    ///
    /// Reliable; All packets will be sent and received, but arranged in sequence.
    /// Which means that only the newest packets will be let through, older packets will be received but they won't get to the user.
    ///
    /// *Details*
    ///
    /// |   Packet Drop   | Packet Duplication | Packet Order     | Packet Fragmentation | Packet Delivery |
    /// | :-------------: | :-------------:    | :-------------:  | :-------------:      | :-------------: |
    /// |    Only old     |      No            |      Sequenced   |      Yes             |   Only newest   |
    ///
    /// Basically this is almost TCP-like but then sequencing instead of ordering.
    ///
    /// # Remark
    /// - When `stream_id` is specified as `None` the default stream will be used; if you are not sure what this is you can leave it at `None`.
    pub fn reliable_sequenced(addr: SocketAddr, payload: Vec<u8>, stream_id: Option<u8>) -> Packet {
        Packet {
            addr,
            payload: payload.into_boxed_slice(),
            delivery: DeliveryGuarantee::Reliable,
            ordering: OrderingGuarantee::Sequenced(stream_id),
        }
    }

    /// Returns the payload of this packet.
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    /// Returns the address of this packet.
    ///
    /// # Remark
    /// Could be both the receiving endpoint or the one to send this packet to.
    /// This depends whether it is a packet that has been received or one that needs to be send.
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// Returns the [`DeliveryGuarantee`](./enum.DeliveryGuarantee.html) of this packet.
    pub fn delivery_guarantee(&self) -> DeliveryGuarantee {
        self.delivery
    }

    /// Returns the [`OrderingGuarantee`](./enum.OrderingGuarantee.html) of this packet.
    pub fn order_guarantee(&self) -> OrderingGuarantee {
        self.ordering
    }
}

#[cfg(test)]
mod tests {
    use crate::packet::{DeliveryGuarantee, OrderingGuarantee, Packet};
    use std::net::SocketAddr;

    #[test]
    fn assure_creation_unreliable_packet() {
        let packet = Packet::unreliable(test_addr(), test_payload());

        assert_eq!(packet.addr(), test_addr());
        assert_eq!(packet.payload(), test_payload().as_slice());
        assert_eq!(packet.delivery_guarantee(), DeliveryGuarantee::Unreliable);
        assert_eq!(packet.order_guarantee(), OrderingGuarantee::None);
    }

    #[test]
    fn assure_creation_unreliable_sequenced() {
        let packet = Packet::unreliable_sequenced(test_addr(), test_payload(), Some(1));

        assert_eq!(packet.addr(), test_addr());
        assert_eq!(packet.payload(), test_payload().as_slice());
        assert_eq!(packet.delivery_guarantee(), DeliveryGuarantee::Unreliable);
        assert_eq!(
            packet.order_guarantee(),
            OrderingGuarantee::Sequenced(Some(1))
        );
    }

    #[test]
    fn assure_creation_reliable() {
        let packet = Packet::reliable_unordered(test_addr(), test_payload());

        assert_eq!(packet.addr(), test_addr());
        assert_eq!(packet.payload(), test_payload().as_slice());
        assert_eq!(packet.delivery_guarantee(), DeliveryGuarantee::Reliable);
        assert_eq!(packet.order_guarantee(), OrderingGuarantee::None);
    }

    #[test]
    fn assure_creation_reliable_ordered() {
        let packet = Packet::reliable_ordered(test_addr(), test_payload(), Some(1));

        assert_eq!(packet.addr(), test_addr());
        assert_eq!(packet.payload(), test_payload().as_slice());
        assert_eq!(packet.delivery_guarantee(), DeliveryGuarantee::Reliable);
        assert_eq!(
            packet.order_guarantee(),
            OrderingGuarantee::Ordered(Some(1))
        );
    }

    #[test]
    fn assure_creation_reliable_sequence() {
        let packet = Packet::reliable_sequenced(test_addr(), test_payload(), Some(1));

        assert_eq!(packet.addr(), test_addr());
        assert_eq!(packet.payload(), test_payload().as_slice());
        assert_eq!(packet.delivery_guarantee(), DeliveryGuarantee::Reliable);
        assert_eq!(
            packet.order_guarantee(),
            OrderingGuarantee::Sequenced(Some(1))
        );
    }

    fn test_payload() -> Vec<u8> {
        b"test".to_vec()
    }

    fn test_addr() -> SocketAddr {
        "127.0.0.1:12345".parse().unwrap()
    }
}
