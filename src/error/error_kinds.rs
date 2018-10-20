use std::io;

/// Errors that could occur with constructing parsing packet contents
#[derive(Fail, Debug, PartialEq, Eq, Clone)]
pub enum PacketErrorKind {
    #[fail(display = "The packet size was bigger than the max allowed size.")]
    ExceededMaxPacketSize,
    #[fail(display = "The packet has a wrong id.")]
    PacketHasWrongId
}

/// Errors that could occur with constructing/parsing fragment contents
#[derive(Fail, Debug, PartialEq, Eq, Clone)]
pub enum FragmentErrorKind
{
    #[fail(display = "Packet header was attached to fragment.")]
    PacketHeaderNotFound,
    #[fail(display = "Entry already exists in the buffer.")]
    EntryAlreadyExists,
    #[fail(display = "The total numbers of fragments are bigger than the allowed fragments.")]
    ExceededMaxFragments,
    #[fail(display = "The fragment received was already processed.")]
    AlreadyProcessedFragment,
    #[fail(display = "The fragment header does not contain the right fragment count.")]
    FragmentWithUnevenNumberOfFragemts,
    #[fail(display = "The fragment has a wrong id.")]
    FragmentHasWrongId,
    #[fail(display = "The fragment supposed to be in a cache but it was not found.")]
    CouldNotFindFragmentById
}

/// Errors that could occur with TCP-protocol
#[derive(Fail, Debug, PartialEq, Eq, Clone)]
pub enum TcpErrorKind {
    #[fail(display = "Could not clone TCP-Stream.")]
    TcpStreamCloneFailed,
    #[fail(display = "Could not take 'rx' channel from inside the 'outgoing loop'.")]
    TcpSteamFailedTakeRx,
    #[fail(display = "Could not get the lock from TCP-connections hash, because it was poisoned.")]
    TcpClientConnectionsHashPoisoned,
    #[fail(display = "Could not get the lock for a specific TCP-client, because it was poisoned.")]
    TcpClientLockFailed,
}