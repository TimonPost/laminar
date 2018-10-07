use super::header::HeaderParser;
use super::RawPacketData;

use std::io::{Result};

/// Contains the raw data this packet exists of. Note that an packet can be divided into seperate fragments
#[derive(Debug)]
pub struct PacketData
{
    parts: Vec<RawPacketData>
}

impl PacketData {
    pub fn new() -> PacketData
    {
        PacketData { parts: Vec::new() }
    }

    pub fn with_capacity(size: usize) -> PacketData
    {
        PacketData { parts: Vec::with_capacity(size) }
    }

    /// Add fragment to this packet
    pub fn add_fragment(&mut self, fragment: &HeaderParser<Output=Result<Vec<u8>>>, fragment_data: Vec<u8>)
    {
        self.parts.push(RawPacketData::new(fragment, fragment_data))
    }

    /// Return the total fragments this packet is divided into.
    pub fn fragment_count(&self) -> usize
    {
        return self.parts.len()
    }

    /// Return the parts this packet exists of.
    pub fn parts(&mut self) -> Vec<Vec<u8>>
    {
        let mut parts_data: Vec<Vec<u8>> = Vec::new();

        for part in self.parts.iter_mut() {
            parts_data.push(part.serialize());
        }

        parts_data
    }
}

mod tests {
    use super::PacketData;
    use packet::header::PacketHeader;

    #[test]
    fn add_ang_get_parts()
    {
        let header = PacketHeader::new(1,1,1);

        let mut packet_data = PacketData::new();
        packet_data.add_fragment(&header, vec![1,2,3,4,5]);
        packet_data.add_fragment(&header, vec![1,2,3,4,5]);
        packet_data.add_fragment(&header, vec![1,2,3,4,5]);

        assert_eq!(packet_data.fragment_count(), 3);

        packet_data.parts().into_iter().map(|x| { assert_eq!(x, vec![1, 2, 3, 4, 5]); });
    }
}

