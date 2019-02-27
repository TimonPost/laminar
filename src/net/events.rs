use crate::packet::{Fragment, Packet};
use std::net::SocketAddr;

/// Events which will be pushed through the event_receiver returned by `Socket::bind`.
#[derive(Debug, PartialEq)]
pub enum SocketEvent {
    /// A packet has been received from a client.
    Packet(Packet),
    /// An fragmentation has completed and this contains the completed data.
    Fragment(Fragment),
    /// A new client connects. Clients are uniquely identified by the ip:port combination at this layer.
    Connect(SocketAddr),
    /// A client disconnects. This is generated from the server-side intentionally disconnecting a client,
    /// or it could be from the client disconnecting.
    Disconnect(SocketAddr),
    /// This is generated if the server has not seen traffic from a client after a configurable amount of time.
    Timeout(SocketAddr),
}
