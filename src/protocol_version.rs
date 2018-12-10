use crc::crc32;
use lazy_static::lazy_static;

pub use crate::net::constants::PROTOCOL_VERSION;

lazy_static! {
    // The CRC32 of the current protocol version.
    static ref VERSION_CRC32: u32 = crc32::checksum_ieee(PROTOCOL_VERSION.as_bytes());
}

/// Wrapper to provide some functions to perform with the current protocol version.
pub struct ProtocolVersion;

impl ProtocolVersion {
    /// Get the current protocol version.
    #[inline]
    pub fn get_version() -> &'static str {
        PROTOCOL_VERSION
    }

    /// This will return the crc32 from the current protocol version.
    #[inline]
    pub fn get_crc32() -> u32 {
        *VERSION_CRC32
    }

    /// Validate a crc32 with the current protocol version and return the results.
    #[inline]
    pub fn valid_version(protocol_version_crc32: u32) -> bool {
        protocol_version_crc32 == ProtocolVersion::get_crc32()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::net::constants::PROTOCOL_VERSION;

    #[test]
    fn valid_version() {
        let protocol_id = crc32::checksum_ieee(PROTOCOL_VERSION.as_bytes());
        assert!(ProtocolVersion::valid_version(protocol_id));
    }

    #[test]
    fn not_valid_version() {
        let protocol_id = crc32::checksum_ieee("not-laminar".as_bytes());
        assert!(!ProtocolVersion::valid_version(protocol_id));
    }

    #[test]
    fn get_crc32() {
        assert_eq!(ProtocolVersion::get_crc32(), *VERSION_CRC32);
    }

    #[test]
    fn get_version() {
        assert_eq!(ProtocolVersion::get_version(), PROTOCOL_VERSION);
    }
}
