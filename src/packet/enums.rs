use std::convert::TryFrom;

use crate::{
    error::{DecodingErrorKind, ErrorKind},
    packet::EnumConverter,
};

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
}

impl TryFrom<u8> for DeliveryGuarantee {
    type Error = ErrorKind;
    /// Get `DeliveryGuarantee` enum instance from integer value.
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(DeliveryGuarantee::Unreliable),
            1 => Ok(DeliveryGuarantee::Reliable),
            _ => Err(ErrorKind::DecodingError(
                DecodingErrorKind::DeliveryGuarantee,
            )),
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
}

impl TryFrom<u8> for OrderingGuarantee {
    type Error = ErrorKind;
    /// Get `OrderingGuarantee` enum instance from integer value.
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(OrderingGuarantee::None),
            1 => Ok(OrderingGuarantee::Sequenced(None)),
            2 => Ok(OrderingGuarantee::Ordered(None)),
            _ => Err(ErrorKind::DecodingError(
                DecodingErrorKind::OrderingGuarantee,
            )),
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
    /// Heartbeat packet
    Heartbeat = 2,
}

impl EnumConverter for PacketType {
    type Enum = PacketType;

    fn to_u8(&self) -> u8 {
        *self as u8
    }
}

impl TryFrom<u8> for PacketType {
    type Error = ErrorKind;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(PacketType::Packet),
            1 => Ok(PacketType::Fragment),
            2 => Ok(PacketType::Heartbeat),
            _ => Err(ErrorKind::DecodingError(DecodingErrorKind::PacketType)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use crate::packet::{
        EnumConverter,
        enums::{DeliveryGuarantee, OrderingGuarantee, PacketType},
    };

    #[test]
    fn assure_parsing_ordering_guarantee() {
        let none = OrderingGuarantee::None;
        let ordered = OrderingGuarantee::Ordered(None);
        let sequenced = OrderingGuarantee::Sequenced(None);

        assert_eq!(
            OrderingGuarantee::None,
            OrderingGuarantee::try_from(none.to_u8()).unwrap()
        );
        assert_eq!(
            OrderingGuarantee::Ordered(None),
            OrderingGuarantee::try_from(ordered.to_u8()).unwrap()
        );
        assert_eq!(
            OrderingGuarantee::Sequenced(None),
            OrderingGuarantee::try_from(sequenced.to_u8()).unwrap()
        )
    }

    #[test]
    fn assure_parsing_delivery_guarantee() {
        let unreliable = DeliveryGuarantee::Unreliable;
        let reliable = DeliveryGuarantee::Reliable;
        assert_eq!(
            DeliveryGuarantee::Unreliable,
            DeliveryGuarantee::try_from(unreliable.to_u8()).unwrap()
        );
        assert_eq!(
            DeliveryGuarantee::Reliable,
            DeliveryGuarantee::try_from(reliable.to_u8()).unwrap()
        )
    }

    #[test]
    fn assure_parsing_packet_type() {
        let packet = PacketType::Packet;
        let fragment = PacketType::Fragment;
        let heartbeat = PacketType::Heartbeat;
        assert_eq!(
            PacketType::Packet,
            PacketType::try_from(packet.to_u8()).unwrap()
        );
        assert_eq!(
            PacketType::Fragment,
            PacketType::try_from(fragment.to_u8()).unwrap()
        );
        assert_eq!(
            PacketType::Heartbeat,
            PacketType::try_from(heartbeat.to_u8()).unwrap()
        );
    }
}
