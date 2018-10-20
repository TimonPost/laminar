use super::{FragmentErrorKind, PacketErrorKind, TcpErrorKind};

use std::fmt::{self, Display,Formatter};
use std::io;

use failure::{Fail, Backtrace, Context};

#[derive(Fail, Debug)]
/// Enum with all possible network errors that could occur.
pub enum NetworkErrorKind
{
    #[fail(display = "Something went wrong with receiving/parsing fragments. Reason: {:?}.", inner)]
    FragmentError { inner: FragmentErrorKind },
    #[fail(display = "Something went wrong with receiving/parsing packets. Reason: {:?}.", inner)]
    PacketError { inner: PacketErrorKind },
    #[fail(display = "Something went wrong with TCP. Reason: {:?}.", inner)]
    TcpError { inner: TcpErrorKind},
    #[fail(display = "Could not add a connection to the connection pool, because the connection lock is poisoned. Reason: {:?}.", inner)]
    FailedToAddConnection { inner: String },
    #[fail(display = "Ans Io Error occurred. Reason: {:?}.", inner )]
    IOError { inner:  io::Error },
    #[fail(display = "Something went wrong when setting non-blocking option.")]
    UnableToSetNonblocking,
    #[fail(display = "Something went wrong when creating UDP SocketState structure.")]
    UDPSocketStateCreationFailed,
    #[fail(display = "The received data did not have any length.")]
    ReceivedDataToShort,
    #[fail(display = "The protocol versions do not match.")]
    ProtocolVersionMismatch,
}

#[derive(Debug)]
/// An error that could occur during network operations.
pub struct NetworkError
{
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
        NetworkErrorKind::FailedToAddConnection { inner: msg.to_owned() }
    }
}

impl From<NetworkErrorKind> for NetworkError {
    fn from(kind: NetworkErrorKind) -> NetworkError {
        NetworkError { inner: Context::new(kind) }
    }
}

impl From<Context<NetworkErrorKind>> for NetworkError {
    fn from(inner: Context<NetworkErrorKind>) -> NetworkError {
        NetworkError { inner }
    }
}

impl From<io::Error> for NetworkError {
    fn from(inner: io::Error) -> NetworkError {
        NetworkErrorKind::IOError { inner }.into()
    }
}

impl From<FragmentErrorKind> for NetworkError {
    fn from(inner: FragmentErrorKind) -> Self {
        NetworkErrorKind::FragmentError { inner }.into()
    }
}

impl From<PacketErrorKind> for NetworkError {
    fn from(inner: PacketErrorKind) -> Self {
        NetworkErrorKind::PacketError { inner }.into()
    }
}

impl From<TcpErrorKind> for NetworkError {
    fn from(inner: TcpErrorKind) -> Self {
        NetworkErrorKind::TcpError { inner }.into()
    }
}