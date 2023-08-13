use async_channel::{TryRecvError, TrySendError};

#[derive(Clone)]
pub struct BidirectionalAsyncChannel<T> {
    incoming: async_channel::Receiver<T>,
    outgoing: async_channel::Sender<T>,
}

impl<T> BidirectionalAsyncChannel<T> {
    /// Creates a pair of connected Peers without limitations on how many messages can be
    /// buffered.
    pub fn create_unbounded_pair() -> (Self, Self) {
        Self::create_pair(async_channel::unbounded(), async_channel::unbounded())
    }

    /// Creates a pair of connected Peers with a limited capacity for many messages can be
    /// buffered in either direction.
    pub fn create_bounded_pair(capacity: usize) -> (Self, Self) {
        Self::create_pair(
            async_channel::bounded(capacity),
            async_channel::bounded(capacity),
        )
    }

    /// Sends a message to the connected peer.
    ///
    /// If the send buffer is full, this method waits until there is space for a message.
    ///
    /// If the peer is disconnected, this method returns an error.
    #[inline]
    pub fn send(&self, message: T) -> async_channel::Send<'_, T> {
        self.outgoing.send(message)
    }

    /// Receives a message from the connected peer.
    ///
    /// If there is no pending messages, this method waits until there is a message.
    ///
    /// If the peer is disconnected, this method receives a message or returns an error if there
    /// are no more messages.
    #[inline]
    pub fn recv(&self) -> async_channel::Recv<'_, T> {
        self.incoming.recv()
    }

    /// Attempts to send a message to the connected peer.
    #[inline]
    pub fn try_send(&self, message: T) -> Result<(), TrySendError<T>> {
        self.outgoing.try_send(message)
    }

    /// Attempts to receive a message from the connected peer.
    #[inline]
    pub fn try_recv(&self) -> Result<T, TryRecvError> {
        self.incoming.try_recv()
    }

    /// Returns true if the associated peer is still connected.
    pub fn is_connected(&self) -> bool {
        !self.incoming.is_closed() && !self.outgoing.is_closed()
    }

    /// Disconnects the paired Peers from either end. Any future attempts to send messages in
    /// either direction will fail, but any messages not yet recieved.
    ///
    /// If the Peer, or it's constituent channels were cloned, all of the cloned instances will
    /// appear disconnected.
    pub fn disconnect(&self) {
        self.outgoing.close();
        self.incoming.close();
    }

    /// Gets the raw sender for the peer.
    pub fn sender(&self) -> async_channel::Sender<T> {
        self.outgoing.clone()
    }

    /// Gets the raw reciever for the peer.
    pub fn reciever(&self) -> async_channel::Receiver<T> {
        self.incoming.clone()
    }

    /// The number of messages that are currently buffered in the send queue. Returns 0 if the
    /// channel is closed.
    pub fn pending_send_count(&self) -> usize {
        self.outgoing.len()
    }

    /// The number of messages that are currently buffered in the recieve queue. Returns 0 if the
    /// channel is closed.
    pub fn pending_recv_count(&self) -> usize {
        self.incoming.len()
    }

    fn create_pair(
        a: (async_channel::Sender<T>, async_channel::Receiver<T>),
        b: (async_channel::Sender<T>, async_channel::Receiver<T>),
    ) -> (Self, Self) {
        let (a_send, a_recv) = a;
        let (b_send, b_recv) = b;
        let a = Self {
            incoming: a_recv,
            outgoing: b_send,
        };
        let b = Self {
            incoming: b_recv,
            outgoing: a_send,
        };
        (a, b)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    static_assertions::assert_impl_all!(BidirectionalAsyncChannel<i32>: Clone);

    #[test]
    pub fn send_works_both_ways() {
        let (a, b) = BidirectionalAsyncChannel::<i32>::create_unbounded_pair();

        assert!(a.try_send(1).is_ok());
        assert!(b.try_send(4).is_ok());
        assert!(a.try_send(2).is_ok());
        assert!(b.try_send(5).is_ok());
        assert!(a.try_send(3).is_ok());
        assert!(b.try_send(6).is_ok());

        assert_eq!(a.pending_send_count(), 3);
        assert_eq!(b.pending_send_count(), 3);
        assert_eq!(a.pending_recv_count(), 3);
        assert_eq!(b.pending_recv_count(), 3);

        assert_eq!(a.try_recv(), Ok(4));
        assert_eq!(a.try_recv(), Ok(5));
        assert_eq!(a.try_recv(), Ok(6));

        assert_eq!(b.try_recv(), Ok(1));
        assert_eq!(b.try_recv(), Ok(2));
        assert_eq!(b.try_recv(), Ok(3));
    }

    #[test]
    pub fn bounded_pairs_error_on_being_full() {
        let (a, b) = BidirectionalAsyncChannel::<i32>::create_bounded_pair(2);

        assert!(a.try_send(1).is_ok());
        assert!(a.try_send(2).is_ok());
        assert!(matches!(a.try_send(3), Err(TrySendError::Full(3))));
        assert!(b.try_send(4).is_ok());
        assert!(b.try_send(5).is_ok());
        assert!(matches!(b.try_send(6), Err(TrySendError::Full(6))));

        assert_eq!(a.try_recv(), Ok(4));
        assert_eq!(a.try_recv(), Ok(5));
        assert_eq!(a.try_recv(), Err(TryRecvError::Empty));

        assert_eq!(b.try_recv(), Ok(1));
        assert_eq!(b.try_recv(), Ok(2));
        assert_eq!(a.try_recv(), Err(TryRecvError::Empty));
    }

    #[test]
    pub fn disconnecting_closes_both_sides() {
        let (a, b) = BidirectionalAsyncChannel::<i32>::create_bounded_pair(2);

        a.disconnect();
        assert!(!a.is_connected());
        assert!(!b.is_connected());

        let (a, b) = BidirectionalAsyncChannel::<i32>::create_bounded_pair(2);

        b.disconnect();
        assert!(!a.is_connected());
        assert!(!b.is_connected());
    }

    #[test]
    pub fn disconnecting_stop_any_future_sends() {
        let (a, b) = BidirectionalAsyncChannel::<i32>::create_bounded_pair(2);

        a.disconnect();
        assert!(!a.is_connected());
        assert!(!b.is_connected());

        assert!(matches!(a.try_send(1), Err(TrySendError::Closed(1))));
        assert!(matches!(b.try_send(1), Err(TrySendError::Closed(1))));
        assert!(matches!(a.try_recv(), Err(TryRecvError::Closed)));
        assert!(matches!(b.try_recv(), Err(TryRecvError::Closed)));
    }

    #[test]
    pub fn disconnecting_allows_existing_items_to_be_flushed() {
        let (a, b) = BidirectionalAsyncChannel::<i32>::create_unbounded_pair();

        assert!(a.try_send(1).is_ok());
        assert!(a.try_send(2).is_ok());
        a.disconnect();
        assert!(matches!(a.try_send(3), Err(TrySendError::Closed(3))));

        assert_eq!(b.try_recv(), Ok(1));
        assert_eq!(b.try_recv(), Ok(2));
        assert_eq!(b.try_recv(), Err(TryRecvError::Closed));
    }

    #[test]
    pub fn dropping_leads_to_disconnect() {
        let (a, b) = BidirectionalAsyncChannel::<i32>::create_unbounded_pair();

        assert!(a.is_connected());
        drop(b);
        assert!(!a.is_connected());

        let (a, b) = BidirectionalAsyncChannel::<i32>::create_unbounded_pair();
        let c = b.clone();

        assert!(a.is_connected());
        drop(b);
        assert!(a.is_connected());
        drop(c);
        assert!(!a.is_connected());
    }
}
