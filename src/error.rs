use failure;
use std::io::ErrorKind;
use std::result;

pub type Error = failure::Error;
pub type Result<T> = result::Result<T, Error>;

#[derive(Fail, Debug)]
pub enum NetworkError {
    // TODO: write more informative error
    #[fail(display = "Lock poisoned")]
    AddConnectionToManagerFailed,
    #[fail(display = "TcpStream clone failed")]
    TcpStreamCloneFailed,
    #[fail(display = "TcpStream failed to take the rx channel in outgoing loop")]
    TcpSteamFailedTakeRx,
    #[fail(display = "TCP client connections hash was poisoned")]
    TcpClientConnectionsHashPoisoned,
    #[fail(display = "The lock for a specific TCP client was poisoned")]
    TcpClientLockFailed,
    #[fail(display = "The fragment of an packet is invalid")]
    InvalidFragmentHeader,
    #[fail(display = "The parsing of the header went wrong")]
    HeaderParsingFailed,
    #[fail(display = "Something went wrong when sending")]
    SendFailed,
    #[fail(display = "Something went wrong when receiving")]
    ReceiveFailed,
    #[fail(display = "The packet size was bigger than the max allowed size.")]
    ExceededMaxPacketSize,
    #[fail(display = "The total of fragments the packet was divided into is bigger than the allowed fragments.")]
    ExceededMaxFragments,
    #[fail(display = "The packet header is invalid.")]
    InvalidPacketHeader,
    #[fail(display = "Error type for wrapping an io error")]
    IoError { kind: ErrorKind, msg: String },
    #[fail(display = "Unable to create UDP SocketState structure")]
    UDPSocketStateCreationFailed,
    #[fail(display = "Unable to set nonblocking option")]
    UnableToSetNonblocking,
}
