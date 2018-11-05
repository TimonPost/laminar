pub trait HeaderParser {
    type Output;

    /// Write the header to the given buffer.
    fn parse(&self, mut buffer: &mut Vec<u8>) -> Self::Output;
}
