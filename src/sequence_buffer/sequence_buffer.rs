use std::clone::Clone;

/// Collection to store data of any kind.
pub struct SequenceBuffer<T>  where T: Default + Clone + Send + Sync  {
    entries: Vec<T>,
    entry_sequences: Vec<u16>,
}

impl<T> SequenceBuffer<T> where T: Default + Clone + Send + Sync {
    /// Create collection with a specific capacity.
    pub fn with_capacity(size: usize) -> Self {
        let mut entries = Vec::with_capacity(size);
        let mut entry_sequences = Vec::with_capacity(size);

        entries.resize(size, T::default());
        entry_sequences.resize(size, 0xFFFF);

        SequenceBuffer {
            entries,
            entry_sequences,
        }
    }

    /// Get mutable entry from collection by sequence number.
    pub fn get_mut(&mut self, sequence: u16) -> Option<&mut T> {
        let index = self.index(sequence);

        if self.entry_sequences[index] != sequence {
            return None;
        }

        Some(&mut self.entries[index])
    }

    /// Insert new entry into the collection.
    pub fn insert(&mut self, data: T, sequence: u16) -> &mut T {
        let index = self.index(sequence);

        self.entries[index] = data;
        self.entry_sequences[index] = sequence;

        &mut self.entries[index]
    }

    /// Remove entry from collection.
    pub fn remove(&mut self, sequence: u16) {
        // TODO: validity check
        let index = self.index(sequence);
        self.entries[index] = T::default();
        self.entry_sequences[index] = 0xFFFF;
    }

    /// checks if an certain entry exists.
    pub fn exists(&self, sequence: u16) -> bool {
        let index = self.index(sequence);
        if self.entry_sequences[index] != sequence {
            return false;
        }

        return true;
    }

    /// Get the length of the collection.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// converts an sequence number to an index that could be used for the inner storage.
    fn index(&self, sequence: u16) -> usize {
        sequence as usize % self.entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::SequenceBuffer;

    #[derive(Clone, Default)]
    struct DataStub;

    #[test]
    fn insert_into_fragment_buffer_test()
    {
        let mut fragment_buffer = SequenceBuffer::with_capacity(2);
        fragment_buffer.insert(DataStub, 1);
        assert!(fragment_buffer.exists(1));
    }

    #[test]
    fn remove_from_fragment_buffer_test() {
        let mut fragment_buffer = SequenceBuffer::with_capacity(2);
        fragment_buffer.insert(DataStub, 1);
        fragment_buffer.remove(1);
        assert!(!fragment_buffer.exists(1));
    }

    #[test]
    fn fragment_buffer_len_test() {
        let mut fragment_buffer = SequenceBuffer::with_capacity(2);
        fragment_buffer.insert(DataStub, 1);
        fragment_buffer.insert(DataStub, 2);
        assert_eq!(fragment_buffer.len(), 2);
    }
}
