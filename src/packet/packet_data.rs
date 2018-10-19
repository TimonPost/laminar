use super::header::HeaderParser;
use super::RawPacketData;

use error::NetworkResult;

/// Contains the raw data this packet exists of. Note that a packet can be divided into separate fragments
#[derive(Debug)]
pub struct PacketData {
    parts: Vec<RawPacketData>,
}

impl PacketData {
    pub fn new() -> PacketData {
        PacketData { parts: Vec::new() }
    }

    pub fn with_capacity(size: usize) -> PacketData {
        PacketData {
            parts: Vec::with_capacity(size),
        }
    }

    /// Add fragment to this packet
    pub fn add_fragment(
        &mut self,
        fragment: &HeaderParser<Output = NetworkResult<Vec<u8>>>,
        fragment_data: Vec<u8>,
    ) {
        self.parts.push(RawPacketData::new(fragment, fragment_data))
    }

    /// Return the total fragments this packet is divided into.
    pub fn fragment_count(&self) -> usize {
        self.parts.len()
    }

    /// Return the parts this packet exists of.
    pub fn parts(&mut self) -> Vec<Vec<u8>> {
        let mut parts_data: Vec<Vec<u8>> = Vec::new();

        for part in self.parts.iter_mut() {
            parts_data.push(part.serialize());
        }

        parts_data
    }
}

#[cfg(test)]
mod tests {
    use super::PacketData;
    use packet::header::PacketHeader;
    use infrastructure::DeliveryMethod;

    #[test]
    fn add_ang_get_parts() {
        let header = PacketHeader::new(1, 1, 1, DeliveryMethod::Unreliable);

        let mut packet_data = PacketData::new();
        packet_data.add_fragment(&header, vec![1, 2, 3, 4, 5]);
        packet_data.add_fragment(&header, vec![1, 2, 3, 4, 5]);
        packet_data.add_fragment(&header, vec![1, 2, 3, 4, 5]);

        assert_eq!(packet_data.fragment_count(), 3);

        let _ = packet_data.parts().into_iter().map(|x| {
            assert_eq!(x, vec![1, 2, 3, 4, 5]);
        });
    }
}
