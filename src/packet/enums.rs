use crate::packet::EnumConverter;

/// Enum to specify how a packet should be delivered.
#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Eq)]
pub enum DeliveryGuarantee {
    /// Packet may or may not be delivered
    Unreliable,
    /// Packet will be delivered
    Reliable,
}

impl EnumConverter for DeliveryGuarantee {
    type Enum = DeliveryGuarantee;

    /// Get integer value from `DeliveryGuarantee` enum.
    fn to_u8(&self) -> u8 {
        *self as u8
    }

    /// Get `DeliveryGuarantee` enum instance from integer value.
    fn from_u8(input: u8) -> Self::Enum {
        match input {
            0 => DeliveryGuarantee::Unreliable,
            1 => DeliveryGuarantee::Reliable,
            _ => unimplemented!("Delivery Guarantee {} does not exist yet.", input),
        }
    }
}

/// Enum to specify how a packet should be arranged.
#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Eq)]
pub enum OrderingGuarantee {
    /// No arranging will be done.
    None,
    /// Packets will be arranged in sequence.
    Sequenced(Option<u8>),
    /// Packets will be arranged in order.
    Ordered(Option<u8>),
}

impl Default for OrderingGuarantee {
    fn default() -> Self {
        OrderingGuarantee::None
    }
}

impl EnumConverter for OrderingGuarantee {
    type Enum = OrderingGuarantee;

    /// Get integer value from `OrderingGuarantee` enum.
    fn to_u8(&self) -> u8 {
        match self {
            OrderingGuarantee::None => 0,
            OrderingGuarantee::Sequenced(_) => 1,
            OrderingGuarantee::Ordered(_) => 2,
        }
    }

    /// Get `OrderingGuarantee` enum instance from integer value.
    fn from_u8(input: u8) -> Self::Enum {
        match input {
            0 => OrderingGuarantee::None,
            1 => OrderingGuarantee::Sequenced(None),
            2 => OrderingGuarantee::Ordered(None),
            _ => unimplemented!("Ordering Guarantee {} does not exist yet.", input),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq)]
/// Id to identify a certain packet type.
pub enum PacketType {
    /// Full packet that is not fragmented
    Packet = 0,
    /// Fragment of a full packet
    Fragment = 1,
}

impl EnumConverter for PacketType {
    type Enum = PacketType;

    fn to_u8(&self) -> u8 {
        *self as u8
    }

    fn from_u8(input: u8) -> Self::Enum {
        match input {
            0 => PacketType::Packet,
            1 => PacketType::Fragment,
            _ => unimplemented!("Packet ID {} does not exist yet.", input),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::packet::{
        enums::{DeliveryGuarantee, OrderingGuarantee, PacketType},
        EnumConverter,
    };

    #[test]
    fn assure_parsing_ordering_guarantee() {
        let none = OrderingGuarantee::None;
        let ordered = OrderingGuarantee::Ordered(None);
        let sequenced = OrderingGuarantee::Sequenced(None);

        assert_eq!(
            OrderingGuarantee::None,
            OrderingGuarantee::from_u8(none.to_u8())
        );
        assert_eq!(
            OrderingGuarantee::Ordered(None),
            OrderingGuarantee::from_u8(ordered.to_u8())
        );
        assert_eq!(
            OrderingGuarantee::Sequenced(None),
            OrderingGuarantee::from_u8(sequenced.to_u8())
        )
    }

    #[test]
    fn assure_parsing_delivery_guarantee() {
        let unreliable = DeliveryGuarantee::Unreliable;
        let reliable = DeliveryGuarantee::Reliable;
        assert_eq!(
            DeliveryGuarantee::Unreliable,
            DeliveryGuarantee::from_u8(unreliable.to_u8())
        );
        assert_eq!(
            DeliveryGuarantee::Reliable,
            DeliveryGuarantee::from_u8(reliable.to_u8())
        )
    }

    #[test]
    fn assure_parsing_packet_id() {
        let packet = PacketType::Packet;
        let fragment = PacketType::Fragment;
        assert_eq!(PacketType::Packet, PacketType::from_u8(packet.to_u8()));
        assert_eq!(PacketType::Fragment, PacketType::from_u8(fragment.to_u8()))
    }
}
