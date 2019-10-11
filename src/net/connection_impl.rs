use std::net::SocketAddr;
use std::time::Instant;

use log::error;

use crate::error::{ErrorKind, Result};
use crate::packet::{DeliveryGuarantee, OutgoingPackets, Packet, PacketInfo};

use super::{
    events::SocketEvent, Connection, ConnectionEventAddress, ConnectionMessenger, VirtualConnection,
};

/// Required by `ConnectionManager` to properly handle connection event.
impl ConnectionEventAddress for SocketEvent {
    /// Returns event address.
    fn address(&self) -> SocketAddr {
        match self {
            SocketEvent::Packet(packet) => packet.addr(),
            SocketEvent::Connect(addr) => *addr,
            SocketEvent::Timeout(addr) => *addr,
        }
    }
}

/// Required by `ConnectionManager` to properly handle user event.
impl ConnectionEventAddress for Packet {
    /// Returns event address.
    fn address(&self) -> SocketAddr {
        self.addr()
    }
}

impl Connection for VirtualConnection {
    /// Defines a user event type.
    type SendEvent = Packet;
    /// Defines a connection event type.
    type ReceiveEvent = SocketEvent;

    /// Creates new connection and initialize it by sending an connection event to the user.
    /// * address - defines a address that connection is associated with.
    /// * time - creation time, used by connection, so that it doesn't get dropped immediately or send heartbeat packet.
    /// * initial_data - if initiated by remote host, this will hold that a packet data.
    fn create_connection(
        messenger: &mut impl ConnectionMessenger<Self::ReceiveEvent>,
        address: SocketAddr,
        time: Instant,
        initial_data: Option<&[u8]>,
    ) -> VirtualConnection {
        // Emit connect event if this is initiated by the remote host.
        if initial_data.is_some() {
            messenger.send_event(&address, SocketEvent::Connect(address));
        }
        VirtualConnection::new(address, messenger.config(), time)
    }

    /// Determines if the given `Connection` should be dropped due to its state.
    fn should_drop(
        &mut self,
        messenger: &mut impl ConnectionMessenger<Self::ReceiveEvent>,
        time: Instant,
    ) -> bool {
        let should_drop = self.packets_in_flight() > messenger.config().max_packets_in_flight
            || self.last_heard(time) >= messenger.config().idle_connection_timeout;
        if should_drop {
            messenger.send_event(
                &self.remote_address,
                SocketEvent::Timeout(self.remote_address),
            );
        }
        should_drop
    }

    /// Processes a received packet: parse it and emit an event.
    fn process_packet(
        &mut self,
        messenger: &mut impl ConnectionMessenger<Self::ReceiveEvent>,
        payload: &[u8],
        time: Instant,
    ) {
        if !payload.is_empty() {
            match self.process_incoming(payload, time) {
                Ok(packets) => {
                    for incoming in packets {
                        messenger.send_event(&self.remote_address, SocketEvent::Packet(incoming.0));
                    }
                }
                Err(err) => error!("Error occured processing incomming packet: {:?}", err),
            }
        } else {
            error!(
                "Error processing packet: {}",
                ErrorKind::ReceivedDataToShort
            );
        }
    }

    /// Processes a received event and send a packet.
    fn process_event(
        &mut self,
        messenger: &mut impl ConnectionMessenger<Self::ReceiveEvent>,
        event: Self::SendEvent,
        time: Instant,
    ) {
        let addr = self.remote_address;
        send_packets(
            messenger,
            &addr,
            self.process_outgoing(
                PacketInfo::user_packet(
                    event.payload(),
                    event.delivery_guarantee(),
                    event.order_guarantee(),
                ),
                None,
                time,
            ),
            "user packet",
        );
    }

    /// Processes various connection-related tasks: resend dropped packets, send heartbeat packet, etc...
    /// This function gets called very frequently.
    fn update(
        &mut self,
        messenger: &mut impl ConnectionMessenger<Self::ReceiveEvent>,
        time: Instant,
    ) {
        // Resend dropped packets
        for dropped in self.gather_dropped_packets() {
            let packets = self.process_outgoing(
                PacketInfo {
                    packet_type: dropped.packet_type,
                    payload: &dropped.payload,
                    // Because a delivery guarantee is only sent with reliable packets
                    delivery: DeliveryGuarantee::Reliable,
                    // This is stored with the dropped packet because they could be mixed
                    ordering: dropped.ordering_guarantee,
                },
                dropped.item_identifier,
                time,
            );
            send_packets(messenger, &self.remote_address, packets, "dropped packets");
        }

        // Send heartbeat packets if required
        if let Some(heartbeat_interval) = messenger.config().heartbeat_interval {
            let addr = self.remote_address;
            if self.last_sent(time) >= heartbeat_interval {
                send_packets(
                    messenger,
                    &addr,
                    self.process_outgoing(PacketInfo::heartbeat_packet(&[]), None, time),
                    "heatbeat packet",
                );
            }
        }
    }
}

// Sends multiple outgoing packets.
fn send_packets(
    ctx: &mut impl ConnectionMessenger<SocketEvent>,
    address: &SocketAddr,
    packets: Result<OutgoingPackets>,
    err_context: &str,
) {
    match packets {
        Ok(packets) => {
            for outgoing in packets {
                ctx.send_packet(address, &outgoing.contents());
            }
        }
        Err(error) => error!("Error occured processing {}: {:?}", err_context, error),
    }
}
