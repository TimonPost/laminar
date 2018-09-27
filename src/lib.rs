//! Amethysts networking protocol

extern crate bincode;
extern crate serde;

#[macro_use]
extern crate serde_derive;

mod net;
mod packet;

pub mod amethyst_error;
pub mod connection;
pub mod server;
pub mod events;

pub use net::udp::UdpSocket;
use packet::{Packet, RawPacket};
