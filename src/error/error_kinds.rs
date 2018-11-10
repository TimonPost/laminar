use failure_derive::Fail;

/// Errors that could occur with constructing parsing packet contents
#[derive(Fail, Debug, PartialEq, Eq, Clone)]
pub enum PacketErrorKind {
    #[fail(display = "The packet size was bigger than the max allowed size.")]
    /// Max packet size was exceeded
    ExceededMaxPacketSize,
    #[fail(display = "The packet has a wrong id.")]
    /// Packet has the wrong ID
    PacketHasWrongId,
}

/// Errors that could occur with constructing/parsing fragment contents
#[derive(Fail, Debug, PartialEq, Eq, Clone)]
pub enum FragmentErrorKind {
    #[fail(display = "Packet header was attached to fragment.")]
    /// PacketHeader was not found in the packet
    PacketHeaderNotFound,
    #[fail(display = "Entry already exists in the buffer.")]
    /// The packet already exists in the buffer
    EntryAlreadyExists,
    #[fail(display = "The total numbers of fragments are bigger than the allowed fragments.")]
    /// Max number of allowed fragments has been exceeded
    ExceededMaxFragments,
    #[fail(display = "The fragment received was already processed.")]
    /// This fragment was already processed
    AlreadyProcessedFragment,
    #[fail(display = "The fragment header does not contain the right fragment count.")]
    /// Attempted to fragment with an incorrect number of fragments
    FragmentWithUnevenNumberOfFragemts,
    #[fail(display = "The fragment has a wrong id.")]
    /// Fragment has incorrect ID
    FragmentHasWrongId,
    #[fail(display = "The fragment supposed to be in a cache but it was not found.")]
    /// Fragment we expected to be able to find we couldn't
    CouldNotFindFragmentById,
}
