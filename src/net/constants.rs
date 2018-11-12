/// Fragment header size
pub const FRAGMENT_HEADER_SIZE: u8 = 4 + STANDARD_HEADER_SIZE;
/// Acked packet header size
pub const ACKED_PACKET_HEADER: u8 = 8 + STANDARD_HEADER_SIZE;
/// Sequenced packet header size
pub const SEQUENCED_PACKET_HEADER: u8 = 2 + STANDARD_HEADER_SIZE;
/// Standard header size
pub const STANDARD_HEADER_SIZE: u8 = 6;
/// Heartbeat header size
pub const HEART_BEAT_HEADER_SIZE: u8 = 5;
/// Default max number of fragments to size
pub const MAX_FRAGMENTS_DEFAULT: u16 = 16;
/// Default max size of each fragment
pub const FRAGMENT_SIZE_DEFAULT: u16 = 1024;

/// Maximum transmission unit of the payload.
///
/// Derived from ethernet_mtu - ipv6_header_size - udp_header_size - packet header size
///       1452 = 1500         - 40               - 8               - 8
///
/// This is not strictly guaranteed -- there may be less room in an ethernet frame than this due to
/// variability in ipv6 header size.
pub const DEFAULT_MTU: u16 = 1452;
/// This is the current protocol version.
///
/// It is used for:
/// - Generating crc32 for the packet header.
/// - Validating if arriving packets have the same protocol version.
pub const PROTOCOL_VERSION: &str = "laminar-0.1.0";
