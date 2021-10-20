use std::clone::Clone;

use crate::packet::SequenceNumber;

pub use self::reassembly_data::ReassemblyData;

mod reassembly_data;

/// Collection to store data of any kind.
#[derive(Debug)]
pub struct SequenceBuffer<T: Clone + Default> {
    sequence_num: SequenceNumber,
    entry_sequences: Box<[Option<SequenceNumber>]>,
    entries: Box<[T]>,
}

impl<T: Clone + Default> SequenceBuffer<T> {
    /// Creates a SequenceBuffer with a desired capacity.
    pub fn with_capacity(size: u16) -> Self {
        Self {
            sequence_num: 0,
            entry_sequences: vec![None; size as usize].into_boxed_slice(),
            entries: vec![T::default(); size as usize].into_boxed_slice(),
        }
    }

    /// Returns the most recently stored sequence number.
    pub fn sequence_num(&self) -> SequenceNumber {
        self.sequence_num
    }

    /// Returns a mutable reference to the entry with the given sequence number.
    pub fn get_mut(&mut self, sequence_num: SequenceNumber) -> Option<&mut T> {
        if self.exists(sequence_num) {
            let index = self.index(sequence_num);
            return Some(&mut self.entries[index]);
        }
        None
    }

    /// Inserts the entry data into the sequence buffer. If the requested sequence number is "too
    /// old", the entry will not be inserted and no reference will be returned.
    pub fn insert(&mut self, sequence_num: SequenceNumber, entry: T) -> Option<&mut T> {
        // sequence number is too old to insert into the buffer
        if sequence_less_than(
            sequence_num,
            self.sequence_num
                .wrapping_sub(self.entry_sequences.len() as u16),
        ) {
            return None;
        }

        self.advance_sequence(sequence_num);

        let index = self.index(sequence_num);
        self.entry_sequences[index] = Some(sequence_num);
        self.entries[index] = entry;
        Some(&mut self.entries[index])
    }

    /// Returns whether or not we have previously inserted an entry for the given sequence number.
    pub fn exists(&self, sequence_num: SequenceNumber) -> bool {
        let index = self.index(sequence_num);
        if let Some(s) = self.entry_sequences[index] {
            return s == sequence_num;
        }
        false
    }

    /// Removes an entry from the sequence buffer
    pub fn remove(&mut self, sequence_num: SequenceNumber) -> Option<T> {
        if self.exists(sequence_num) {
            let index = self.index(sequence_num);
            let value = std::mem::take(&mut self.entries[index]);
            self.entry_sequences[index] = None;
            return Some(value);
        }
        None
    }

    // Advances the sequence number while removing older entries.
    fn advance_sequence(&mut self, sequence_num: SequenceNumber) {
        if sequence_greater_than(sequence_num.wrapping_add(1), self.sequence_num) {
            self.remove_entries(u32::from(sequence_num));
            self.sequence_num = sequence_num.wrapping_add(1);
        }
    }

    fn remove_entries(&mut self, mut finish_sequence: u32) {
        let start_sequence = u32::from(self.sequence_num);
        if finish_sequence < start_sequence {
            finish_sequence += 65536;
        }

        if finish_sequence - start_sequence < self.entry_sequences.len() as u32 {
            for sequence in start_sequence..=finish_sequence {
                self.remove(sequence as u16);
            }
        } else {
            for index in 0..self.entry_sequences.len() {
                self.entries[index] = T::default();
                self.entry_sequences[index] = None;
            }
        }
    }

    // Generates an index for use in `entry_sequences` and `entries`.
    fn index(&self, sequence: SequenceNumber) -> usize {
        sequence as usize % self.entry_sequences.len()
    }
}

pub fn sequence_greater_than(s1: u16, s2: u16) -> bool {
    ((s1 > s2) && (s1 - s2 <= 32768)) || ((s1 < s2) && (s2 - s1 > 32768))
}

pub fn sequence_less_than(s1: u16, s2: u16) -> bool {
    sequence_greater_than(s2, s1)
}

#[cfg(test)]
mod tests {
    use crate::sequence_buffer::sequence_greater_than;
    use crate::sequence_buffer::sequence_less_than;

    use super::SequenceBuffer;

    #[derive(Clone, Default)]
    struct DataStub;

    #[test]
    fn test_sequence_comparisons_than() {
        assert!(sequence_greater_than(1, 0));
        assert!(sequence_less_than(0, 1));

        // right around the halfway point is where we cut over.
        assert!(sequence_greater_than(32768, 0));
        assert!(sequence_less_than(32769, 0));

        // in this case, 0 is greater than u16 max because we're likely at the wrapping case
        assert!(sequence_greater_than(0, u16::max_value()));
    }

    #[test]
    fn max_sequence_number_should_not_exist_by_default() {
        let buffer: SequenceBuffer<DataStub> = SequenceBuffer::with_capacity(2);
        assert!(!buffer.exists(u16::max_value()));
    }

    #[test]
    fn ensure_entries_and_entry_sequences_are_the_same_size() {
        let buffer: SequenceBuffer<DataStub> = SequenceBuffer::with_capacity(2);
        assert_eq!(buffer.entry_sequences.len(), buffer.entries.len());
    }

    #[test]
    fn normal_inserts_should_fill_buffer() {
        let mut buffer = SequenceBuffer::with_capacity(8);
        for i in 0..8 {
            buffer.insert(i, DataStub);
        }
        assert_eq!(count_entries(&buffer), 8);
    }

    #[test]
    fn insert_into_buffer_test() {
        let mut buffer = SequenceBuffer::with_capacity(2);
        buffer.insert(0, DataStub);
        assert!(buffer.exists(0));
    }

    #[test]
    fn remove_from_buffer_test() {
        let mut buffer = SequenceBuffer::with_capacity(2);
        buffer.insert(0, DataStub);
        buffer.remove(0);
        assert!(!buffer.exists(0));
    }

    #[test]
    fn insert_into_buffer_old_entry_test() {
        let mut buffer = SequenceBuffer::with_capacity(8);
        buffer.insert(8, DataStub);
        // this entry would overlap with sequence 8 based on the buffer size so we must ensure that
        // it does not.
        buffer.insert(0, DataStub);
        assert!(!buffer.exists(0));

        // however, this one is more recent so it should definitely exist.
        buffer.insert(16, DataStub);
        assert!(buffer.exists(16));

        // since we are pretty far ahead at this point, there should only be 1 valid entry in here.
        assert_eq!(count_entries(&buffer), 1);
    }

    #[test]
    fn new_sequence_nums_evict_old_ones() {
        let mut buffer = SequenceBuffer::with_capacity(2);
        for i in 0..3 {
            buffer.insert(i, DataStub);
            assert_eq!(buffer.sequence_num(), i + 1);
        }
        assert!(!buffer.exists(0));
        assert!(buffer.exists(1));
        assert!(buffer.exists(2));
        assert_eq!(count_entries(&buffer), 2);
    }

    #[test]
    fn older_sequence_numbers_arent_inserted() {
        let mut buffer = SequenceBuffer::with_capacity(8);
        buffer.insert(10, DataStub);

        assert_eq!(buffer.sequence_num(), 11);

        // inserting 'older' should fail to insert
        buffer.insert(2, DataStub);
        assert!(!buffer.exists(2));

        // insert respects boundary wrap. Both of these would be earlier than 11
        buffer.insert(u16::max_value(), DataStub);
        buffer.insert(0, DataStub);
        assert!(!buffer.exists(u16::max_value()));
        assert!(!buffer.exists(0));

        assert_eq!(count_entries(&buffer), 1);
    }

    fn count_entries(buffer: &SequenceBuffer<DataStub>) -> usize {
        let nums: usize = buffer.entry_sequences.iter().flatten().count();
        nums
    }
}
