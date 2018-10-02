use std::io::Cursor;

pub trait HeaderReader
{
    type Header;

    /// Reads the specified header from the given Cursor.
    fn read(rdr:  &mut Cursor<Vec<u8>>) -> Self::Header;
}