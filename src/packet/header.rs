//! This module provides parses and readers for the headers that could be appended to any packet.
//! We use headers to control reliability, fragmentation, and ordering.

pub use self::acked_packet_header::AckedPacketHeader;
pub use self::arranging_header::ArrangingHeader;
pub use self::fragment_header::FragmentHeader;
pub use self::header_reader::HeaderReader;
pub use self::header_writer::HeaderWriter;
pub use self::standard_header::StandardHeader;

mod acked_packet_header;
mod arranging_header;
mod fragment_header;
mod header_reader;
mod header_writer;
mod standard_header;
