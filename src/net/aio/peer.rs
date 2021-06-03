use super::channel::BidirectionalAsyncChannel;
use std::fmt;
use std::ops::Deref;

/// A bidirectional channel for binary messages.
#[derive(Clone)]
pub struct Peer(BidirectionalAsyncChannel<Box<[u8]>>);

impl Peer {
    /// Creates a pair of connected Peers without limitations on how many messages can be
    /// buffered.
    pub fn create_unbounded_pair() -> (Self, Self) {
        let (a, b) = BidirectionalAsyncChannel::create_unbounded_pair();
        (Self(a), Self(b))
    }

    /// Creates a pair of connected Peers with a limited capacity for many messages can be
    /// buffered in either direction.
    pub fn create_bounded_pair(capacity: usize) -> (Self, Self) {
        let (a, b) = BidirectionalAsyncChannel::create_bounded_pair(capacity);
        (Self(a), Self(b))
    }
}

impl fmt::Debug for Peer {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Peer {{ connected: {} }}", self.is_connected())
    }
}

impl Deref for Peer {
    type Target = BidirectionalAsyncChannel<Box<[u8]>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod test {
    use super::*;
    static_assertions::assert_impl_all!(Peer: Deref, Clone, Send, Sync);
}
