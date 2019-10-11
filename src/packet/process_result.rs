use std::collections::VecDeque;

use crate::either::Either;
use crate::packet::{OutgoingPacket, Packet, PacketType};

/// Struct that implements `Iterator`, and is used to return incoming (from bytes to packets) or outgoing (from packet to bytes) packets.
/// It is used as optimization in cases, where most of the time there is only one element to iterate, and we don't want to create a vector for it.
#[derive(Debug)]
pub struct ZeroOrMore<T> {
    data: Either<Option<T>, VecDeque<T>>,
}

impl<T> ZeroOrMore<T> {
    fn zero() -> Self {
        Self {
            data: Either::Left(None),
        }
    }

    fn one(data: T) -> Self {
        Self {
            data: Either::Left(Some(data)),
        }
    }

    fn many(vec: VecDeque<T>) -> Self {
        Self {
            data: Either::Right(vec),
        }
    }
}

impl<T> Iterator for ZeroOrMore<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.data {
            Either::Left(option) => option.take(),
            Either::Right(vec) => vec.pop_front(),
        }
    }
}

/// Stores packets with headers that will be sent to the network, implements `IntoIterator` for convenience.
#[derive(Debug)]
pub struct OutgoingPackets<'a> {
    data: ZeroOrMore<OutgoingPacket<'a>>,
}

impl<'a> OutgoingPackets<'a> {
    /// Stores only one packet, without allocating on the heap.
    pub fn one(packet: OutgoingPacket<'a>) -> Self {
        Self {
            data: ZeroOrMore::one(packet),
        }
    }

    /// Stores multiple packets, allocated on the heap.
    pub fn many(packets: VecDeque<OutgoingPacket<'a>>) -> Self {
        Self {
            data: ZeroOrMore::many(packets),
        }
    }
}

impl<'a> IntoIterator for OutgoingPackets<'a> {
    type Item = OutgoingPacket<'a>;
    type IntoIter = ZeroOrMore<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.data
    }
}

/// Stores parsed packets with their types, that was received from network, implements `IntoIterator` for convenience.
#[derive(Debug)]
pub struct IncomingPackets {
    data: ZeroOrMore<(Packet, PacketType)>,
}

impl IncomingPackets {
    /// No packets are stored
    pub fn zero() -> Self {
        Self {
            data: ZeroOrMore::zero(),
        }
    }

    /// Stores only one packet, without allocating on the heap.
    pub fn one(packet: Packet, packet_type: PacketType) -> Self {
        Self {
            data: ZeroOrMore::one((packet, packet_type)),
        }
    }

    /// Stores multiple packets, allocated on the heap.
    pub fn many(vec: VecDeque<(Packet, PacketType)>) -> Self {
        Self {
            data: ZeroOrMore::many(vec),
        }
    }
}

impl IntoIterator for IncomingPackets {
    type Item = (Packet, PacketType);
    type IntoIter = ZeroOrMore<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.data
    }
}
