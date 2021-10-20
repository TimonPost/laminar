//! This module provides the logic around the processing of the packet.
//! Like ordering, sequencing, controlling congestion, fragmentation, and packet acknowledgment.

pub use self::acknowledgment::AcknowledgmentHandler;
pub use self::acknowledgment::SentPacket;
pub use self::fragmenter::Fragmentation;

mod acknowledgment;
mod fragmenter;

pub mod arranging;
