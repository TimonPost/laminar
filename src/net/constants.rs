pub const FRAGMENT_HEADER_SIZE: u8 = 9;
pub const PACKET_HEADER_SIZE: u8 = 14;
pub const HEART_BEAT_HEADER_SIZE: u8 = 5;
pub const MAX_FRAGMENTS_DEFAULT: u16 = 16;
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
pub const PROTOCOL_VERSION: &'static str = "laminar-0.1.0";
