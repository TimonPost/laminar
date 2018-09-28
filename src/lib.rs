//! Amethysts networking protocol

extern crate bincode;
extern crate serde;

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;

mod net;
mod packet;

pub mod error;
pub mod events;

pub use net::udp::UdpSocket;
use packet::{Packet, RawPacket};
