//! Laminar semi-reliable UDP protocol for multiplayer games. This library implements wraps around a UDP
//! and provides light weight stream based interface that provides certain guarantees like reliability.
//!
//! Laminar was designed to be used within the [Amethyst][amethyst] game engine.
//!
//! [amethyst]: https://github.com/amethyst/amethyst
//!
//! # Concepts
//!
//! This library is mostly based off of [Gaffer on Games][gog] and shares features with RakNet. The idea is to provide a low level
//! UDP protocol that supports the use cases of video games that require multilayer features. The library
//! itself provides a few low level types of packets that provides different types of guarantees. The most
//! basic are unreliable and reliable packets. This generally correlates to state update packets that do not
//! require to be synced, meaning the packet can get dropped without harm to the game. The other is used for
//! example score updates, where you want to make sure that the data is received on the other end even in case
//! of a packet drop. For more information, read the projects [README.md][readme]
//!
//! [gog]: https://gafferongames.com/
//! [readme]: https://github.com/amethyst/laminar/blob/master/README.md
//!
//! # Example
//!

#![warn(missing_docs)]

mod infrastructure;
mod packet;
mod protocol_version;
mod sequence_buffer;

/// Contains networking related configuration
mod config;
/// All internal error handling logic
mod error;
/// Networking modules
mod net;

pub use self::config::Config;
pub use self::error::{ErrorKind, Result};
pub use self::infrastructure::DeliveryMethod;
pub use self::net::Socket;
pub use self::net::SocketEvent;
pub use self::net::VirtualConnection;
pub use self::packet::Packet;
pub use self::protocol_version::ProtocolVersion;
