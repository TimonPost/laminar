//! This module provides the logic between the low-level abstract types and the types that the user will be interacting with.
//! You can think of the socket, connection management, congestion control.

mod connection;
mod link_conditioner;
mod quality;
mod socket;
mod virtual_connection;

pub mod constants;
pub mod events;
pub mod managers;

pub use self::link_conditioner::LinkConditioner;
pub use self::quality::{NetworkQuality, RttMeasurer};
pub use self::socket::{Socket, SocketEventSender};
pub use self::virtual_connection::VirtualConnection;
