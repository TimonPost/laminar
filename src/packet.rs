//! This module provides all the logic around the packet, such as reading, parsing, and constructing headers.

pub mod header;

mod enums;
mod outgoing;
mod packet_reader;
mod packet_structure;
mod process_result;

pub use self::enums::{DeliveryGuarantee, OrderingGuarantee, PacketType};
pub use self::outgoing::{OutgoingPacket, OutgoingPacketBuilder};
pub use self::packet_reader::PacketReader;
pub use self::packet_structure::{Packet, PacketInfo};
pub use self::process_result::{IncomingPackets, OutgoingPackets};

pub type SequenceNumber = u16;

pub trait EnumConverter {
    type Enum;

    fn to_u8(&self) -> u8;
}
