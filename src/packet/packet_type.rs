use packet::header::{PacketHeader, FragmentHeader};

/// These are the different packets that could be send by te user.
pub enum PacketType
{
    /// Packet header containing packet information.
    Normal(PacketHeader),
    /// Part of an packet also called 'fragment' containing fragment info.
    Fragment(FragmentHeader),
    /// Packet to keep the connection alive.
    HeartBeat { /* fields ... */ },
    /// Disconnect request
    Disconnect { /* fields ... */ }
}

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq)]
/// Id to identify an certain packet type.
pub enum PacketTypeId
{
    Packet = 0,
    Fragment = 1,
    HeartBeat = 2,
    Disconnect = 3,
    Unknown = 255,
}

impl PacketTypeId
{
    /// Get integer value from `PacketTypeId` enum.
    pub fn get_id(packet_type: PacketTypeId) -> u8
    {
        packet_type as u8
    }

    /// Get `PacketTypeid` enum instance from integer value.
    pub fn get_packet_type(packet_type_id: u8) -> PacketTypeId {
        match packet_type_id {
            0 => PacketTypeId::Packet,
            1 => PacketTypeId::Fragment,
            2 => PacketTypeId::HeartBeat,
            3 => PacketTypeId::Disconnect,
            _ => PacketTypeId::Unknown
        }
    }
}
