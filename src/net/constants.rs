/// Fragment header size
pub const FRAGMENT_HEADER_SIZE: u8 = 4 + STANDARD_HEADER_SIZE;
/// Acked packet header size
pub const ACKED_PACKET_HEADER: u8 = 8 + STANDARD_HEADER_SIZE;
/// Standard header size
pub const STANDARD_HEADER_SIZE: u8 = 6;
/// Heartbeat header size
pub const HEART_BEAT_HEADER_SIZE: u8 = 5;
/// Default max number of fragments to size
pub const MAX_FRAGMENTS_DEFAULT: u16 = 16;
/// Default max size of each fragment
pub const FRAGMENT_SIZE_DEFAULT: u16 = 1024;

/// This is the current protocol version.
///
/// It is used for:
/// - Generating crc32 for the packet header.
/// - Validating if arriving packets have the same protocol version.
pub const PROTOCOL_VERSION: &str = "laminar-0.1.0";
