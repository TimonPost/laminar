//! Amethysts networking protocol

extern crate bincode;
extern crate failure;
extern crate serde;

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
#[macro_use]
extern crate failure_derive;

pub mod net;
pub mod packet;

pub mod error;
pub mod events;
