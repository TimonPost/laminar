pub const FRAGMENT_HEADER_SIZE: u8 = 5;
pub const PACKET_HEADER_SIZE: u8 = 8;
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