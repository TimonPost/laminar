use crate::{
    config::Config,
    error::{FragmentErrorKind, Result},
    net::constants::FRAGMENT_HEADER_SIZE,
    packet::header::{AckedPacketHeader, FragmentHeader},
    sequence_buffer::{ReassemblyData, SequenceBuffer},
};

use std::io::Write;

/// Type that will manage fragmentation of packets.
pub struct Fragmentation {
    fragments: SequenceBuffer<ReassemblyData>,
    config: Config,
}

impl Fragmentation {
    /// Creates and returns a new Fragmentation
    pub fn new(config: &Config) -> Fragmentation {
        Fragmentation {
            fragments: SequenceBuffer::with_capacity(config.fragment_reassembly_buffer_size),
            config: config.clone(),
        }
    }

    /// This functions checks how many times a number fits into another number and will round up.
    ///
    /// For example we have two numbers:
    /// - number 1 = 4000;
    /// - number 2 = 1024;
    /// If you do it the easy way the answer will be 4000/1024 = 3.90625.
    /// But since we care about how how many whole times the number fits in we need the result 4.
    ///
    /// Note that when rust is rounding it is always rounding to zero (3.456 as u32 = 3)
    /// 1. calculate with modulo if `number 1` fits exactly in the `number 2`.
    /// 2. Divide `number 1` with `number 2` (this wil be rounded to zero by rust)
    /// 3. So in all cases we need to add 1 to get the right amount of fragments.
    ///
    /// lets take an example
    ///
    /// Calculate modules:
    /// - number 1 % number 2 = 928
    /// - this is bigger than 0 so remainder = 1
    ///
    /// Calculate how many times the `number 1` fits in `number 2`:
    /// - number 1 / number 2 = 3,90625 (this will be rounded to 3)
    /// - add remainder from above to 3 = 4.
    ///
    /// The above described method will figure out for all number how many times it fits into another number rounded up.
    ///
    /// So an example of dividing a packet of bytes we get these fragments:
    ///
    /// So for 4000 bytes we need 4 fragments
    /// [fragment: 1024] [fragment: 1024] [fragment: 1024] [fragment: 928]
    pub fn fragments_needed(payload_length: u16, fragment_size: u16) -> u16 {
        let remainder = if payload_length % fragment_size > 0 {
            1
        } else {
            0
        };
        ((payload_length / fragment_size) + remainder)
    }

    /// Split the given payload into fragments and write those fragments to the passed packet data.
    pub fn spit_into_fragments<'a>(payload: &'a [u8], config: &Config) -> Result<Vec<&'a [u8]>> {
        let mut fragments = Vec::new();

        let payload_length = payload.len() as u16;
        let num_fragments =
            // Safe cast max fragments is u8
            Fragmentation::fragments_needed(payload_length, config.fragment_size) as u8;

        if num_fragments > config.max_fragments {
            Err(FragmentErrorKind::ExceededMaxFragments)?;
        }

        for fragment_id in 0..num_fragments {
            // get start and end position of buffer
            let start_fragment_pos = u16::from(fragment_id) * config.fragment_size;
            let mut end_fragment_pos = (u16::from(fragment_id) + 1) * config.fragment_size;

            // If remaining buffer fits int one packet just set the end position to the length of the packet payload.
            if end_fragment_pos > payload_length {
                end_fragment_pos = payload_length;
            }

            // get specific slice of data for fragment
            let fragment_data = &payload[start_fragment_pos as usize..end_fragment_pos as usize];

            fragments.push(fragment_data);
        }

        Ok(fragments)
    }

    /// This will read fragment data and return the complete packet when all fragments are received.
    pub fn handle_fragment(
        &mut self,
        fragment_header: FragmentHeader,
        fragment_payload: &[u8],
        acked_header: Option<AckedPacketHeader>,
    ) -> Result<Option<(Vec<u8>, AckedPacketHeader)>> {
        // read fragment packet

        self.create_fragment_if_not_exists(fragment_header);

        let num_fragments_received;
        let num_fragments_total;
        let sequence;
        let total_buffer;

        {
            // get entry of previous received fragments
            let reassembly_data = match self.fragments.get_mut(fragment_header.sequence()) {
                Some(val) => val,
                None => Err(FragmentErrorKind::CouldNotFindFragmentById)?,
            };

            // Got the data
            if reassembly_data.num_fragments_total != fragment_header.fragment_count() {
                Err(FragmentErrorKind::FragmentWithUnevenNumberOfFragments)?
            }

            if usize::from(fragment_header.id()) >= reassembly_data.fragments_received.len() {
                Err(FragmentErrorKind::ExceededMaxFragments)?;
            }

            if reassembly_data.fragments_received[usize::from(fragment_header.id())] {
                Err(FragmentErrorKind::AlreadyProcessedFragment)?
            }

            // increase number of received fragments and set the specific fragment to received.
            reassembly_data.num_fragments_received += 1;
            reassembly_data.fragments_received[usize::from(fragment_header.id())] = true;

            // add the payload from the fragment to the buffer whe have in cache
            reassembly_data.buffer.write_all(&*fragment_payload)?;

            if let Some(acked_header) = acked_header {
                if reassembly_data.acked_header.is_none() {
                    reassembly_data.acked_header = Some(acked_header);
                } else {
                    Err(FragmentErrorKind::MultipleAckHeaders)?;
                }
            }

            num_fragments_received = reassembly_data.num_fragments_received;
            num_fragments_total = reassembly_data.num_fragments_total;
            sequence = reassembly_data.sequence as u16;
            total_buffer = reassembly_data.buffer.clone();
        }

        // if we received all fragments then remove entry and return the total received bytes.
        if num_fragments_received == num_fragments_total {
            let sequence = sequence as u16;
            if let Some(mut reassembly_data) = self.fragments.remove(sequence) {
                if reassembly_data.acked_header.is_none() {
                    Err(FragmentErrorKind::MissingAckHeader)?;
                }

                let acked_header = reassembly_data.acked_header.take().unwrap();
                return Ok(Some((total_buffer, acked_header)));
            } else {
                Err(FragmentErrorKind::CouldNotFindFragmentById)?;
            }
        }

        Ok(None)
    }

    /// If fragment does not exist we need to insert a new entry.
    fn create_fragment_if_not_exists(&mut self, fragment_header: FragmentHeader) {
        if !self.fragments.exists(fragment_header.sequence()) {
            let reassembly_data = ReassemblyData::new(
                fragment_header.sequence(),
                fragment_header.fragment_count(),
                (u16::from(FRAGMENT_HEADER_SIZE) + self.config.fragment_size) as usize,
            );

            self.fragments
                .insert(fragment_header.sequence(), reassembly_data);
        }
    }
}

#[cfg(test)]
mod test {
    use super::Fragmentation;

    #[test]
    pub fn expect_right_number_of_fragments() {
        let fragment_number = Fragmentation::fragments_needed(4000, 1024);
        let fragment_number1 = Fragmentation::fragments_needed(500, 1024);

        assert_eq!(fragment_number, 4);
        assert_eq!(fragment_number1, 1);
    }
}
