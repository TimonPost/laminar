use crc::crc16;

use lazy_static::lazy_static;

pub use crate::net::constants::PROTOCOL_VERSION;

lazy_static! {
    // The CRC16 of the current protocol version.
    static ref VERSION_CRC16: u16 = crc16::checksum_x25(PROTOCOL_VERSION.as_bytes());
}

/// Wrapper to provide some functions to perform with the current protocol version.
pub struct ProtocolVersion;

impl ProtocolVersion {
    /// Get the current protocol version.
    #[inline]
    #[cfg(test)]
    pub fn get_version() -> &'static str {
        PROTOCOL_VERSION
    }

    /// This will return the crc16 from the current protocol version.
    #[inline]
    pub fn get_crc16() -> u16 {
        *VERSION_CRC16
    }

    /// Validate a crc16 with the current protocol version and return the results.
    #[inline]
    pub fn valid_version(protocol_version_crc16: u16) -> bool {
        protocol_version_crc16 == ProtocolVersion::get_crc16()
    }
}

#[cfg(test)]
mod test {
    use crate::net::constants::PROTOCOL_VERSION;

    use super::*;

    #[test]
    fn valid_version() {
        let protocol_id = crc16::checksum_x25(PROTOCOL_VERSION.as_bytes());
        assert!(ProtocolVersion::valid_version(protocol_id));
    }

    #[test]
    fn not_valid_version() {
        let protocol_id = crc16::checksum_x25(b"not-laminar");
        assert!(!ProtocolVersion::valid_version(protocol_id));
    }

    #[test]
    fn get_crc16() {
        assert_eq!(ProtocolVersion::get_crc16(), *VERSION_CRC16);
    }

    #[test]
    fn get_version() {
        assert_eq!(ProtocolVersion::get_version(), PROTOCOL_VERSION);
    }
}
