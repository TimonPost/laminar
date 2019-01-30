/// Trait for writing a header
pub trait HeaderWriter {
    /// Associated type since we parse the header into an Output
    type Output;

    /// Write the header to the given buffer.
    fn parse(&self, buffer: &mut Vec<u8>) -> Self::Output;
}
