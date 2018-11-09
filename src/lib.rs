//! Laminar semi-reliable UDP protocol for multiplayer games. This library implements wraps around a UDP
//! and provides light weight stream based interface that provides certain guarentees like reliablity.
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
//! ```rust
//! extern crate laminar;
//!
//! use laminar::{UdpSocket, NetworkConfig};
//! use laminar::Packet;
//!
//! use std::net::Ipv4Addr;
//!
//! fn main() {
//!   let addr = "127.0.0.1:12345".parse().unwrap();
//!
//!   let mut socket = UdpSocket::bind(addr, NetworkConfig::default()).unwrap();
//!
//!   let data = "example data".as_bytes();
//!   let packet: Packet = Packet::reliable_unordered(addr, data.to_vec());
//!
//!   socket.send(packet).unwrap();
//!
//!   let data = socket.recv().unwrap();
//!   println!("{:?}", data);
//! }
//! ```

#![cfg_attr(feature = "cargo-clippy", deny(needless_return))]
#![warn(missing_docs)]
#![deny(unused_imports)]


extern crate bincode;
extern crate byteorder;
extern crate crc;
#[macro_use]
extern crate lazy_static;
extern crate serde;

#[macro_use]
extern crate log;

extern crate failure;
#[macro_use]
extern crate failure_derive;
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
