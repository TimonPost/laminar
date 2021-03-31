//! Module with logic for arranging items in-order on multiple streams.
//!
//! _"Order is the process of putting something in a particular order."_
//!
//! # How ordering works.
//! Imagine we have this sequence: `1,5,4,2,3` and we want the user to eventually see: `1,2,3,4,5`.
//!
//! Let's define some variables:
//!
//! ## Variable Setup
//! **hashmap**
//!
//! | Key     | Value |
//! | :-------------: | :-------------:    |
//! |       _       |        _         |
//!
//! `expected_index = 1;`
//!
//! ## Ordering
//! **Receive '1'**
//!
//! - Packet 1 is equals to our expected index we can return it immediately.
//! - Increase `expected_index` to '2'
//!
//! **Receive '5'**
//!
//! Packet '5' is not equal to our expected index so we need to store it until we received **all** packets up to 5 before returning.
//!
//! | Key     | Value |
//! | :-------------: | :-------------:    |
//! |       5       |        packet         |
//!
//! **Receive '4'**
//!
//! Packet '4' is not equal to our expected index so we need to store it until we received **all** packets up to 4 before returning.
//!
//! | Key     | Value |
//! | :-------------: | :-------------:    |
//! |       5       |        packet         |
//! |       4       |        packet         |
//!
//! **Receive '3'**
//!
//! Packet '3' is not equal to our expected index so we need to store it until we received **all** packets up to 3 before returning.
//!
//! | Key     | Value |
//! | :-------------: | :-------------:    |
//! |       5       |        packet         |
//! |       4       |        packet         |
//! |       4       |        packet         |
//!
//! **Receive '2'**
//!
//! - Packet 2 is equals to our expected index we can return it immediately.
//! - Increase `expected_index` to '3'
//!
//! Now we received our `expected_index` we can check if we have the next `expected_index` in storage.
//!
//! This could be done with an iterator which returns packets as long there are packets in our storage matching the `expected_index`.
//!
//! ```no-run
//! let stream = OrderingStream::new();
//!
//! let iter = stream.iter_mut();
//!
//! while let Some(packet) = iter.next() {
//!    // packets from iterator are in order.
//! }
//! ```
//!
//! # Remarks
//! - See [super-module](../index.html) description for more details.

use std::collections::HashMap;

use crate::packet::SequenceNumber;

use super::{Arranging, ArrangingSystem};

/// An ordering system that can arrange items in order on different streams.
///
/// Checkout [`OrderingStream`](./struct.OrderingStream.html), or module description for more details.
///
/// # Remarks
/// - See [super-module](../index.html) for more information about streams.
pub struct OrderingSystem<T> {
    // '[HashMap]' with streams on which items can be ordered.
    streams: HashMap<u8, OrderingStream<T>>,
}

impl<T> OrderingSystem<T> {
    /// Constructs a new [`OrderingSystem`](./struct.OrderingSystem.html).
    pub fn new() -> OrderingSystem<T> {
        OrderingSystem {
            streams: HashMap::with_capacity(32),
        }
    }
}

impl<'a, T> ArrangingSystem for OrderingSystem<T> {
    type Stream = OrderingStream<T>;

    /// Returns the number of ordering streams currently active.
    fn stream_count(&self) -> usize {
        self.streams.len()
    }

    /// Try to get an [`OrderingStream`](./struct.OrderingStream.html) by `stream_id`.
    /// When the stream does not exist, it will be inserted by the given `stream_id` and returned.
    fn get_or_create_stream(&mut self, stream_id: u8) -> &mut Self::Stream {
        self.streams
            .entry(stream_id)
            .or_insert_with(|| OrderingStream::new(stream_id))
    }
}

/// A stream on which items will be arranged in-order.
///
/// # Algorithm
///
/// With every ordering operation an `incoming_index` is given. We also keep a local record of the `expected_index`.
///
/// There are three scenarios that are important to us.
/// 1. `incoming_index` == `expected_index`.
/// This package meets the expected order, so we can return it immediately.
/// 2. `incoming_index` > `expected_index`.
/// This package is newer than we expect, so we have to hold it temporarily until we have received all previous packages.
/// 3. `incoming_index`< `expected_index`
/// This can only happen in cases where we have a duplicated package. Again we don't give anything back.
/// # Remarks
/// - See [super-module](../index.html) for more information about streams.
pub struct OrderingStream<T> {
    // The id of this stream.
    _stream_id: u8,
    // Storage with items that are waiting for older items to arrive.
    // Items are stored by key and value where the key is the incoming index and the value is the item value.
    storage: HashMap<u16, T>,
    // Next expected item index.
    expected_index: u16,
    // unique identifier which should be used for ordering on a different stream e.g. the remote endpoint.
    unique_item_identifier: u16,
}

impl<T> OrderingStream<T> {
    /// Constructs a new, empty [`OrderingStream<T>`](./struct.OrderingStream.html).
    ///
    /// The default stream will have a capacity of 32 items.
    pub fn new(stream_id: u8) -> OrderingStream<T> {
        OrderingStream::with_capacity(1024, stream_id)
    }

    /// Constructs a new, empty [`OrderingStream`] with the specified capacity.
    ///
    /// The stream will be able to hold exactly capacity elements without
    /// reallocating. If capacity is 0, the vector will not allocate.
    ///
    /// It is important to note that although the returned stream has the capacity specified,
    /// the stream will have a zero length.
    ///
    /// [`OrderingStream`]: ./struct.OrderingStream.html
    pub fn with_capacity(size: usize, stream_id: u8) -> OrderingStream<T> {
        OrderingStream {
            storage: HashMap::with_capacity(size),
            expected_index: 0,
            _stream_id: stream_id,
            unique_item_identifier: 0,
        }
    }

    /// Returns the identifier of this stream.
    #[cfg(test)]
    pub fn stream_id(&self) -> u8 {
        self._stream_id
    }

    /// Returns the next expected index.
    #[cfg(test)]
    pub fn expected_index(&self) -> u16 {
        self.expected_index
    }

    /// Returns the unique identifier which should be used for ordering on the other stream e.g. the remote endpoint.
    pub fn new_item_identifier(&mut self) -> SequenceNumber {
        let id = self.unique_item_identifier;
        self.unique_item_identifier = self.unique_item_identifier.wrapping_add(1);
        id
    }

    /// Returns an iterator of stored items.
    ///
    /// # Algorithm for returning items from an Iterator.
    ///
    /// 1. See if there is an item matching our `expected_index`
    /// 2. If there is return the `Some(item)`
    ///    - Increase the `expected_index`
    ///    - Start at '1'
    /// 3. If there isn't return `None`
    ///
    /// # Example
    ///
    /// ```ignore
    /// let stream = OrderingStream::new();
    ///
    /// let iter = stream.iter_mut();
    ///
    /// while let Some(item) = iter.next() {
    ///    // Items from iterator are in order.
    /// }
    /// ```
    ///
    /// # Remarks
    /// - Iterator mutates the `expected_index`.
    /// - You can't use this iterator for iterating trough all cached values.
    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut {
            items: &mut self.storage,
            expected_index: &mut self.expected_index,
        }
    }
}

fn is_u16_within_half_window_from_start(start: u16, incoming: u16) -> bool {
    // check (with wrapping) if the incoming value lies within the `next u16::max_value()/2` from start
    incoming.wrapping_sub(start) <= u16::max_value() / 2 + 1
}

impl<T> Arranging for OrderingStream<T> {
    type ArrangingItem = T;

    /// Orders the given item based on the ordering algorithm.
    ///
    /// With every ordering operation an `incoming_index` is given. We also keep a local record of the `expected_index`.
    ///
    /// # Algorithm
    ///
    /// There are three scenarios that are important to us.
    /// 1. `incoming_index` == `expected_index`.
    /// This package meets the expected order, so we can return it immediately.
    /// 2. `incoming_index` > `expected_index`.
    /// This package is newer than we expect, so we have to hold it temporarily until we have received all previous packages.
    /// 3. `incoming_index` < `expected_index`
    /// This can only happen in cases where we have a duplicated package. Again we don't give anything back.
    ///
    /// # Remark
    /// - When we receive an item there is a possibility that a gap is filled and one or more items will could be returned.
    ///   You should use the `iter_mut` instead for reading the items in order.
    ///   However the item given to `arrange` will be returned directly when it matches the `expected_index`.
    fn arrange(
        &mut self,
        incoming_offset: u16,
        item: Self::ArrangingItem,
    ) -> Option<Self::ArrangingItem> {
        if incoming_offset == self.expected_index {
            self.expected_index = self.expected_index.wrapping_add(1);
            Some(item)
        } else if is_u16_within_half_window_from_start(self.expected_index, incoming_offset) {
            self.storage.insert(incoming_offset, item);
            None
        } else {
            // only occurs when we get a duplicated incoming_offset.
            None
        }
    }
}

/// Mutable Iterator for [`OrderingStream<T>`](./struct.OrderingStream.html).
///
/// # Algorithm for returning items from Iterator.
///
/// 1. See if there is an item matching our `expected_index`
/// 2. If there is return the `Some(item)`
///    - Increase the `expected_index`
///    - Start at '1'
/// 3. If there isn't return `None`
///
/// # Remarks
///
/// - Iterator mutates the `expected_index`.
/// - You can't use this iterator for iterating trough all cached values.
pub struct IterMut<'a, T> {
    items: &'a mut HashMap<u16, T>,
    expected_index: &'a mut u16,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = T;

    /// Returns `Some` when there is an item in our cache matching the `expected_index`.
    /// Returns `None` if there are no times matching our `expected` index.
    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        match self.items.remove(&self.expected_index) {
            None => None,
            Some(e) => {
                *self.expected_index = self.expected_index.wrapping_add(1);
                Some(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{is_u16_within_half_window_from_start, Arranging, ArrangingSystem, OrderingSystem};

    #[derive(Debug, PartialEq, Clone)]
    struct Packet {
        pub sequence: u16,
        pub ordering_stream: u8,
    }

    impl Packet {
        fn new(sequence: u16, ordering_stream: u8) -> Packet {
            Packet {
                sequence,
                ordering_stream,
            }
        }
    }

    #[test]
    fn create_stream() {
        let mut system: OrderingSystem<Packet> = OrderingSystem::new();
        let stream = system.get_or_create_stream(1);

        assert_eq!(stream.expected_index(), 0);
        assert_eq!(stream.stream_id(), 1);
    }

    #[test]
    fn create_existing_stream() {
        let mut system: OrderingSystem<Packet> = OrderingSystem::new();

        system.get_or_create_stream(1);
        let stream = system.get_or_create_stream(1);

        assert_eq!(stream.stream_id(), 1);
    }

    #[test]
    fn packet_wraps_around_offset() {
        let mut system: OrderingSystem<()> = OrderingSystem::new();

        let stream = system.get_or_create_stream(1);
        for idx in 0..=65500 {
            assert![stream.arrange(idx, ()).is_some()];
        }
        assert![stream.arrange(123, ()).is_none()];
        for idx in 65501..=65535u16 {
            assert![stream.arrange(idx, ()).is_some()];
        }
        assert![stream.arrange(0, ()).is_some()];
        for idx in 1..123 {
            assert![stream.arrange(idx, ()).is_some()];
        }
        assert![stream.iter_mut().next().is_some()];
    }

    #[test]
    fn exactly_half_u16_packet_is_stored() {
        let mut system: OrderingSystem<u16> = OrderingSystem::new();

        let stream = system.get_or_create_stream(1);
        for idx in 0..=32766 {
            assert![stream.arrange(idx, idx).is_some()];
        }
        assert![stream.arrange(32768, 32768).is_none()];
        assert![stream.arrange(32767, 32767).is_some()];
        assert_eq![Some(32768), stream.iter_mut().next()];
        assert_eq![None, stream.iter_mut().next()];
    }

    #[test]
    fn u16_forward_half() {
        assert![!is_u16_within_half_window_from_start(0, 65535)];
        assert![!is_u16_within_half_window_from_start(0, 32769)];

        assert![is_u16_within_half_window_from_start(0, 32768)];
        assert![is_u16_within_half_window_from_start(0, 32767)];

        assert![is_u16_within_half_window_from_start(32767, 65535)];
        assert![!is_u16_within_half_window_from_start(32766, 65535)];
        assert![is_u16_within_half_window_from_start(32768, 65535)];
        assert![is_u16_within_half_window_from_start(32769, 0)];
    }

    #[test]
    fn can_iterate() {
        let mut system: OrderingSystem<Packet> = OrderingSystem::new();

        system.get_or_create_stream(1);
        let stream = system.get_or_create_stream(1);

        let stub_packet0 = Packet::new(0, 1);
        let stub_packet1 = Packet::new(1, 1);
        let stub_packet2 = Packet::new(2, 1);
        let stub_packet3 = Packet::new(3, 1);
        let stub_packet4 = Packet::new(4, 1);

        {
            assert_eq!(
                stream.arrange(0, stub_packet0.clone()).unwrap(),
                stub_packet0
            );

            assert![stream.arrange(3, stub_packet3.clone()).is_none()];
            assert![stream.arrange(4, stub_packet4.clone()).is_none()];
            assert![stream.arrange(2, stub_packet2.clone()).is_none()];
        }
        {
            let mut iterator = stream.iter_mut();

            // since we are awaiting for packet '2' our iterator should not return yet.
            assert_eq!(iterator.next(), None);
        }
        {
            assert_eq!(
                stream.arrange(1, stub_packet1.clone()).unwrap(),
                stub_packet1
            );
        }
        {
            // since we processed packet 2 by now we should be able to iterate and get back: 3,4,5;
            let mut iterator = stream.iter_mut();

            assert_eq!(iterator.next().unwrap(), stub_packet2);
            assert_eq!(iterator.next().unwrap(), stub_packet3);
            assert_eq!(iterator.next().unwrap(), stub_packet4);
        }
    }

    /// Asserts that the given collection, on the left, should result - after it is ordered - into the given collection, on the right.
    macro_rules! assert_order {
        ( [$( $x:expr ),*] , [$( $y:expr),*] , $stream_id:expr) => {
        {
            // initialize vector of given range on the left.
            let before = [$($x,)*];

            // initialize vector of given range on the right.
            let after = [$($y,)*];

            // create system to handle the ordering of our packets.
            let mut ordering_system = OrderingSystem::<Packet>::new();

            // get stream '1' to order the packets on.
            let stream = ordering_system.get_or_create_stream(1);
            let ordered_packets : Vec<_> = std::array::IntoIter::new(before)
                .filter_map(|seq| stream.arrange(seq, Packet::new(seq, $stream_id))
                    .map(|p| Some(p).into_iter() // if we get some packets, append packets from stream as well
                        .chain(stream.iter_mut())
                        .map(|p| p.sequence)
                        .collect::<Vec<_>>()))
                    .flatten()
                    .collect();

             // assert if the expected range of the given numbers equals to the processed range which is in sequence.
             assert_eq!(after.to_vec(), ordered_packets);
            }
        };
    }

    #[test]
    fn expect_right_order() {
        // we order on stream 1
        assert_order!([0, 2, 4, 3, 1], [0, 1, 2, 3, 4], 1);
        assert_order!([0, 4, 3, 2, 1], [0, 1, 2, 3, 4], 1);
        assert_order!([4, 2, 3, 1, 0], [0, 1, 2, 3, 4], 1);
        assert_order!([3, 2, 1, 0, 4], [0, 1, 2, 3, 4], 1);
        assert_order!([1, 0, 3, 2, 4], [0, 1, 2, 3, 4], 1);
        assert_order!([4, 1, 0, 3, 2], [0, 1, 2, 3, 4], 1);
        assert_order!([2, 1, 3, 0, 4], [0, 1, 2, 3, 4], 1);
        assert_order!([1, 0, 3, 2, 4], [0, 1, 2, 3, 4], 1);
    }

    #[test]
    fn order_on_multiple_streams() {
        // we order on streams [1...8]
        assert_order!([0, 2, 4, 3, 1], [0, 1, 2, 3, 4], 1);
        assert_order!([0, 4, 3, 2, 1], [0, 1, 2, 3, 4], 2);
        assert_order!([4, 2, 3, 1, 0], [0, 1, 2, 3, 4], 3);
        assert_order!([3, 2, 1, 0, 4], [0, 1, 2, 3, 4], 4);
        assert_order!([1, 0, 3, 2, 4], [0, 1, 2, 3, 4], 5);
        assert_order!([4, 1, 0, 3, 2], [0, 1, 2, 3, 4], 6);
        assert_order!([2, 1, 3, 0, 4], [0, 1, 2, 3, 4], 7);
        assert_order!([1, 0, 3, 2, 4], [0, 1, 2, 3, 4], 8);
    }
}
