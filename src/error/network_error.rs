use super::{FragmentErrorKind, PacketErrorKind};

use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io;
use std::sync::PoisonError;

use failure::{Backtrace, Context, Fail};

#[derive(Fail, Debug)]
/// Enum with all possible network errors that could occur.
pub enum NetworkErrorKind {
    #[fail(
        display = "Something went wrong with receiving/parsing fragments. Reason: {:?}.",
        _0
    )]
    /// Error relating to receiving or parsing a fragment
    FragmentError(FragmentErrorKind),
    #[fail(
        display = "Something went wrong with receiving/parsing packets. Reason: {:?}.",
        _0
    )]
    /// Error relating to receiving or parsing a packet
    PacketError(PacketErrorKind),
    #[fail(
        display = "Could not add a connection to the connection pool, because the connection lock is poisoned. Reason: {:?}.",
        _0
    )]
    /// Failed to add a connection
    FailedToAddConnection(String),
    #[fail(display = "An IO Error occurred. Reason: {:?}.", _0)]
    /// Wrapper around a std io::Error
    IOError(io::Error),
    #[fail(display = "Something went wrong when setting non-blocking option.")]
    /// Error setting nonblocking on a udp server
    UnableToSetNonblocking,
    #[fail(display = "Something went wrong when creating UDP SocketState structure.")]
    /// Error when creating the UDP Socket State
    UDPSocketStateCreationFailed,
    #[fail(display = "The received data did not have any length.")]
    /// Did not receive enough data
    ReceivedDataToShort,
    #[fail(display = "The protocol versions do not match.")]
    /// Protocol versions did not match
    ProtocolVersionMismatch,
    #[fail(
        display = "Something went wrong with connection timeout thread. Reason: {:?}",
        _0
    )]
    /// Error occurred in connection pool.
    ConnectionPoolError(String),
    #[fail(display = "Joining thread failed.")]
    /// Error occurred when joining thread.
    JoiningThreadFailed,
    #[fail(display = "There was an unexpected error caused by an poisoned lock.")]
    /// There was an unexpected error caused by an poisoned lock.
    PoisonedLock(String),
}

#[derive(Debug)]
/// An error that could occur during network operations.
pub struct NetworkError {
    inner: Context<NetworkErrorKind>,
}

impl Fail for NetworkError {
    /// Returns a reference to the underlying cause of this failure, if it
    /// is an error that wraps other errors.
    ///
    /// Returns `None` if this failure does not have another error as its
    /// underlying cause. By default, this returns `None`.
    ///
    /// This should **never** return a reference to `self`, but only return
    /// `Some` when it can return a **different** failure. Users may loop
    /// over the cause chain, and returning `self` would result in an infinite
    /// loop.
    fn cause(&self) -> Option<&Fail> {
        self.inner.cause()
    }

    /// Returns a reference to the `Backtrace` carried by this failure, if it
    /// carries one.
    ///
    /// Returns `None` if this failure does not carry a backtrace. By
    /// default, this returns `None`.
    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for NetworkError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl NetworkError {
    /// Get the error kind from the error. This is useful when you want to match on the error kind.
    pub fn kind(&self) -> &NetworkErrorKind {
        self.inner.get_context()
    }

    /// Generate an `NetworkErrorKind` for poisoned connection.
    pub fn poisoned_connection_error(msg: &str) -> NetworkErrorKind {
        NetworkErrorKind::FailedToAddConnection(msg.to_owned())
    }
}

impl From<NetworkErrorKind> for NetworkError {
    fn from(kind: NetworkErrorKind) -> NetworkError {
        NetworkError {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<NetworkErrorKind>> for NetworkError {
    fn from(inner: Context<NetworkErrorKind>) -> NetworkError {
        NetworkError { inner }
    }
}

impl From<io::Error> for NetworkError {
    fn from(inner: io::Error) -> NetworkError {
        NetworkErrorKind::IOError(inner).into()
    }
}

impl From<FragmentErrorKind> for NetworkError {
    fn from(inner: FragmentErrorKind) -> Self {
        NetworkErrorKind::FragmentError(inner).into()
    }
}

impl From<PacketErrorKind> for NetworkError {
    fn from(inner: PacketErrorKind) -> Self {
        NetworkErrorKind::PacketError(inner).into()
    }
}

impl<T> From<PoisonError<T>> for NetworkError {
    fn from(inner: PoisonError<T>) -> Self {
        NetworkErrorKind::FailedToAddConnection(inner.description().to_owned()).into()
    }
}
