use std::io::Cursor;
/// Trait that supports reading a Header from a packet
pub trait HeaderReader {
    /// Associated type for the HeaderReader, since it reads it from a Header
    type Header;

    /// Read the specified header from the given Cursor.
    fn read(rdr: &mut Cursor<&[u8]>) -> Self::Header;

    /// This will get the size of the header.
    fn size(&self) -> u8;
}
