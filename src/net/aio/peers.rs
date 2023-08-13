use super::Peer;
use core::hash::Hash;
use dashmap::DashMap;

/// A keyed mapping of [Peer]s with ownership semantics.
///
/// Dropping will disconnect all owned peers.
///
/// [Peer]: crate::Peer
#[derive(Debug)]
pub struct Peers<T>(DashMap<T, Peer>)
where
    T: Eq + Hash;

impl<T: Eq + Hash> Peers<T> {
    /// Gets a [Peer] by it's ID, if available.
    ///
    /// [Peer]: crate::Peer
    pub fn get(&self, id: &T) -> Option<Peer> {
        self.0.get(&id).and_then(|kv| {
            let peer = kv.value().clone();
            if peer.is_connected() {
                Some(peer)
            } else {
                None
            }
        })
    }

    /// Gets the number of active connections managed by it.
    pub fn len(&self) -> usize {
        self.0.iter().filter(|kv| kv.value().is_connected()).count()
    }

    /// Checks if the store has a connection to the given ID.
    pub fn contains(&self, id: &T) -> bool {
        self.0
            .get(&id)
            .map(|kv| kv.value().is_connected())
            .unwrap_or(false)
    }

    /// Creates a new unbounded peer pair and stores one end, mapping it to the provided ID,
    /// returning the other end.
    ///
    /// If a peer was previous stored at the given ID, it will be replaced and disconnected.
    #[must_use]
    pub fn create_unbounded(&self, id: T) -> Peer {
        let (a, b) = Peer::create_unbounded_pair();
        if let Some(prior) = self.0.insert(id, a) {
            prior.disconnect();
        }
        b
    }

    /// Creates an bounded peer pair and stores one end, mapping it to the provided ID, returning
    /// the other end.
    ///
    /// If a peer was previous stored at the given ID, it will be dropped and replaced.
    #[must_use]
    pub fn create_bounded(&self, id: T, capacity: usize) -> Peer {
        let (a, b) = Peer::create_bounded_pair(capacity);
        self.0.insert(id, a);
        b
    }

    /// Disconnects and removes a connection by it's ID
    ///
    /// A no-op if there no Peer with the given ID.
    pub fn disconnect(&self, id: &T) {
        if let Some((_, peer)) = self.0.remove(&id) {
            peer.disconnect();
        }
    }

    /// Removes all peers that are disconnected.
    pub fn flush_disconnected(&self) {
        self.0.retain(|_, peer| peer.is_connected())
    }
}

impl<T: Eq + Hash> Default for Peers<T> {
    fn default() -> Self {
        Self(DashMap::<T, Peer>::new())
    }
}

impl<T: Eq + Hash> Drop for Peers<T> {
    fn drop(&mut self) {
        for kv in self.0.iter() {
            kv.value().disconnect();
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    static_assertions::assert_impl_all!(Peers<i32>: Default, Drop, Send, Sync);

    #[test]
    pub fn test_contains_works() {
        const ID: i32 = 420;
        let peers = Peers::<i32>::default();
        let _peer = peers.create_unbounded(ID);
        assert!(peers.contains(&ID));
        assert!(peers.get(&ID).is_some());
    }

    #[test]
    pub fn disconnecting_removes_peer() {
        const ID: i32 = 420;
        let peers = Peers::<i32>::default();
        let peer = peers.create_unbounded(ID);
        assert!(peers.contains(&ID));
        assert!(peers.get(&ID).is_some());
        peer.disconnect();
        assert!(!peers.contains(&ID));
        assert!(peers.get(&ID).is_none());
    }

    #[test]
    pub fn disconnecting_via_drop_removes_peer() {
        const ID: i32 = 420;
        let peers = Peers::<i32>::default();
        let peer = peers.create_unbounded(ID);
        assert!(peers.contains(&ID));
        assert!(peers.get(&ID).is_some());
        drop(peer);
        assert!(!peers.contains(&ID));
        assert!(peers.get(&ID).is_none());
    }

    #[test]
    pub fn disconnecting_local_disconnects_remote() {
        const ID: i32 = 420;
        let peers = Peers::<i32>::default();
        let peer_remote = peers.create_unbounded(ID);
        peers.disconnect(&ID);
        assert!(!peer_remote.is_connected());
    }

    #[test]
    pub fn dropping_disconnects_all_remotes() {
        let peers = Peers::<i32>::default();
        let a = peers.create_unbounded(1);
        let b = peers.create_unbounded(2);
        let c = peers.create_unbounded(3);

        assert!(a.is_connected());
        assert!(b.is_connected());
        assert!(c.is_connected());
        drop(peers);
        assert!(!a.is_connected());
        assert!(!b.is_connected());
        assert!(!c.is_connected());
    }
}
