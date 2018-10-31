use std::io::Cursor;

pub trait HeaderReader {
    type Header;

    /// Read the specified header from the given Cursor.
    fn read(rdr: &mut Cursor<&[u8]>) -> Self::Header;

    /// This will get the size of the header.
    fn size(&self) -> u8;
}
