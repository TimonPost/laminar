use crate::config::NetworkConfig;
use crate::error::{FragmentErrorKind, NetworkResult};
use crate::packet::header::{AckedPacketHeader, FragmentHeader, HeaderWriter, HeaderReader};
use crate::packet::PacketData;
use crate::sequence_buffer::{ReassemblyData, SequenceBuffer};

use std::io::{Cursor, Read, Write};
use std::sync::Arc;

/// Type that will manage fragmentation of packets.
pub struct Fragmentation {
    fragments: SequenceBuffer<ReassemblyData>,
    config: Arc<NetworkConfig>,
}

impl Fragmentation {
    /// Creates and returns a new Fragmentation
    pub fn new(config: Arc<NetworkConfig>) -> Fragmentation {
        Fragmentation {
            fragments: SequenceBuffer::with_capacity(config.fragment_reassembly_buffer_size),
            config,
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
    /// So an example of dividing an packet of bytes we get these fragments:
    ///
    /// So for 4000 bytes we need 4 fragments
    /// [fragment: 1024] [fragment: 1024] [fragment: 1024] [fragment: 928]
    pub fn total_fragments_needed(payload_length: u16, fragment_size: u16) -> u16 {
        let remainder = if payload_length % fragment_size > 0 {
            1
        } else {
            0
        };
        ((payload_length / fragment_size) + remainder)
    }

    /// Split the given payload into fragments and write those fragments to the passed packet data.
    pub fn spit_into_fragments(
        payload: &[u8],
        acked_header: AckedPacketHeader,
        packet_data: &mut PacketData,
        config: &Arc<NetworkConfig>,
    ) -> NetworkResult<()> {
        let payload_length = payload.len() as u16;
        let num_fragments =
            Fragmentation::total_fragments_needed(payload_length, config.fragment_size) as u8; /* safe cast max fragments is u8 */

        if num_fragments > config.max_fragments {
            Err(FragmentErrorKind::ExceededMaxFragments)?;
        }

        for fragment_id in 0..num_fragments {
            let fragment = FragmentHeader::new(
                acked_header.standard_header,
                fragment_id,
                num_fragments,
                acked_header,
            );
            let mut buffer = Vec::with_capacity(fragment.size() as usize);
            fragment.parse(&mut buffer)?;

            // get start end pos in buffer
            let start_fragment_pos = u16::from(fragment_id) * config.fragment_size;
            let mut end_fragment_pos = (u16::from(fragment_id) + 1) * config.fragment_size;

            // If remaining buffer fits int one packet just set the end position to the length of the packet payload.
            if end_fragment_pos > payload_length {
                end_fragment_pos = payload_length;
            }

            // get specific slice of data for fragment
            let fragment_data = &payload[start_fragment_pos as usize..end_fragment_pos as usize];

            packet_data.add_fragment(&buffer, fragment_data)?;
        }

        Ok(())
    }

    /// This will read fragment data and returns the complete packet data when all fragments are received.
    pub fn handle_fragment(
        &mut self,
        cursor: &mut Cursor<&[u8]>,
    ) -> NetworkResult<Option<Vec<u8>>> {
        // read fragment packet
        let fragment_header = FragmentHeader::read(cursor)?;

        self.create_fragment_if_not_exists(&fragment_header)?;

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
                Err(FragmentErrorKind::FragmentWithUnevenNumberOfFragemts)?
            }

            if reassembly_data.fragments_received[usize::from(fragment_header.id())] {
                Err(FragmentErrorKind::AlreadyProcessedFragment)?
            }

            // increase number of received fragments and set the specific fragment to received.
            reassembly_data.num_fragments_received += 1;
            reassembly_data.fragments_received[usize::from(fragment_header.id())] = true;

            // read payload after fragment header
            let mut payload = Vec::new();
            cursor.read_to_end(&mut payload)?;

            // add the payload from the fragment to the buffer whe have in cache
            reassembly_data.buffer.write_all(payload.as_slice())?;

            num_fragments_received = reassembly_data.num_fragments_received;
            num_fragments_total = reassembly_data.num_fragments_total;
            sequence = reassembly_data.sequence as u16;
            total_buffer = reassembly_data.buffer.clone();
        }

        // if whe received all fragments then remove entry and return the total received bytes.
        if num_fragments_received == num_fragments_total {
            let sequence = sequence as u16;
            self.fragments.remove(sequence);

            return Ok(Some(total_buffer));
        }

        Ok(None)
    }

    /// If fragment does not exist we need to insert a new entry.
    fn create_fragment_if_not_exists(
        &mut self,
        fragment_header: &FragmentHeader,
    ) -> NetworkResult<()> {
        if !self.fragments.exists(fragment_header.sequence()) {
            if fragment_header.id() == 0 {
                match fragment_header.packet_header() {
                    Some(_header) => {
                        let reassembly_data = ReassemblyData::new(
                            fragment_header.sequence(),
                            fragment_header.fragment_count(),
                            (9 + self.config.fragment_size) as usize,
                        );

                        self.fragments
                            .insert(reassembly_data.clone(), fragment_header.sequence());
                    }
                    None => Err(FragmentErrorKind::PacketHeaderNotFound)?,
                }
            } else {
                Err(FragmentErrorKind::AlreadyProcessedFragment)?
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::Fragmentation;

    #[test]
    pub fn total_fragments_needed_test() {
        let fragment_number = Fragmentation::total_fragments_needed(4000, 1024);
        let fragment_number1 = Fragmentation::total_fragments_needed(500, 1024);

        assert_eq!(fragment_number, 4);
        assert_eq!(fragment_number1, 1);
    }
}
