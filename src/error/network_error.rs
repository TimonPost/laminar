use super::{FragmentErrorKind, PacketErrorKind};

use crate::SocketEvent;
use crossbeam_channel::SendError;
use std::fmt::{self, Display, Formatter};
use std::io;

#[derive(Debug)]
/// Enum with all possible network errors that could occur.
pub enum ErrorKind {
    /// Error relating to receiving or parsing a fragment
    FragmentError(FragmentErrorKind),
    /// Error relating to receiving or parsing a packet
    PacketError(PacketErrorKind),
    /// Failed to add a connection
    FailedToAddConnection(String),
    /// Wrapper around a std io::Error
    IOError(io::Error),
    /// Error setting nonblocking on a udp server
    UnableToSetNonblocking,
    /// Error when creating the UDP Socket State
    UDPSocketStateCreationFailed,
    /// Did not receive enough data
    ReceivedDataToShort,
    /// Protocol versions did not match
    ProtocolVersionMismatch,
    /// Error occurred in connection pool.
    ConnectionPoolError(String),
    /// Error occurred when joining thread.
    JoiningThreadFailed,
    /// There was an unexpected error caused by a poisoned lock.
    PoisonedLock(String),
    /// Could not send on `SendChannel`.
    SendError(SendError<SocketEvent>),
    /// Expected header but could not be read from buffer.
    CouldNotReadHeader(String),
}

impl Display for ErrorKind {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::FragmentError(e) => { write!(fmt, "Something went wrong with receiving/parsing fragments. Reason: {:?}.", e) },
            ErrorKind::PacketError(e) => { write!(fmt, "Something went wrong with receiving/parsing packets. Reason: {:?}.", e) },
            ErrorKind::FailedToAddConnection(e) => { write!(fmt, "Could not add a connection to the connection pool, because the connection lock is poisoned. Reason: {:?}.", e) },
            ErrorKind::IOError(e) => { write!(fmt, "An IO Error occurred. Reason: {:?}.", e) },
            ErrorKind::UnableToSetNonblocking => { write!(fmt, "Something went wrong when setting non-blocking option.") },
            ErrorKind::UDPSocketStateCreationFailed => { write!(fmt, "Something went wrong when creating UDP SocketState structure.") },
            ErrorKind::ReceivedDataToShort => { write!(fmt, "The received data did not have any length.") },
            ErrorKind::ProtocolVersionMismatch => { write!(fmt, "The protocol versions do not match.") },
            ErrorKind::ConnectionPoolError(e) => { write!(fmt, "Something went wrong with connection timeout thread. Reason: {:?}", e) },
            ErrorKind::JoiningThreadFailed => { write!(fmt, "Joining thread failed.") },
            ErrorKind::PoisonedLock(e) => { write!(fmt, "There was an unexpected error caused by an poisoned lock. Reason: {:?}", e) },
            ErrorKind::SendError(e) => { write!(fmt, "Could not sent on channel because it was closed. Reason: {:?}", e) },
            ErrorKind::CouldNotReadHeader(header) => { write!(fmt, "Expected {} header but could not be read from buffer.", header) }
        }
    }
}

impl From<io::Error> for ErrorKind {
    fn from(inner: io::Error) -> ErrorKind {
        ErrorKind::IOError(inner)
    }
}

impl From<PacketErrorKind> for ErrorKind {
    fn from(inner: PacketErrorKind) -> Self {
        ErrorKind::PacketError(inner)
    }
}

impl From<FragmentErrorKind> for ErrorKind {
    fn from(inner: FragmentErrorKind) -> Self {
        ErrorKind::FragmentError(inner)
    }
}

impl From<crossbeam_channel::SendError<SocketEvent>> for ErrorKind {
    fn from(inner: SendError<SocketEvent>) -> Self {
        ErrorKind::SendError(inner)
    }
}
