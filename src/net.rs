//! This module provides the logic between the low-level abstract types and the types that the user will be interacting with.
//! You can think of the socket, connection management, congestion control.

mod connection;
mod events;
mod link_conditioner;
mod quality;
mod socket;
mod virtual_connection;

pub mod constants;
pub mod managers;

pub use self::events::{SocketEvent, ConnectionSendEvent, ConnectionReceiveEvent };
pub use self::link_conditioner::LinkConditioner;
pub use self::quality::{NetworkQuality, RttMeasurer};
pub use self::socket::Socket;
pub use self::virtual_connection::VirtualConnection;
