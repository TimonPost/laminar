/// The size of the fragment header.
pub const FRAGMENT_HEADER_SIZE: u8 = 4;
/// The size of the acknowledgment header.
pub const ACKED_PACKET_HEADER: u8 = 8;
/// The size of the arranging header.
pub const ARRANGING_PACKET_HEADER: u8 = 3;
/// The size of the standard header.
pub const STANDARD_HEADER_SIZE: u8 = 5;
/// The ordering stream that will be used to order on if none was specified.
pub const DEFAULT_ORDERING_STREAM: u8 = 255;
/// The sequencing stream that will be used to sequence packets on if none was specified.
pub const DEFAULT_SEQUENCING_STREAM: u8 = 255;
/// Default maximal number of fragments to size.
pub const MAX_FRAGMENTS_DEFAULT: u16 = 16;
/// Default maximal size of each fragment.
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
/// - Generating crc16 for the packet header.
/// - Validating if arriving packets have the same protocol version.
pub const PROTOCOL_VERSION: &str = "laminar-0.1.0";
