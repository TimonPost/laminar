/// Contains code dealing with Packet Headers
pub mod header;

mod packet_structure;
mod packet_data;
mod packet_type;

pub use self::packet_data::PacketData;
pub use self::packet_structure::Packet;
pub use self::packet_type::{PacketTypeId, PacketType};

