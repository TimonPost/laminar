//! This module provides the logic between the low-level abstract types and the types that the user will be interacting with.
//! You can think of the socket, connection management, congestion control.

pub use self::connection::{Connection, ConnectionFactory, ConnectionMessenger};
pub use self::connection_impl::ConnectionImpl;
pub use self::connection_manager::{ConnectionManager, DatagramSocket};
pub use self::events::SocketEvent;
pub use self::factory_impl::FactoryImpl;
pub use self::link_conditioner::LinkConditioner;
pub use self::quality::{NetworkQuality, RttMeasurer};
pub use self::socket::Socket;
pub use self::socket_with_conditioner::SocketWithConditioner;
pub use self::virtual_connection::VirtualConnection;

mod connection;
mod connection_impl;
mod connection_manager;
mod events;
mod factory_impl;
mod link_conditioner;
mod quality;
mod socket;
mod socket_with_conditioner;
mod virtual_connection;

pub mod constants;
