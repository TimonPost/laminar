//! This module provides the logic between the low-level abstract types and the types that the user will be interacting with.
//! You can think of the socket, connection management, congestion control.

mod connection;
mod link_conditioner;
mod metrics_collector;
mod quality;
mod reliability_system;
mod socket;
mod virtual_connection;

pub mod constants;
pub mod events;
pub mod managers;

pub use self::link_conditioner::LinkConditioner;
pub use self::metrics_collector::MetricsCollector;
pub use self::quality::{NetworkQuality, RttMeasurer};
pub use self::reliability_system::{IncomingPackets, OutgoingPackets, ReliabilitySystem};
pub use self::socket::{Socket, SocketWithConditioner};
pub use self::virtual_connection::VirtualConnection;
