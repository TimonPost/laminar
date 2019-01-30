use crate::error::NetworkResult;
use std::io::Write;

/// Contains the raw data this packet exists of. Note that a packet can be divided into separate fragments
#[derive(Debug, Default)]
pub struct PacketData {
    parts: Vec<Vec<u8>>,
}

impl PacketData {
    /// Creates a new PacketData with a specified capacity
    pub fn with_capacity(size: usize) -> PacketData {
        PacketData {
            parts: Vec::with_capacity(size),
        }
    }

    /// Add fragment to this packet
    pub fn add_fragment(&mut self, fragment: &[u8], fragment_data: &[u8]) -> NetworkResult<()> {
        let mut part = Vec::with_capacity(fragment.len() + fragment_data.len());
        part.write_all(fragment)?;
        part.write_all(fragment_data)?;
        self.parts.push(part);
        Ok(())
    }

    /// Return the total fragments this packet is divided into.
    pub fn fragment_count(&self) -> usize {
        self.parts.len()
    }

    /// Return the parts this packet exists of.
    pub fn parts(&mut self) -> &Vec<Vec<u8>> {
        &self.parts
    }
}

#[cfg(test)]
mod tests {
    use super::PacketData;
    use crate::packet::header::{AckedPacketHeader, HeaderReader, HeaderWriter, StandardHeader};

    #[test]
    fn add_ang_get_parts() {
        let acked_header = AckedPacketHeader::new(StandardHeader::default(), 1, 1, 5421);
        let mut buffer = Vec::new();
        let _ = acked_header.parse(&mut buffer);

        let mut packet_data = PacketData::with_capacity(acked_header.size() as usize);
        let _ = packet_data.add_fragment(&buffer, &vec![1, 2, 3, 4, 5]);
        let _ = packet_data.add_fragment(&buffer, &vec![1, 2, 3, 4, 5]);
        let _ = packet_data.add_fragment(&buffer, &vec![1, 2, 3, 4, 5]);

        assert_eq!(packet_data.fragment_count(), 3);

        let _ = packet_data.parts().into_iter().map(|x| {
            let _header = &x[0..acked_header.size() as usize];
            let body = &x[acked_header.size() as usize..buffer.len()];
            assert_eq!(body.to_vec(), vec![1, 2, 3, 4, 5]);
        });
    }
}
