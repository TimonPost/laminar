//! This module is about arranging items, over different streams, based on an certain algorithm.
//!
//! The above sentence contains a lot of important information, lets zoom in at the above sentence.
//!
//! ## Items
//!
//! By items, you can understand 'packets' and 'arranging' can be done based either with sequencing or ordering.
//!
//! ## Ordering VS Sequencing
//! Let's define two concepts here:
//! _"Sequencing: this is the process of only caring about the newest items."_ [1](https://dictionary.cambridge.org/dictionary/english/sequencing)
//! _"Ordering: this is the process of putting something in a particular order."_ [2](https://dictionary.cambridge.org/dictionary/english/ordering)
//!
//! - Sequencing: Only the newest items will be passed trough e.g. `1,3,2,5,4` which results in `1,3,5`.
//! - Ordering: All items are returned in order `1,3,2,5,4` which results in `1,2,3,4,5`.
//!
//! ## Arranging Streams
//! What are these 'arranging streams'?
//! You can see 'arranging streams' as something to arrange items that have no relationship at all with one another.
//!
//! ## Simple Example
//! Think of a highway where you have several lanes where cars are driving.
//! Because there are these lanes, cars can move on faster.
//! For example, the cargo drivers drive on the right and the high-speed cars on the left.
//! The cargo drivers have no influence on fast cars and vice versa.
//!
//! ## Real Example
//! If a game developer wants to send data to a client, it may be that he wants to send data ordered, unordered or sequenced.
//! Data might be the following:
//! 1. Player movement, we want to order player movements because we don't care about old positions.
//! 2. Bullet movement, we want to order bullet movement because we don't care about old positions of bullets.
//! 3. Chat messages, we want to order chat messages because it is nice to see the text in the right order.
//!
//! Player movement and chat messages are totally unrelated to each other and you absolutely do not want that movement packets are interrupted when a chat message is not sent.
//! With ordering, we can only return items when all packets up to the current package are received.
//!
//! So if a chat package is missing, the other packages will suffer from it.
//! It would be nice if we could order player movements and chat messages separately. This is exactly what `ordering streams` are meant for.
//! The game developer can indicate on which stream he can order his packets and how he wants to arrange them.
//! For example, the game developer can say: "Let me set all chat messages to 'stream 1' and all motion packets to 'stream 2'.

pub use self::ordering::{IterMut, OrderingStream, OrderingSystem};
pub use self::sequencing::{SequencingStream, SequencingSystem};

mod ordering;
mod sequencing;

/// A trait which can be implemented for arranging operations.
pub trait Arranging {
    type ArrangingItem;

    /// Arrange the given item based on the given index.
    /// If the `incoming_offset` somehow does not satisfies the arranging algorithm it returns `None`.
    /// If the `incoming_offset` satisfies the arranging algorithm it returns `Some` with the passed item.
    fn arrange(
        &mut self,
        incoming_index: u16,
        item: Self::ArrangingItem,
    ) -> Option<Self::ArrangingItem>;
}

/// An arranging system that has multiple streams on which you can arrange items.
pub trait ArrangingSystem {
    /// The type of stream that is used for arranging items.
    type Stream;

    /// Returns the number of streams currently created.
    fn stream_count(&self) -> usize;
    /// Try to get a `Stream` by `stream_id`. When the stream does not exist, it will be inserted by the given `stream_id` and returned.
    fn get_or_create_stream(&mut self, stream_id: u8) -> &mut Self::Stream;
}
