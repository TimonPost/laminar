//! Laminar semi-reliable UDP protocol for multiplayer games. This library just implements the low level
//! aspects of a UDP socket. It provides light weight wrappers and basic stream based functionality.
//!
//! Laminar was designed to be used within the [Amethyst][amethyst] game engine.
//!
//! [amethyst]: https://github.com/amethyst/amethyst
//!
//! # Concepts
//!
//! This library is mostly based off of [Gaffer on Games][gog] and RakNet. The idea is provide a low level
//! UDP protocol that supports the use cases of video games that require multiplayer features. The library
//! itself provides a few low level types of packets that provides different types of guarentees. The most
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
//! ```rust
//! use laminar::{UdpSocket, NetworkConfig};
//! use laminar::Packet;
//! use std::net::Ipv4Addr;
//!
//! fn main() {
//!   let addr = "127.0.0.1:12345".parse().unwrap();
//!
//!   let mut socket = UdpSocket::bind(addr, NetworkConfig::default()).unwrap();
//!
//!   let data = "example data".as_bytes();
//!   let packet: Packet = Packet::sequenced_unordered(addr, data.to_vec());
//!
//!   socket.send(packet).unwrap();
//! }
//! ```

extern crate bincode;
extern crate byteorder;
extern crate failure;
extern crate serde;
#[macro_use]
extern crate log;
#[macro_use]
extern crate failure_derive;
extern crate crc;
#[macro_use]
extern crate lazy_static;
extern crate rand;

pub mod error;
pub mod events;
pub mod infrastructure;
pub mod net;
pub mod packet;
pub mod protocol_version;
mod sequence_buffer;

pub use net::{NetworkConfig, UdpSocket};
pub use packet::Packet;
