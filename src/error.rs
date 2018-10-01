use failure;
use std::result;

pub type Error = failure::Error;
pub type Result<T> = result::Result<T, Error>;

#[derive(Fail, Debug)]
pub enum NetworkError {
    // TODO: write more informative error
    #[fail(display = "Lock posioned")]
    AddConnectionToManagerFailed,
    #[fail(display = "TcpStream clone failed")]
    TcpStreamCloneFailed,
    #[fail(display = "TcpStream failed to take the rx channel in outgoing loop")]
    TcpSteamFailedTakeRx,
    #[fail(display = "Failed to get client lock to start outgoing loop")]
    TcpStreamFailedClientLock,
    #[fail(display = "TCP client connections hash was poisoned")]
    TcpClientConnectionsHashPoisoned
}
