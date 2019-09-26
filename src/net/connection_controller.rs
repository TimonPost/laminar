use crate::{
    config::Config,
    error::Result,
    net::{events::SocketEvent, SocketSender, VirtualConnection},
    packet::{DeliveryGuarantee, OutgoingPackets, Packet, PacketInfo},
};
use crossbeam_channel::Sender;
use log::error;
use std::{self, net::SocketAddr, time::Instant};

/// Controls all aspects of the connection:
/// * Processes incoming data (from a socket) or events (from a user).
/// * Updates connection state: resends dropped packets, sends heartbeat packet, etc.
/// * Creates new connections.
/// * Checks if connection should be dropped.
/// It doesn't own connections, but only owns necessary components to process them.
#[derive(Debug)]
pub struct ConnectionController<PacketSender> {
    config: Config,
    packet_sender: PacketSender,
    event_sender: Sender<SocketEvent>,
}

/// Defines a connection type.
type Connection = VirtualConnection;
/// Defines a user event type.
type UserEvent = Packet;
/// Defines a connection event type.
type ConnectionEvent = SocketEvent;

impl<PacketSender: SocketSender> ConnectionController<PacketSender> {
    /// Creates a new instance of `ConnectionHandler`.
    pub fn new(
        config: Config,
        packet_sender: PacketSender,
        event_sender: Sender<ConnectionEvent>,
    ) -> Self {
        ConnectionController {
            config,
            packet_sender,
            event_sender,
        }
    }

    /// Creates new connection. Also will init it and send connection event to a user.
    pub fn create_connection(
        &self,
        address: SocketAddr,
        time: Instant,
        initial_data: Option<&[u8]>,
    ) -> Connection {
        // emit connect event if this is initiated by remote host.
        if initial_data.is_some() {
            self.event_sender
                .send(ConnectionEvent::Connect(address))
                .unwrap();
        }
        Connection::new(address, &self.config, time)
    }

    /// Determine if this connection should be dropped due to its state.
    pub fn should_drop(&self, connection: &mut Connection, time: Instant) -> bool {
        let should_drop = connection.packets_in_flight() > self.config.max_packets_in_flight
            || connection.last_heard(time) >= self.config.idle_connection_timeout;
        if should_drop {
            self.event_sender
                .send(ConnectionEvent::Timeout(connection.remote_address))
                .unwrap();
        }
        should_drop
    }

    /// Handle a packet received from a socket.
    pub fn handle_packet(&mut self, connection: &mut Connection, payload: &[u8], time: Instant) {
        match connection.process_incoming(payload, time) {
            Ok(packets) => {
                for incoming in packets {
                    self.event_sender
                        .send(ConnectionEvent::Packet(incoming.0))
                        .unwrap();
                }
            }
            Err(err) => error!("Error occured processing incomming packet: {:?}", err),
        }
    }

    /// Handle an event received from a user.
    pub fn handle_event(&mut self, connection: &mut Connection, event: UserEvent, time: Instant) {
        self.send_packets(
            &connection.remote_address.clone(),
            connection.process_outgoing(
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

    /// Process various connection related tasks: resend dropped packets, send heartbeat packet, etc...
    /// This function gets called very frequently.
    pub fn update(&mut self, connection: &mut Connection, time: Instant) {
        // resend dropped packets
        let dropped_packets = connection.gather_dropped_packets();
        for dropped in dropped_packets {
            let packets = connection.process_outgoing(
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
            self.send_packets(&connection.remote_address, packets, "dropped packets");
        }

        // send heartbeat packets if required.
        if let Some(heartbeat_interval) = self.config.heartbeat_interval {
            if connection.last_sent(time) >= heartbeat_interval {
                self.send_packets(
                    &connection.remote_address.clone(),
                    connection.process_outgoing(PacketInfo::heartbeat_packet(&[]), None, time),
                    "heatbeat packet",
                );
            }
        }
    }

    /// Helper function that sends multiple outgoing packets
    fn send_packets(
        &mut self,
        address: &SocketAddr,
        packets: Result<OutgoingPackets>,
        err_context: &str,
    ) {
        match packets {
            Ok(packets) => {
                for outgoing in packets {
                    if let Err(error) = self
                        .packet_sender
                        .send_packet(address, &outgoing.contents())
                    {
                        error!("Error occured sending {}: {:?}", err_context, error);
                    }
                }
            }
            Err(error) => error!("Error occured processing {}: {:?}", err_context, error),
        }
    }
}
