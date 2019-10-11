use std::io::Cursor;

/// Trait that supports reading a Header from a packet
pub trait HeaderReader {
    /// Associated type for the HeaderReader, since it reads it from a Header
    type Header;

    /// Reads the specified header from the given Cursor.
    fn read(rdr: &mut Cursor<&[u8]>) -> Self::Header;

    /// Returns the size of the header.
    fn size() -> u8;
}
