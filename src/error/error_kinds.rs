use std::fmt::{self, Display, Formatter};

/// Errors that could occur with constructing parsing packet contents
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PacketErrorKind {
    /// Max packet size was exceeded
    ExceededMaxPacketSize,
    /// Packet has the wrong ID
    PacketHasWrongId,
}

/// Errors that could occur with constructing/parsing fragment contents
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FragmentErrorKind {
    /// PacketHeader was not found in the packet
    PacketHeaderNotFound,
    /// The packet already exists in the buffer
    EntryAlreadyExists,
    /// Max number of allowed fragments has been exceeded
    ExceededMaxFragments,
    /// This fragment was already processed
    AlreadyProcessedFragment,
    /// Attempted to fragment with an incorrect number of fragments
    FragmentWithUnevenNumberOfFragemts,
    /// Fragment has incorrect ID
    FragmentHasWrongId,
    /// Fragment we expected to be able to find we couldn't
    CouldNotFindFragmentById,
}

impl Display for FragmentErrorKind {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            FragmentErrorKind::PacketHeaderNotFound => { write!(fmt, "Packet header was attached to fragment.") },
            FragmentErrorKind::EntryAlreadyExists => { write!(fmt, "Entry already exists in the buffer.") },
            FragmentErrorKind::ExceededMaxFragments => { write!(fmt, "The total numbers of fragments are bigger than the allowed fragments.") },
            FragmentErrorKind::AlreadyProcessedFragment => { write!(fmt, "The fragment received was already processed.") },
            FragmentErrorKind::FragmentWithUnevenNumberOfFragemts => { write!(fmt, "The fragment header does not contain the right fragment count.") },
            FragmentErrorKind::FragmentHasWrongId => { write!(fmt, "The fragment has a wrong id.") },
            FragmentErrorKind::CouldNotFindFragmentById => { write!(fmt, "The fragment supposed to be in a cache but it was not found.") },
        }
    }
}

impl Display for PacketErrorKind {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            PacketErrorKind::ExceededMaxPacketSize => { write!(fmt, "The packet size was bigger than the max allowed size.") },
            PacketErrorKind::PacketHasWrongId => { write!(fmt, "The packet has a wrong id.") },
        }
    }
}
