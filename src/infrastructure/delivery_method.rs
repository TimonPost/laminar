/// This enum defines on how an packet would be delivered to the other side.
#[derive(Copy, Clone, Debug,PartialOrd, PartialEq, Eq)]
pub enum DeliveryMethod
{
    /// Unreliable. Packets can be dropped, duplicated or arrive without order
    ///
    /// *Details*
    ///
    ///  1. Unreliable
    ///  2. No guarantee for delivery.
    ///  3. No guarantee for order.
    ///  4. No way of getting dropped packet
    ///  5. Duplication possible
    ///
    /// Basically just bare UDP
    Unreliable,
    /// Reliable. All packets will be sent and received, but without order
    ///
    /// *Details*
    ///
    ///  1. Reliable.
    ///  2. Guarantee of delivery.
    ///  3. No guarantee for order.
    ///  4. Packets will not be dropped.
    ///  5. Duplication not possible
    ///
    /// Basically this is almost TCP like without ordering of packets.
    ReliableUnordered,
    /// Unreliable. Packets can be dropped, but never duplicated and arrive in order
    ///
    /// This will create an reliable ordered packet.
    ///
    /// *Details*
    ///
    ///  1. Unreliable.
    ///  2. No guarantee of delivery.
    ///  3. Guarantee for order.
    ///  4. Packets can be dropped but you will be able to retrieve dropped packets.
    ///  5. Duplication not possible
    ///
    /// Basically this is UDP with the ability to retrieve dropped packets by acknowledgements.
    SequencedOrdered,
    /// Unreliable. Packets can be dropped, and arrive out of order but you will be able to retrieve dropped packet.
    ///
    /// *Details*
    ///
    ///  1. Unreliable.
    ///  2. No guarantee of delivery.
    ///  3. No Guarantee for order.
    ///  4. Packets can be dropped but you will be able to retrieve dropped packets.
    ///  5. Duplication not possible
    ///
    /// Basically this is UDP with the ability to retrieve dropped packets by acknowledgements.
    SequencedUnordered,
    /// *Details*
    ///
    ///  1. Reliable.
    ///  2. Guarantee of delivery.
    ///  3. Guarantee for order.
    ///  4. Packets will not be dropped.
    ///  5. Duplication not possible
    ///
    /// Basically this is almost TCP like with ordering of packets.
    ReliableOrdered,
}

impl DeliveryMethod {
    /// Get integer value from `DeliveryMethod` enum.
    pub fn get_delivery_method_id(delivery_method: DeliveryMethod) -> u8 {
        delivery_method as u8
    }

    /// Get `DeliveryMethod` enum instance from integer value.
    pub fn get_delivery_method_from_id(delivery_method_id: u8) -> DeliveryMethod {
        match delivery_method_id {
            0 => DeliveryMethod::Unreliable,
            1 => DeliveryMethod::ReliableUnordered,
            2 => DeliveryMethod::ReliableOrdered,
            3 => DeliveryMethod::SequencedOrdered,
            4 => DeliveryMethod::SequencedOrdered,
            _ => DeliveryMethod::Unreliable
        }
    }
}