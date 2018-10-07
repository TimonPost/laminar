pub trait HeaderParser
{
    type Output;

    /// Parse the given instance to the specified Header type
    fn parse(&self) -> Self::Output;
}