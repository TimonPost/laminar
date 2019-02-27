use crate::net::constants::STANDARD_HEADER_SIZE;
use crate::packet::header::{
    AckedPacketHeader, ArrangingHeader, FragmentHeader, HeaderReader, StandardHeader,
};
use crate::{IntoBoxedSlice, Result};

use std::io::Cursor;

/// Could be used to read the packet contents of laminar.
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
        StandardHeader::read(&mut self.cursor)
    }

    /// Read the `StandardHeader` from the underlying buffer.
    ///
    /// # Remark
    /// - Will change the position to the location of `StandardHeader`
    pub fn read_arranging_header(&mut self, start_offset: u16) -> Result<ArrangingHeader> {
        self.cursor.set_position(u64::from(start_offset));
        ArrangingHeader::read(&mut self.cursor)
    }

    /// Read the `AckedPacketHeader` from the underlying buffer.
    ///
    /// # Remark
    /// - Will change the position to the location of `AckedPacketHeader`
    pub fn read_acknowledge_header(&mut self) -> Result<AckedPacketHeader> {
        // acknowledge header comes after standard header.
        self.cursor.set_position(u64::from(STANDARD_HEADER_SIZE));
        AckedPacketHeader::read(&mut self.cursor)
    }

    /// Read the payload` from the underlying buffer.
    ///
    /// # Remark
    /// - Notice that this will continue on the position of last read header;
    /// e.g. when reading `StandardHeader` the position of the underlying `Cursor` will be at the end where it left of,
    /// when calling this function afterward it will read all the bytes from there on.
    pub fn read_payload(&self) -> Box<[u8]> {
        self.buffer
            .into_boxed_slice(self.cursor.position() as usize, self.buffer.len())
    }

    /// Read the `FragmentHeader` and optional the `AckedPacketHeader` from the underlying buffer.
    ///
    /// # Remark
    /// - Notice that this will continue on the position of last read header;
    /// e.g. when reading `StandardHeader` the position of the underlying `Cursor` will be at the end where it left of,
    /// when calling this function afterward it will read the `FragmentHeader` from there on.
    /// - Note that only the first fragment of a sequence contains acknowledgement information that's why `AckedPacketHeader` is optional.
    pub fn read_fragment(&mut self) -> Result<(FragmentHeader, Option<AckedPacketHeader>)> {
        let fragment_header = FragmentHeader::read(&mut self.cursor)?;

        let acked_header = if fragment_header.id() == 0 {
            Some(AckedPacketHeader::read(&mut self.cursor)?)
        } else {
            None
        };

        Ok((fragment_header, acked_header))
    }
}

#[cfg(test)]
mod tests {
    use crate::packet::header::HeaderReader;
    use crate::packet::{DeliveryGuarantee, OrderingGuarantee, PacketReader, PacketType};

    #[test]
    fn read_reliable_ordered() {
        // standard header, acked header, arranging header
        let reliable_ordered_payload: Vec<u8> = vec![
            vec![0, 0, 0, 1, 0, 1, 2],
            vec![0, 1, 0, 2, 0, 0, 0, 3],
            vec![0, 1, 2],
        ]
        .concat();
        let mut reader = PacketReader::new(reliable_ordered_payload.as_slice());

        let standard_header = reader.read_standard_header().unwrap();
        let acked_header = reader.read_acknowledge_header().unwrap();
        let arranging_header = reader
            .read_arranging_header((standard_header.size() + acked_header.size()) as u16)
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
}
