use packet::Packet;
use laminar::error::NetworkErrorKind;
use laminar::events::Event;

/// Net event which occurred on the network.
pub enum NetEvent
{
    /// Event containing an packet with received data.
    Packet(Packet),
    /// Broad cast message.
    BroadCast { data: Box<[u8]>},
    /// Event containing error that has occurred in the network.
    Error(NetworkErrorKind),
    /// Events that can happen with an client.
    ClientEvent(Event),
    /// Empty event.
    Empty
}