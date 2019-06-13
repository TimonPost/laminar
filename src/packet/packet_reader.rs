use crate::net::constants::STANDARD_HEADER_SIZE;
use crate::packet::header::{
    AckedPacketHeader, ArrangingHeader, FragmentHeader, HeaderReader, StandardHeader,
};
use crate::{ErrorKind, Result};

use std::io::Cursor;

/// Can be used to read the packet contents of laminar.
///
/// # Remarks
/// - `PacketReader` is using an underlying `Cursor` to manage the reading of the bytes.
/// - `PacketReader` can interpret where some data is located in the buffer, that's why you don't have to worry about the position of the `Cursor`.
pub struct PacketReader<'s> {
    buffer: &'s [u8],
    cursor: Cursor<&'s [u8]>,
}

impl<'s> PacketReader<'s> {
    /// Construct a new instance of `PacketReader`, the given `buffer` will be used to read information from.
    pub fn new(buffer: &'s [u8]) -> PacketReader<'s> {
        PacketReader {
            buffer,
            cursor: Cursor::new(buffer),
        }
    }

    /// Read the `StandardHeader` from the underlying buffer.
    ///
    /// # Remark
    /// - Will change the position to the location of `StandardHeader`
    pub fn read_standard_header(&mut self) -> Result<StandardHeader> {
        self.cursor.set_position(0);

        if self.can_read(StandardHeader::size()) {
            StandardHeader::read(&mut self.cursor)
        } else {
            Err(ErrorKind::CouldNotReadHeader(String::from("standard")))
        }
    }

    /// Read the `StandardHeader` from the underlying buffer.
    ///
    /// # Remark
    /// - Will change the position to the location of `StandardHeader`
    pub fn read_arranging_header(&mut self, start_offset: u16) -> Result<ArrangingHeader> {
        self.cursor.set_position(u64::from(start_offset));

        if self.can_read(ArrangingHeader::size()) {
            ArrangingHeader::read(&mut self.cursor)
        } else {
            Err(ErrorKind::CouldNotReadHeader(String::from("arranging")))
        }
    }

    /// Read the `AckedPacketHeader` from the underlying buffer.
    ///
    /// # Remark
    /// - Will change the position to the location of `AckedPacketHeader`
    pub fn read_acknowledge_header(&mut self) -> Result<AckedPacketHeader> {
        // acknowledge header comes after standard header.
        self.cursor.set_position(u64::from(STANDARD_HEADER_SIZE));

        if self.can_read(AckedPacketHeader::size()) {
            AckedPacketHeader::read(&mut self.cursor)
        } else {
            Err(ErrorKind::CouldNotReadHeader(String::from(
                "acknowledgment",
            )))
        }
    }

    /// Read the `FragmentHeader` and optionally the `AckedPacketHeader` from the underlying buffer.
    ///
    /// # Remark
    /// - Notice that this will continue on the position of last read header;
    /// e.g. when reading `StandardHeader` the position of the underlying `Cursor` will be at the end where it left of,
    /// when calling this function afterward it will read the `FragmentHeader` from there on.
    /// - Note that only the first fragment of a sequence contains acknowledgment information that's why `AckedPacketHeader` is optional.
    pub fn read_fragment(&mut self) -> Result<(FragmentHeader, Option<AckedPacketHeader>)> {
        if self.can_read(FragmentHeader::size()) {
            let fragment_header = FragmentHeader::read(&mut self.cursor)?;

            let acked_header = if fragment_header.id() == 0 {
                Some(AckedPacketHeader::read(&mut self.cursor)?)
            } else {
                None
            };

            Ok((fragment_header, acked_header))
        } else {
            Err(ErrorKind::CouldNotReadHeader(String::from("fragment")))
        }
    }

    /// Read the payload` from the underlying buffer.
    ///
    /// # Remark
    /// - Notice that this will continue on the position of last read header;
    /// e.g. when reading `StandardHeader` the position of the underlying `Cursor` will be at the end where it left of,
    /// when calling this function afterward it will read all the bytes from there on.
    pub fn read_payload(&self) -> Box<[u8]> {
        self.buffer[self.cursor.position() as usize..self.buffer.len()]
            .to_vec()
            .into_boxed_slice()
    }

    // checks if a given length of bytes could be read with the buffer.
    fn can_read(&self, length: u8) -> bool {
        (self.buffer.len() - self.cursor.position() as usize) >= length as usize
    }
}

#[cfg(test)]
mod tests {
    use crate::packet::header::{AckedPacketHeader, HeaderReader, StandardHeader};
    use crate::packet::{DeliveryGuarantee, OrderingGuarantee, PacketReader, PacketType};

    #[test]
    fn can_read_bytes() {
        let buffer = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

        let reader = PacketReader::new(buffer.as_slice());
        assert_eq!(reader.can_read(buffer.len() as u8), true);
        assert_eq!(reader.can_read((buffer.len() + 1) as u8), false);
    }

    #[test]
    fn assure_read_standard_header() {
        // standard header
        let reliable_ordered_payload: Vec<u8> = vec![vec![0, 1, 0, 1, 2]].concat();

        let mut reader = PacketReader::new(reliable_ordered_payload.as_slice());

        let standard_header = reader.read_standard_header().unwrap();

        assert_eq!(standard_header.protocol_version(), 1);
        assert_eq!(standard_header.packet_type(), PacketType::Packet);
        assert_eq!(
            standard_header.delivery_guarantee(),
            DeliveryGuarantee::Reliable
        );
        assert_eq!(
            standard_header.ordering_guarantee(),
            OrderingGuarantee::Ordered(None)
        );
    }

    #[test]
    fn assure_read_acknowledgment_header() {
        // standard header, acked header
        let reliable_ordered_payload: Vec<u8> =
            vec![vec![0, 1, 0, 1, 2], vec![0, 1, 0, 2, 0, 0, 0, 3]].concat();

        let mut reader = PacketReader::new(reliable_ordered_payload.as_slice());

        let acked_header = reader.read_acknowledge_header().unwrap();

        assert_eq!(acked_header.sequence(), 1);
        assert_eq!(acked_header.ack_seq(), 2);
        assert_eq!(acked_header.ack_field(), 3);
    }

    #[test]
    fn assure_read_fragment_header() {
        // standard header, acked header, arranging header
        let reliable_ordered_payload: Vec<u8> = vec![
            vec![0, 1, 0, 1, 2],
            vec![0, 1, 0, 3],
            vec![0, 1, 0, 2, 0, 0, 0, 3],
        ]
        .concat();

        let mut reader = PacketReader::new(reliable_ordered_payload.as_slice());

        let standard_header = reader.read_standard_header().unwrap();
        let (fragment_header, acked_header) = reader.read_fragment().unwrap();

        assert_eq!(standard_header.protocol_version(), 1);
        assert_eq!(standard_header.packet_type(), PacketType::Packet);
        assert_eq!(
            standard_header.delivery_guarantee(),
            DeliveryGuarantee::Reliable
        );
        assert_eq!(
            standard_header.ordering_guarantee(),
            OrderingGuarantee::Ordered(None)
        );

        assert_eq!(acked_header.unwrap().sequence(), 1);
        assert_eq!(acked_header.unwrap().ack_seq(), 2);
        assert_eq!(acked_header.unwrap().ack_field(), 3);

        assert_eq!(fragment_header.sequence(), 1);
        assert_eq!(fragment_header.id(), 0);
        assert_eq!(fragment_header.fragment_count(), 3);
    }

    #[test]
    fn assure_read_unreliable_sequenced_header() {
        // standard header, arranging header
        let reliable_ordered_payload: Vec<u8> = vec![vec![0, 1, 0, 1, 2], vec![0, 1, 2]].concat();

        let mut reader = PacketReader::new(reliable_ordered_payload.as_slice());

        let arranging_header = reader
            .read_arranging_header(StandardHeader::size() as u16)
            .unwrap();

        assert_eq!(arranging_header.arranging_id(), 1);
        assert_eq!(arranging_header.stream_id(), 2);
    }

    #[test]
    fn assure_read_reliable_ordered_header() {
        // standard header, acked header, arranging header
        let reliable_ordered_payload: Vec<u8> = vec![
            vec![0, 1, 0, 1, 2],
            vec![0, 1, 0, 2, 0, 0, 0, 3],
            vec![0, 1, 2],
        ]
        .concat();
        let mut reader = PacketReader::new(reliable_ordered_payload.as_slice());

        let standard_header = reader.read_standard_header().unwrap();
        let acked_header = reader.read_acknowledge_header().unwrap();
        let arranging_header = reader
            .read_arranging_header((StandardHeader::size() + AckedPacketHeader::size()) as u16)
            .unwrap();

        assert_eq!(standard_header.protocol_version(), 1);
        assert_eq!(standard_header.packet_type(), PacketType::Packet);
        assert_eq!(
            standard_header.delivery_guarantee(),
            DeliveryGuarantee::Reliable
        );
        assert_eq!(
            standard_header.ordering_guarantee(),
            OrderingGuarantee::Ordered(None)
        );

        assert_eq!(acked_header.sequence(), 1);
        assert_eq!(acked_header.ack_seq(), 2);
        assert_eq!(acked_header.ack_field(), 3);

        assert_eq!(arranging_header.arranging_id(), 1);
        assert_eq!(arranging_header.stream_id(), 2);
    }

    #[test]
    fn assure_read_reliable_unordered_header() {
        // standard header, acked header, arranging header
        let reliable_ordered_payload: Vec<u8> =
            vec![vec![0, 1, 0, 1, 2], vec![0, 1, 0, 2, 0, 0, 0, 3]].concat();
        let mut reader = PacketReader::new(reliable_ordered_payload.as_slice());

        let standard_header = reader.read_standard_header().unwrap();
        let acked_header = reader.read_acknowledge_header().unwrap();

        assert_eq!(standard_header.protocol_version(), 1);
        assert_eq!(standard_header.packet_type(), PacketType::Packet);
        assert_eq!(
            standard_header.delivery_guarantee(),
            DeliveryGuarantee::Reliable
        );
        assert_eq!(
            standard_header.ordering_guarantee(),
            OrderingGuarantee::Ordered(None)
        );

        assert_eq!(acked_header.sequence(), 1);
        assert_eq!(acked_header.ack_seq(), 2);
        assert_eq!(acked_header.ack_field(), 3);
    }

    #[test]
    fn expect_read_error() {
        // standard header (with one corrupt byte)
        let reliable_ordered_payload: Vec<u8> = vec![vec![0, 1, 0, 1]].concat();

        let mut reader = PacketReader::new(reliable_ordered_payload.as_slice());

        assert!(reader.read_standard_header().is_err());
    }
}
