/// Contains code dealing with Packet Headers
pub mod header;

mod enums;
mod fragment;
mod outgoing;
mod packet_reader;
mod packet_structure;

pub use self::enums::{DeliveryGuarantee, OrderingGuarantee, PacketType};
pub use self::fragment::Fragment;
pub use self::outgoing::{Outgoing, OutgoingPacket, OutgoingPacketBuilder};
pub use self::packet_reader::PacketReader;
pub use self::packet_structure::Packet;
pub trait EnumConverter {
    type Enum;

    fn to_u8(&self) -> u8;
    fn from_u8(input: u8) -> Self::Enum;
}
