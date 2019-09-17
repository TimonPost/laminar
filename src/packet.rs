//! This module provides all the logic around the packet, such as reading, parsing, and constructing headers.

pub mod header;

mod enums;
mod outgoing;
mod packet_reader;
mod packet_structure;

pub use self::enums::{DeliveryGuarantee, OrderingGuarantee, PacketType};
pub use self::outgoing::{Outgoing, OutgoingPacket, OutgoingPacketBuilder};
pub use self::packet_reader::PacketReader;
pub use self::packet_structure::{GenericPacket, Packet};

pub type SequenceNumber = u16;

pub trait EnumConverter {
    type Enum;

    fn to_u8(&self) -> u8;
}
