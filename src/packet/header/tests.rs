use super::{PacketHeader,FragmentHeader, HeaderParser, HeaderReader};
use byteorder::ReadBytesExt;
use std::io::Cursor;

#[test]
pub fn packet_header_serializes_deserializes()
{
    let packet_header = PacketHeader::new(1,1,5421);
    let packet_serialized: Vec<u8> = packet_header.parse().unwrap();

    let mut cursor = Cursor::new(packet_serialized);
    let packet_deserialized: PacketHeader = PacketHeader::read(&mut cursor).unwrap();

    assert_eq!(packet_header.seq, 1);
    assert_eq!(packet_header.ack_seq, 1);
    assert_eq!(packet_header.ack_field, 5421);
}

#[test]
pub fn fragment_header_serializes_deserializes()
{
    let packet_header = PacketHeader::new(1,1,5421);
    let packet_serialized: Vec<u8> = packet_header.parse().unwrap();

    let fragment = FragmentHeader::new(0, 1, packet_header.clone());
    let fragment_serialized = fragment.parse().unwrap();

    let mut cursor = Cursor::new(fragment_serialized);
    let fragment_deserialized: FragmentHeader = FragmentHeader::read(&mut cursor).unwrap();

    assert_eq!(fragment_deserialized.id, 0);
    assert_eq!(fragment_deserialized.num_fragments, 1);
    assert_eq!(fragment_deserialized.sequence, 1);

    assert!(fragment_deserialized.packet_header.is_some());

    let fragment_packet_header = fragment_deserialized.packet_header.unwrap();
    assert_eq!(fragment_packet_header.seq, 1);
    assert_eq!(fragment_packet_header.ack_seq, 1);
    assert_eq!(fragment_packet_header.ack_field, 5421);
}


