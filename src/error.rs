///! Module contains error handling and error types for laminar
mod error_kinds;
mod network_error;

pub use self::error_kinds::{FragmentErrorKind, PacketErrorKind};
pub use self::network_error::{NetworkError, NetworkErrorKind};

use std::result;

/// Wrapped result type for Laminar errors.
pub type NetworkResult<T> = result::Result<T, NetworkError>;
