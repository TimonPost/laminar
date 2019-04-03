//! This module contains the laminar error handling logic.


use crate::SocketEvent;
use crossbeam_channel::SendError;
use std::{
    fmt::{self, Display, Formatter},
    io, result,
};

/// Wrapped result type for Laminar errors.
pub type Result<T> = result::Result<T, ErrorKind>;

#[derive(Debug)]
/// Enum with all possible network errors that could occur.
pub enum ErrorKind {
    /// Error relating to receiving or parsing a fragment
    FragmentError(FragmentErrorKind),
    /// Error relating to receiving or parsing a packet
    PacketError(PacketErrorKind),
    /// Wrapper around a std io::Error
    IOError(io::Error),
    /// Did not receive enough data
    ReceivedDataToShort,
    /// Protocol versions did not match
    ProtocolVersionMismatch,
    /// Could not send on `SendChannel`.
    SendError(SendError<SocketEvent>),
    /// Expected header but could not be read from buffer.
    CouldNotReadHeader(String),
}

impl Display for ErrorKind {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::FragmentError(e) => write!(
                fmt,
                "Something went wrong with receiving/parsing fragments. Reason: {:?}.",
                e
            ),
            ErrorKind::PacketError(e) => write!(
                fmt,
                "Something went wrong with receiving/parsing packets. Reason: {:?}.",
                e
            ),
            ErrorKind::IOError(e) => write!(fmt, "An IO Error occurred. Reason: {:?}.", e),
            ErrorKind::ReceivedDataToShort => {
                write!(fmt, "The received data did not have any length.")
            }
            ErrorKind::ProtocolVersionMismatch => {
                write!(fmt, "The protocol versions do not match.")
            }
            ErrorKind::SendError(e) => write!(
                fmt,
                "Could not sent on channel because it was closed. Reason: {:?}",
                e
            ),
            ErrorKind::CouldNotReadHeader(header) => write!(
                fmt,
                "Expected {} header but could not be read from buffer.",
                header
            ),
        }
    }
}

/// Errors that could occur while parsing packet contents
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PacketErrorKind {
    /// The maximal allowed size of the packet was exceeded
    ExceededMaxPacketSize,
}

impl Display for PacketErrorKind {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            PacketErrorKind::ExceededMaxPacketSize => {
                write!(fmt, "The packet size was bigger than the max allowed size.")
            }
        }
    }
}

/// Errors that could occur with constructing/parsing fragment contents
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FragmentErrorKind {
    /// PacketHeader was not found in the packet
    PacketHeaderNotFound,
    /// Max number of allowed fragments has been exceeded
    ExceededMaxFragments,
    /// This fragment was already processed
    AlreadyProcessedFragment,
    /// Attempted to fragment with an incorrect number of fragments
    FragmentWithUnevenNumberOfFragemts,
    /// Fragment we expected to be able to find we couldn't
    CouldNotFindFragmentById,
}

impl Display for FragmentErrorKind {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            FragmentErrorKind::PacketHeaderNotFound => {
                write!(fmt, "Packet header was attached to fragment.")
            }
            FragmentErrorKind::ExceededMaxFragments => write!(
                fmt,
                "The total numbers of fragments are bigger than the allowed fragments."
            ),
            FragmentErrorKind::AlreadyProcessedFragment => {
                write!(fmt, "The fragment received was already processed.")
            }
            FragmentErrorKind::FragmentWithUnevenNumberOfFragemts => write!(
                fmt,
                "The fragment header does not contain the right fragment count."
            ),
            FragmentErrorKind::CouldNotFindFragmentById => write!(
                fmt,
                "The fragment supposed to be in a the cache but it was not found."
            ),
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
