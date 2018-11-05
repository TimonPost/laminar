///! This module provides infrastructure logic. With infrastructure is meant, everything that's responsible for the packet flow and processing.

mod delivery_method;
mod fragmenter;
mod channels;

pub use self::channels::{ReliableChannel, UnreliableChannel, SequencedChannel};
pub use self::delivery_method::DeliveryMethod;
pub use self::fragmenter::Fragmentation;
pub use self::channels::Channel;
