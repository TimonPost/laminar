//! Amethysts networking protocol

extern crate bincode;
extern crate serde;

#[macro_use]
extern crate serde_derive;

mod net;
mod packet;


use packet::{Packet, RawPacket};

pub mod server;
pub mod connection;
pub mod amethyst_error;


pub use net::UdpSocket;
