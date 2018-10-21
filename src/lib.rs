//! Amethysts networking protocol

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

pub mod error;
pub mod events;
pub mod net;
pub mod packet;
pub mod protocol_version;
mod sequence_buffer;
pub mod infrastructure;

/// This functions checks how many times a number fits into another number and will round up.
///
/// For example we have two numbers:
/// - number 1 = 4000;
/// - number 2 = 1024;
/// If you do it the easy way the answer will be 4000/1024 = 3.90625.
/// But since we care about how how many whole times the number fits in we need the result 4.
///
/// Note that when rust is rounding it is always rounding to zero (3.456 as u32 = 3)
/// 1. calculate with modulo if `number 1` fits exactly in the `number 2`.
/// 2. Divide `number 1` with `number 2` (this wil be rounded to zero by rust)
/// 3. So in all cases we need to add 1 to get the right amount of fragments.
///
/// lets take an example
///
/// Calculate modules:
/// - number 1 % number 2 = 928
/// - this is bigger than 0 so remainder = 1
///
/// Calculate how many times the `number 1` fits in `number 2`:
/// - number 1 / number 2 = 3,90625 (this will be rounded to 3)
/// - add remainder from above to 3 = 4.
///
/// The above described method will figure out for all number how many times it fits into another number rounded up.
///
/// So an example of dividing an packet of bytes we get these fragments:
///
/// So for 4000 bytes we need 4 fragments
/// [fragment: 1024] [fragment: 1024] [fragment: 1024] [fragment: 928]
fn total_fragments_needed(payload_length: u16, fragment_size: u16) -> u16 {
    let remainder = if payload_length % fragment_size > 0 { 1 } else { 0 };
    ((payload_length / fragment_size) + remainder)
}