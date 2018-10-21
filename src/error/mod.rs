mod error_kinds;
mod network_error;

pub use self::network_error::{NetworkErrorKind, NetworkError};
pub use self::error_kinds::{PacketErrorKind, FragmentErrorKind};

use std::result;
use failure::Error;

pub type Result<T> = result::Result<T, Error>;

/// ```text
/// use laminar::error::{NetworkError, NetworkErrorKind, FragmentErrorKind};
/// use laminar::net::{UdpSocket, NetworkConfig};
///
/// let mut udp_socket: UdpSocket = UdpSocket::bind(self.host, NetworkConfig::default()).unwrap();
/// let result: Result<Option<Packet>, NetworkError> = udp_socket.recv();
///
///  match result {
///      Err(error) => {
///         match *error.kind() {
///             NetworkErrorKind::FragmentError { inner } => {},
///             NetworkErrorKind::PacketError { inner } => {},
///             NetworkErrorKind::TcpError { inner } => {},
///             NetworkErrorKind::FailedToAddConnection { inner } => {},
///             NetworkErrorKind::IOError { inner } => { },
///             NetworkErrorKind::UnableToSetNonblocking => {},
///             NetworkErrorKind::UDPSocketStateCreationFailed => {},
///             NetworkErrorKind::ReceivedDataToShort => {},
///         }
///      }
///      _ => {}
///  }
/// ```
pub type NetworkResult<T> = result::Result<T, NetworkError>;


