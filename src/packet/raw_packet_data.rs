use super::header::HeaderParser;
use std::io::Result;

#[derive(Clone, Debug)]
/// This is the raw packet data that.
pub struct RawPacketData {
    // these are the header bytes.
    header: Vec<u8>,
    // these are the payload bytes
    body: Vec<u8>,
}

impl RawPacketData {
    pub fn new(header: &HeaderParser<Output = Result<Vec<u8>>>, body: Vec<u8>) -> RawPacketData {
        let header = header.parse().unwrap();
        RawPacketData { header, body }
    }

    /// Serialize the packet header and body into one byte buffer
    pub fn serialize(&mut self) -> Vec<u8> {
        let mut vec = Vec::with_capacity(self.header.len() + self.body.len());
        vec.append(&mut self.header);
        vec.append(&mut self.body);
        vec
    }
}

#[cfg(test)]
mod tests {
    use super::RawPacketData;
    use net::constants::PACKET_HEADER_SIZE;
    use packet::header::PacketHeader;
    use infrastructure::DeliveryMethod;

    #[test]
    fn serialize_raw_data_test() {
        let header = PacketHeader::new(1, 1, 1, DeliveryMethod::SequencedUnordered);

        let data = vec![1, 2, 3, 4, 5];
        let mut raw_packet_data = RawPacketData::new(&header, data.clone());
        assert_eq!(
            raw_packet_data.serialize().len(),
            ((PACKET_HEADER_SIZE as usize) + data.len())
        );
    }
}
