use packet::header::{AckedPacketHeader, FragmentHeader};

/// These are the different packets that could be send by te user.
#[allow(dead_code)]
pub enum PacketType {
    /// Packet header containing packet information.
    Normal(AckedPacketHeader),
    /// Part of an packet also called 'fragment' containing fragment info.
    Fragment(FragmentHeader),
    /// Packet to keep the connection alive.
    HeartBeat {/* fields ... */},
    /// Disconnect request
    Disconnect {/* fields ... */},
}

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq)]
/// Id to identify an certain packet type.
pub enum PacketTypeId {
    /// Full packet that is not fragmented
    Packet = 0,
    /// Fragment of a full packet
    Fragment = 1,
    /// Special packet that serves as a heartbeat
    HeartBeat = 2,
    /// Special packet that disconnects
    Disconnect = 3,
    /// Unknown packet type
    Unknown = 255,
}

impl PacketTypeId {
    /// Get integer value from `PacketTypeId` enum.
    pub fn get_id(packet_type: PacketTypeId) -> u8 {
        packet_type as u8
    }

    /// Get `PacketTypeid` enum instance from integer value.
    pub fn get_packet_type(packet_type_id: u8) -> PacketTypeId {
        match packet_type_id {
            0 => PacketTypeId::Packet,
            1 => PacketTypeId::Fragment,
            2 => PacketTypeId::HeartBeat,
            3 => PacketTypeId::Disconnect,
            _ => PacketTypeId::Unknown,
        }
    }
}
