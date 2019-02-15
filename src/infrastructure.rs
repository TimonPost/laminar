mod channels;
///! This module provides infrastructure logic. With infrastructure is meant, everything that's responsible for the packet flow and processing.
mod delivery_method;
mod fragmenter;

pub mod arranging;

pub use self::channels::Channel;
pub use self::channels::{ReliableChannel, SequencedChannel, UnreliableChannel};
pub use self::delivery_method::DeliveryMethod;
pub use self::fragmenter::Fragmentation;
