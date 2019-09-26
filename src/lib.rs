//! Laminar is a semi-reliable UDP-based protocol for multiplayer games. This library implements wrappers around the UDP-protocol,
//! and provides a lightweight, message-based interface which provides certain guarantees like reliability and ordering.
//!
//! Laminar was designed to be used within the [Amethyst][amethyst] game engine but is usable without it.
//!
//! [amethyst]: https://github.com/amethyst/amethyst
//!
//! # Concepts
//!
//! This library is loosely based off of [Gaffer on Games][gog] and has features similar to RakNet, Steam Socket, and netcode.io.
//! The idea is to provide a native Rust low-level UDP-protocol which supports the use of cases of video games that require multiplayer features.
//! The library itself provides a few low-level types of packets that provide different types of guarantees. The most
//! basic are unreliable and reliable packets. Ordering, sequencing can be done on multiple streams.
//! For more information, read the projects [README.md][readme], [book][book], [docs][docs] or [examples][examples].
//!
//! [gog]: https://gafferongames.com/
//! [readme]: https://github.com/amethyst/laminar/blob/master/README.md
//! [book]: https://github.com/amethyst/laminar/tree/master/docs/md_book
//! [docs]: https://docs.rs/laminar/
//! [examples]: https://github.com/amethyst/laminar/tree/master/examples

#![warn(missing_docs)]
#![allow(clippy::trivially_copy_pass_by_ref)]

mod config;
mod either;
mod error;
mod infrastructure;
mod net;
mod packet;
mod protocol_version;
mod sequence_buffer;

#[cfg(feature = "tester")]
mod throughput;

#[cfg(feature = "tester")]
pub use self::throughput::ThroughputMonitoring;

#[cfg(test)]
pub mod test_utils;

pub use self::config::Config;
pub use self::error::{ErrorKind, Result};
pub use self::net::{Socket, SocketEvent};
pub use self::packet::{DeliveryGuarantee, OrderingGuarantee, Packet};
