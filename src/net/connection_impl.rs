use crate::{
    error::{ErrorKind, Result},
    net::{Connection, ConnectionMessenger, VirtualConnection},
    packet::{DeliveryGuarantee, OutgoingPackets, Packet, PacketInfo},
};

use super::events::SocketEvent;

use std::net::SocketAddr;
use std::time::Instant;

use log::error;

pub struct ConnectionImpl {
    pub non_accepted_timeout: Option<Instant>,
    pub conn: VirtualConnection,
}

impl std::fmt::Debug for ConnectionImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}:{}",
            self.conn.remote_address.ip(),
            self.conn.remote_address.port()
        )
    }
}

impl Connection for ConnectionImpl {
    /// Defines a user event type.
    type UserEvent = Packet;
    /// Defines a connection event type.
    type ConnectionEvent = SocketEvent;

    /// Initial call with a payload, when connection is created by accepting remote packet.
    fn after_remote_accepted(
        &mut self,
        time: Instant,
        messenger: &mut impl ConnectionMessenger<Self::ConnectionEvent>,
        payload: &[u8],
    ) {
        // emit connect event, for remote connection
        messenger.send_event(SocketEvent::Connect(self.conn.remote_address));
        self.process_packet(time, messenger, payload);
    }

    /// Initial call with a event, when connection is created by accepting user event.
    fn after_local_accepted(
        &mut self,
        time: Instant,
        messenger: &mut impl ConnectionMessenger<Self::ConnectionEvent>,
        event: Self::UserEvent,
    ) {
        self.process_event(time, messenger, event);
    }

    /// Processes a received packet: parse it and emit an connection event.
    fn process_packet(
        &mut self,
        time: Instant,
        messenger: &mut impl ConnectionMessenger<Self::ConnectionEvent>,
        payload: &[u8],
    ) {
        if !payload.is_empty() {
            match self.conn.process_incoming(payload, time) {
                Ok(packets) => {
                    for incoming in packets {
                        messenger.send_event(SocketEvent::Packet(incoming.0));
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

    /// Processes an user event and send a packet.
    fn process_event(
        &mut self,
        time: Instant,
        messenger: &mut impl ConnectionMessenger<Self::ConnectionEvent>,
        event: Self::UserEvent,
    ) {
        self.non_accepted_timeout = None;
        let addr = self.conn.remote_address;
        send_packets(
            messenger,
            &addr,
            self.conn.process_outgoing(
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
    /// This function gets called frequently.
    fn update(
        &mut self,
        time: Instant,
        messenger: &mut impl ConnectionMessenger<Self::ConnectionEvent>,
    ) {
        // resend dropped packets
        for dropped in self.conn.gather_dropped_packets() {
            let packets = self.conn.process_outgoing(
                PacketInfo {
                    packet_type: dropped.packet_type,
                    payload: &dropped.payload,
                    // because a delivery guarantee is only sent with reliable packets
                    delivery: DeliveryGuarantee::Reliable,
                    // this is stored with the dropped packet because they could be mixed
                    ordering: dropped.ordering_guarantee,
                },
                dropped.item_identifier,
                time,
            );
            send_packets(
                messenger,
                &self.conn.remote_address,
                packets,
                "dropped packets",
            );
        }

        // send heartbeat packets if required
        if let Some(heartbeat_interval) = messenger.config().heartbeat_interval {
            let addr = self.conn.remote_address;
            if self.conn.last_sent(time) >= heartbeat_interval {
                send_packets(
                    messenger,
                    &addr,
                    self.conn
                        .process_outgoing(PacketInfo::heartbeat_packet(&[]), None, time),
                    "heatbeat packet",
                );
            }
        }
    }

    /// Last call before connection is destroyed.
    fn before_discarded(
        &mut self,
        _time: Instant,
        messenger: &mut impl ConnectionMessenger<Self::ConnectionEvent>,
    ) {
        messenger.send_event(SocketEvent::Timeout(self.conn.remote_address));
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
