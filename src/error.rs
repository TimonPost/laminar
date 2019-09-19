//! This module contains the laminar error handling logic.

use crate::either::Either;
use crate::net::events::{ConnectionEvent, ReceiveEvent, SendEvent};
use crate::net::managers::ConnectionManagerError;
use crossbeam_channel::SendError;
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    io, result,
};

/// Wrapped result type for Laminar errors.
pub type Result<T> = result::Result<T, ErrorKind>;

#[derive(Debug)]
/// Enum with all possible network errors that could occur.
pub enum ErrorKind {
    /// Error in decoding the packet
    DecodingError(DecodingErrorKind),
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
    SendError(SendError<Either<ConnectionEvent<SendEvent>, ConnectionEvent<ReceiveEvent>>>),
    /// Expected header but could not be read from buffer.
    CouldNotReadHeader(String),
    /// Errors that is returned from `ConnectionManager` either preprocessing data or processing packet
    ConnectionError(ConnectionManagerError),
}

impl Display for ErrorKind {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::DecodingError(e) => write!(
                fmt,
                "Something went wrong with parsing the header. Reason: {:?}.",
                e
            ),
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
            ErrorKind::ConnectionError(err) => write!(
                fmt,
                "Something went wrong in ConnectionManager. Reason: {:?}.",
                err
            ),
        }
    }
}

impl Error for ErrorKind {}

/// Errors that could occur while parsing packet contents
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DecodingErrorKind {
    /// The [PacketType] could not be read
    PacketType,
    /// The [OrderingGuarantee] could not be read
    OrderingGuarantee,
    /// The [DeliveryGuarantee] could not be read
    DeliveryGuarantee,
}

impl Display for DecodingErrorKind {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            DecodingErrorKind::PacketType => write!(fmt, "The packet type could not be read."),
            DecodingErrorKind::OrderingGuarantee => {
                write!(fmt, "The ordering guarantee could not be read.")
            }
            DecodingErrorKind::DeliveryGuarantee => {
                write!(fmt, "The delivery guarantee could not be read.")
            }
        }
    }
}

/// Errors that could occur while parsing packet contents
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PacketErrorKind {
    /// The maximal allowed size of the packet was exceeded
    ExceededMaxPacketSize,
    /// Only user packets (a.k.a PacketType::Packet) can be fragmented
    PacketTypeCannotBeFragmented,
}

impl Display for PacketErrorKind {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            PacketErrorKind::ExceededMaxPacketSize => {
                write!(fmt, "The packet size was bigger than the max allowed size.")
            }
            PacketErrorKind::PacketTypeCannotBeFragmented => write!(
                fmt,
                "Only user packets (PacketType::Packet) can be fragmented."
            ),
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

impl From<SendError<ConnectionEvent<SendEvent>>> for ErrorKind {
    fn from(inner: SendError<ConnectionEvent<SendEvent>>) -> Self {
        ErrorKind::SendError(SendError(Either::Left(inner.0)))
    }
}

impl From<SendError<ConnectionEvent<ReceiveEvent>>> for ErrorKind {
    fn from(inner: SendError<ConnectionEvent<ReceiveEvent>>) -> Self {
        ErrorKind::SendError(SendError(Either::Right(inner.0)))
    }
}

impl From<ConnectionManagerError> for ErrorKind {
    fn from(inner: ConnectionManagerError) -> Self {
        ErrorKind::ConnectionError(inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn able_to_box_errors() {
        let _: Box<dyn Error> = Box::new(ErrorKind::CouldNotReadHeader("".into()));
    }
}
