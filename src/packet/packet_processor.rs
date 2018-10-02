use std::io::{self, Cursor, ErrorKind, Error, Write, Read};
use std::net::{SocketAddr, IpAddr};

use net::{SocketState, NetworkConfig};
use super::{header, Packet, FragmentBuffer, ReassemblyData};
use self::header::{FragmentHeader, PacketHeader, HeaderParser, HeaderReader};

use error::{NetworkError, Result};

/// An wrapper for processing data.
pub struct PacketProcessor
{
    /// buffer for temporarily fragment storage
    reassembly_buffer: FragmentBuffer<ReassemblyData>,
    config: NetworkConfig,
}

impl PacketProcessor {
    pub fn new(config: NetworkConfig) -> Self
    {
        PacketProcessor { reassembly_buffer: FragmentBuffer::with_capacity(config.fragment_reassembly_buffer_size), config }
    }

    /// Process data and return the resulting packet
    pub fn process_data(&mut self, packet: Vec<u8>, addr: SocketAddr, socket_state: &mut SocketState) -> Result<Option<Packet>>
    {
        let prefix_byte = packet[0];
        let mut cursor = Cursor::new(packet);

        let mut received_bytes = Ok(None);

        if prefix_byte & 1 == 0 {
            received_bytes = self.handle_normal_packet(&mut cursor, &addr, socket_state);
        } else {
            received_bytes = self.handle_fragment(&mut cursor);
        }

        return match received_bytes {
            Ok(Some(payload)) => return Ok(Some(Packet::new(addr, payload))),
            Ok(None) => return Ok(None),
            Err(e) => Err (NetworkError::ReceiveFailed.into())
        }
    }

    /// Extract fragments from data.
    fn handle_fragment(&mut self, cursor: &mut Cursor<Vec<u8>>) -> Result<Option<Vec<u8>>>
    {
        // read fragment packet
        let fragment_header = FragmentHeader::read(cursor)?;

        self.create_fragment_if_not_exists(&fragment_header);

        let mut num_fragments_received = 0;
        let mut num_fragments_total = 0;
        let mut sequence = 0;
        let mut total_buffer = Vec::new();

        {
            // get entry of previous received fragments
            let reassembly_data = match self.reassembly_buffer.get_mut(fragment_header.sequence) {
                Some(val) => val,
                None => return Err(NetworkError::FragmentInvalid.into())
            };

            // Got the data
            if reassembly_data.num_fragments_total != fragment_header.num_fragments {
                return Err(NetworkError::FragmentInvalid.into());
            }

            if reassembly_data.fragments_received[usize::from(fragment_header.id)] {
                return Err(NetworkError::FragmentInvalid.into());
            }

            // increase number of received fragments and set the specific fragment to received.
            reassembly_data.num_fragments_received += 1;
            reassembly_data.fragments_received[usize::from(fragment_header.id)] = true;

            // read payload after fragment header
            let mut payload = Vec::new();
            cursor.read_to_end(&mut payload)?;

            // add the payload from the fragment to the buffer whe have in cache
            reassembly_data.buffer.write(payload.as_slice());

            num_fragments_received = reassembly_data.num_fragments_received;
            num_fragments_total = reassembly_data.num_fragments_total;
            sequence = reassembly_data.sequence as u16;
            total_buffer = reassembly_data.buffer.clone();
        }

        // if whe received all fragments then remove entry and return the total received bytes.
        if num_fragments_received == num_fragments_total {
            let sequence = sequence as u16;
            self.reassembly_buffer.remove(sequence);

            return Ok(Some(total_buffer))
        }

        return Ok(None);
    }

    /// Extract normal header and dat from data.
    fn handle_normal_packet(&mut self, cursor: &mut Cursor<Vec<u8>>, addr: &SocketAddr, socket_state: &mut SocketState) -> Result<Option<Vec<u8>>>
    {
        let packet_header = PacketHeader::read(cursor);

        match packet_header {
            Ok(header) => {
                socket_state.process_received(*addr, &header)?;

                let mut payload = Vec::new();
                cursor.read_to_end(&mut payload)?;

                Ok(Some(payload))
            }
            Err(e) => {  return Err(NetworkError::HeaderParsingFailed.into());}
        }
    }

    /// if fragment does not exists we need to insert a new entry
    fn create_fragment_if_not_exists(&mut self, fragment_header: &FragmentHeader) -> Result<()>
    {
        if !self.reassembly_buffer.exists(fragment_header.sequence) {
            if fragment_header.id == 0 {
                if fragment_header.packet_header.is_none() {
                    return Err(NetworkError::AddConnectionToManagerFailed.into());
                }

                let packet_header = fragment_header.packet_header.unwrap();
                let ack = packet_header.ack_seq;
                let ack_bits = packet_header.ack_field;

                let reassembly_data = ReassemblyData::new(fragment_header.sequence, ack, ack_bits, fragment_header.num_fragments, fragment_header.size() as usize, (9 + self.config.fragment_size) as usize);

                self.reassembly_buffer.insert(reassembly_data.clone(), fragment_header.sequence);
            } else {
                return Err(NetworkError::AddConnectionToManagerFailed.into());
            }
        }

        Ok(())
    }
}