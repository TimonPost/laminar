use crate::net::LinkConditioner;
use super::channel::BidirectionalAsyncChannel;
use futures_timer::Delay;
use bevy_tasks::TaskPool;
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

    /// Converts the peer into a conditioned one. All outgoing sends will be randomly dropped
    /// and have additional latency added based on the provided LinkConditioner.
    ///
    /// Useful for locally testing high latency or packet loss conditions.
    ///
    /// It is strongly advised not to use this in a release build as it might introduce
    /// unnecessary packet loss and latency.
    pub fn with_link_conditioner(self, pool: &TaskPool, conditioner: LinkConditioner) -> Self {
        let (a, b) = Self::create_unbounded_pair();
        pool.spawn(Self::conditioned_send(
                pool.clone(), b.reciever(), conditioner, self.sender())).detach();
        pool.spawn(super::forward(self.reciever(), b.sender())).detach();
        a
    }

    async fn conditioned_send(
        pool: TaskPool,
        input: async_channel::Receiver<Box<[u8]>>,
        mut conditioner: LinkConditioner,
        output: async_channel::Sender<Box<[u8]>>
    ) {
        while let Ok(message) = input.recv().await {
            if !conditioner.should_send() {
                continue;
            }

            if output.is_closed() {
                break;
            }

            let latency = conditioner.sample_latency();
            let output = output.clone();
            pool.spawn(async move {
                Delay::new(latency).await;
                output.send(message).await;
            });
        }
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
