# Packet Headers 
In this topic we'll discuss the different headers we are pre-pending to the data sent via laminar. 

## Standard header
Will be included in each packet.
```rust 
pub struct StandardHeader {
    /// crc32 of the protocol version.
    pub protocol_version: u32,
    /// specifies the packet type.
    pub packet_type_id: PacketTypeId,
    /// specifies how this packet should be processed.
    pub delivery_method: DeliveryMethod,
}
```
## Fragment header.
This is the header containing header fragment information prefix with tha standard header. 
The first first fragment will also contain information the parent packet information.
```rust
pub struct FragmentHeader {
    standard_header: StandardHeader,
    // this is the sequence number to which the fragment belongs.
    sequence: u16,
    // this is the id from the fragment sho we know how to order the fragments on the other side.
    id: u8,
    // the number of fragments to which this fragment belongs.
    num_fragments: u8,
    // acked information of which will be included into the first fragment with id '1'.
    packet_header: Option<AckedPacketHeader>,
}
```

## Acked packet header.
This will be used for reliable packets.
```rust
pub struct AckedPacketHeader {
    pub standard_header: StandardHeader,
    /// this is the sequence number so that we can know where in the sequence of packages this packet belongs.
    pub seq: u16,
    // this is the last acknowledged sequence number.
    ack_seq: u16,
    // this is an bitfield of all last 32 acknowledged packages
    ack_field: u32,
}
```