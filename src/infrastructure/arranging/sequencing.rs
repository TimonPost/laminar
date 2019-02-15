//! Module with logic for arranging items in-sequence on multiple streams.
//!
//! "_Sequencing is the process of only caring about the newest items._"
//!
//! With sequencing, we only care about the newest items. When old items arrive we just toss them away.
//!
//! Example: sequence `1,3,2,5,4` will result into `1,3,5`.
//!
//! # Remarks
//! - See [super-module](../index.html) description for more details.

use super::{Arranging, ArrangingSystem};
use std::{
    collections::HashMap,
    marker::PhantomData
};

/// An sequencing system that can arrange items in sequence on different streams.
///
/// Checkout [`SequencingStream`](./struct.SequencingStream.html), or module description for more details.
///
/// # Remarks
/// - See [super-module](../index.html) for more information about streams.
pub struct SequencingSystem<T> {
    // '[HashMap]' with streams on which items can be arranged in-sequence.
    streams: HashMap<u8, SequencingStream<T>>,
}

impl<T> SequencingSystem<T> {
    /// Constructs a new [`SequencingSystem`](./struct.SequencingSystem.html).
    pub fn new() -> SequencingSystem<T> {
        SequencingSystem {
            streams: HashMap::with_capacity(32),
        }
    }
}

impl<T> ArrangingSystem for SequencingSystem<T> {
    type Stream = SequencingStream<T>;

    /// Returns the number of sequencing streams currently created.
    fn stream_count(&self) -> usize {
        self.streams.len()
    }

    /// Try to get an [`SequencingStream`](./struct.SequencingStream.html) by `stream_id`.
    /// When the stream does not exist, it will be inserted by the given `stream_id` and returned.
    fn get_or_create_stream(&mut self, stream_id: u8) -> &mut Self::Stream {
        self.streams
            .entry(stream_id)
            .or_insert_with(|| SequencingStream::new(stream_id))
    }
}

/// A stream on which items will be arranged in-sequence.
///
/// # Algorithm
///
/// With every sequencing operation an `top_index` is given.
///
/// There are two scenarios that are important to us.
/// 1. `incoming_index` >= `top_index`.
/// This item is the newest or newer than the last one we have seen.
/// Because of that we should return it back to the user.
/// 2. `incoming_index` < `top_index`.
/// This item is older than the newest item we have seen so far.
/// Since we don't care about old items we can toss it a way.
///
/// # Remarks
/// - See [super-module](../index.html) for more information about streams.
pub struct SequencingStream<T> {
    // the id of this stream.
    stream_id: u8,
    // the highest seen item index.
    top_index: usize,
    // I need `PhantomData`, otherwise, I can't use a generic in the `Arranging` implementation because `T` is not constrained.
    phantom: PhantomData<T>
}

impl<T> SequencingStream<T> {
    /// Constructs a new, empty '[SequencingStream](./struct.SequencingStream.html)'.
    ///
    /// The default stream will have a capacity of 32 items.
    pub fn new(stream_id: u8) -> SequencingStream<T> {
        SequencingStream {
            stream_id,
            top_index: 0,
            phantom: PhantomData
        }
    }

    /// Returns the identifier of this stream.
    fn stream_id(&self) -> u8 {
        self.stream_id
    }
}

impl<T> Arranging for SequencingStream<T> {
    type ArrangingItem = T;

    /// Will arrange the given item based on a sequencing algorithm.
    ///
    /// With every sequencing operation an `top_index` is given.
    ///
    /// # Algorithm
    ///
    /// There are two scenarios that are important to us.
    /// 1. `incoming_index` >= `top_index`.
    /// This item is the newest or newer than the last one we have seen.
    /// Because of that we should return it back to the user.
    /// 2. `incoming_index` < `top_index`.
    /// This item is older than we the newest packet we have seen so far.
    /// Since we don't care about old items we can toss it a way.
    ///
    /// # Remark
    /// - All old packets will be tossed away.
    /// - None is returned when an old packet is received.
    fn arrange(
        &mut self,
        incoming_index: usize,
        item: Self::ArrangingItem,
    ) -> Option<Self::ArrangingItem> {
        if incoming_index >= self.top_index {
            self.top_index = incoming_index;
            return Some(item);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{Arranging, ArrangingSystem, SequencingSystem};

    #[derive(Debug, PartialEq, Clone)]
    struct Packet {
        pub sequence: usize,
        pub ordering_stream: u8,
    }

    impl Packet {
        fn new(sequence: usize, ordering_stream: u8) -> Packet {
            Packet {
                sequence,
                ordering_stream,
            }
        }
    }

    #[test]
    fn create_stream() {
        let mut system: SequencingSystem<Packet> = SequencingSystem::new();
        let stream = system.get_or_create_stream(1);

        assert_eq!(stream.stream_id(), 1);
    }

    #[test]
    fn create_existing_stream() {
        let mut system: SequencingSystem<Packet> = SequencingSystem::new();

        system.get_or_create_stream(1);
        let stream = system.get_or_create_stream(1);

        assert_eq!(stream.stream_id(), 1);
    }

    /// asserts that the given collection, on the left, should result - after it is sequenced - into the given collection, on the right.
    macro_rules! assert_sequence {
        ( [$( $x:expr ),*], [$( $y:expr),*], $stream_id:expr) => {
            {
                // initialize vector of given range on the left.
                let mut before: Vec<usize> = Vec::new();
                $(
                    before.push($x);
                )*

                // initialize vector of given range on the right.
                let mut after: Vec<usize> = Vec::new();
                $(
                    after.push($y);
                )*

                // generate test packets
                let mut packets = Vec::new();

                for (_, v) in before.iter().enumerate() {
                    packets.push(Packet::new(*v, $stream_id));
                }

                // create system to handle sequenced packets.
                let mut sequence_system = SequencingSystem::<Packet>::new();

                // get stream '1' to process the sequenced packets on.
                let stream = sequence_system.get_or_create_stream(1);

                // get packets arranged in sequence.
                let mut sequenced_packets = Vec::new();

                for packet in packets.into_iter() {
                    match stream.arrange(packet.sequence, packet.clone()) {
                        Some(packet) => { sequenced_packets.push(packet.sequence);},
                        None => {}
                    };
                }

               // assert if the expected range of the given numbers equals to the processed range which is in sequence.
               assert_eq!(after, sequenced_packets);
            }
        };
    }

    // This will assert a bunch of ranges to a correct sequenced range.
    #[test]
    fn sequencing_test() {
        assert_sequence!([1, 3, 5, 4, 2], [1, 3, 5], 1);
        assert_sequence!([1, 5, 4, 3, 2], [1, 5], 1);
        assert_sequence!([5, 3, 4, 2, 1], [5], 1);
        assert_sequence!([4, 3, 2, 1, 5], [4, 5], 1);
        assert_sequence!([2, 1, 4, 3, 5], [2, 4, 5], 1);
        assert_sequence!([5, 2, 1, 4, 3], [5], 1);
        assert_sequence!([3, 2, 4, 1, 5], [3, 4, 5], 1);
    }

    // This will assert a bunch of ranges to a correct sequenced range.
    #[test]
    fn multiple_stream_sequencing_test() {
        assert_sequence!([1, 3, 5, 4, 2], [1, 3, 5], 1);
        assert_sequence!([1, 5, 4, 3, 2], [1, 5], 2);
        assert_sequence!([5, 3, 4, 2, 1], [5], 3);
        assert_sequence!([4, 3, 2, 1, 5], [4, 5], 4);
        assert_sequence!([2, 1, 4, 3, 5], [2, 4, 5], 5);
        assert_sequence!([5, 2, 1, 4, 3], [5], 6);
        assert_sequence!([3, 2, 4, 1, 5], [3, 4, 5], 7);
    }
}
