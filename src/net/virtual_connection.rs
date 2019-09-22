use crate::{
    error::{ErrorKind, Result},
    net::events::{ConnectionEvent, DestroyReason, DisconnectReason, ReceiveEvent},
    net::managers::{
        ConnectionManager, ConnectionManagerError, ConnectionManagerEvent, ConnectionState,
    },
    net::{MetricsCollector, SocketWithConditioner},
    net::{OutgoingPackets, ReliabilitySystem},
    packet::{GenericPacket, Packet, PacketType},
};

use crossbeam_channel::{self, Sender};
use std::fmt;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

/// Contains the information about a certain 'virtual connection' over udp.
/// This connections has core components that manages connection state, and has reliability information.
pub struct VirtualConnection {
    remote_address: SocketAddr,
    reliability_system: ReliabilitySystem,
    connection_manager: Box<dyn ConnectionManager>,
    current_state: ConnectionState,
}

impl VirtualConnection {
    /// Creates new VirtualConnection.
    pub fn new(
        remote_address: SocketAddr,
        reliability_system: ReliabilitySystem,
        connection_manager: Box<dyn ConnectionManager>,
    ) -> VirtualConnection {
        Self {
            remote_address,
            reliability_system,
            connection_manager,
            current_state: Default::default(),
        }
    }

    /// Returns connection remote address.
    pub fn remote_address(&self) -> SocketAddr {
        self.remote_address
    }

    /// Invokes connect request on ConnectionManager.
    pub fn connect(&mut self, payload: Box<[u8]>) {
        self.connection_manager.connect(payload);
    }

    /// Invokes disconnect request on ConnectionManager.
    pub fn disconnect(&mut self) {
        self.connection_manager.disconnect();
    }

    /// Returns current connection state: `Connecting`, `Connected` or `Disconnected`.
    pub fn get_current_state(&self) -> &ConnectionState {
        &self.current_state
    }

    /// Returns whether connection should be dropped, and provides a drop reason.
    pub fn should_be_dropped(
        &self,
        max_idle_time: Duration,
        time: Instant,
    ) -> Option<DestroyReason> {
        if self.reliability_system.should_be_dropped() {
            Some(DestroyReason::TooManyPacketsInFlight)
        } else if let ConnectionState::Disconnected(_) = self.current_state {
            Some(DestroyReason::GracefullyDisconnected)
        } else if self.reliability_system.last_heard(time) >= max_idle_time {
            Some(DestroyReason::Timeout)
        } else {
            None
        }
    }

    /// Checks if connection should send heartbeat packet and sends it.
    pub fn handle_heartbeat(
        &mut self,
        time: Instant,
        heartbeat_interval: Duration,
        socket: &mut SocketWithConditioner,
        metrics: &mut MetricsCollector,
    ) {
        if self.reliability_system.last_sent(time) >= heartbeat_interval {
            send_packets(
                &self.remote_address,
                self.reliability_system
                    .process_outgoing(GenericPacket::heartbeat_packet(&[]), time),
                self.connection_manager.as_mut(),
                socket,
                metrics,
                "sending heartbeat packet",
            );
        }
    }

    /// Resends all dropped packets.
    pub fn resend_dropped_packets(
        &mut self,
        time: Instant,
        socket: &mut SocketWithConditioner,
        metrics: &mut MetricsCollector,
    ) {
        for dropped in self.reliability_system.gather_dropped_packets() {
            send_packets(
                &self.remote_address,
                self.reliability_system.process_dropped(&dropped, time),
                self.connection_manager.as_mut(),
                socket,
                metrics,
                "sending dropped packet",
            );
        }
    }

    /// Processes outgoing packet, and sends it.
    pub fn process_outgoing(
        &mut self,
        packet: &Packet,
        time: Instant,
        socket: &mut SocketWithConditioner,
        metrics: &mut MetricsCollector,
    ) {
        let packets = self.reliability_system.process_outgoing(
            GenericPacket {
                packet_type: PacketType::Packet,
                payload: packet.payload(),
                delivery: packet.delivery_guarantee(),
                ordering: packet.order_guarantee(),
            },
            time,
        );

        send_packets(
            &self.remote_address,
            packets,
            self.connection_manager.as_mut(),
            socket,
            metrics,
            "sending outgoing packet",
        );
    }

    /// Processes incoming bytes by converting to packets and process them.
    pub fn process_incoming(
        &mut self,
        received_payload: &[u8],
        mut tmp_buffer: &mut [u8],
        time: Instant,
        event_sender: &Sender<ConnectionEvent<ReceiveEvent>>,
        metrics: &mut MetricsCollector,
    ) -> Result<()> {
        let received_payload = self
            .connection_manager
            .preprocess_incoming(received_payload, &mut tmp_buffer)?;
        let packets = self.reliability_system.process_incoming(
            self.remote_address,
            received_payload,
            time,
        )?;
        for (packet, packet_type) in packets {
            if packet_type != PacketType::Connection {
                if let ConnectionState::Connected(_) = self.current_state {
                    if let Err(err) = event_sender.send(ConnectionEvent(
                        self.remote_address,
                        ReceiveEvent::Packet(packet),
                    )) {
                        metrics.track_connection_error(
                            &self.remote_address,
                            &ErrorKind::from(err),
                            "sending incoming packet",
                        );
                    }
                }
            } else if let Err(err) = self
                .connection_manager
                .process_protocol_data(packet.payload())
            {
                metrics.track_connection_error(
                    &self.remote_address,
                    &ErrorKind::from(err),
                    "processing connection manager data",
                );
            }
        }
        Ok(())
    }

    /// Calls `update` method for `ConnectionManager`, in the loop, until it returns None
    /// These updates returns either new packets to be sent, or connection state changes.
    pub fn update_connection_manager(
        &mut self,
        sender: &Sender<ConnectionEvent<ReceiveEvent>>,
        metrics: &mut MetricsCollector,
        socket: &mut SocketWithConditioner,
        time: Instant,
        buffer: &mut [u8],
    ) {
        while let Some(changes) = self.connection_manager.update(buffer, time) {
            match changes {
                ConnectionManagerEvent::NewPacket(packet) => {
                    send_packets(
                        &self.remote_address,
                        self.reliability_system.process_outgoing(packet, time),
                        self.connection_manager.as_mut(),
                        socket,
                        metrics,
                        "sending packet from connection manager",
                    );
                }
                ConnectionManagerEvent::NewState(state) => {
                    if let Some(old) = self.current_state.try_change(&state) {
                        if let Err(err) = match &self.current_state {
                            ConnectionState::Connected(data) => sender.send(ConnectionEvent(
                                self.remote_address,
                                ReceiveEvent::Connected(data.clone()),
                            )),
                            ConnectionState::Disconnected(closed_by) => {
                                sender.send(ConnectionEvent(
                                    self.remote_address,
                                    ReceiveEvent::Disconnected(DisconnectReason::ClosedBy(
                                        closed_by.clone(),
                                    )),
                                ))
                            }
                            _ => {
                                metrics.track_connection_error(
                                    &self.remote_address,
                                    &ErrorKind::ConnectionError(ConnectionManagerError::Fatal(
                                        format!(
                                            "Invalid state transition: {:?} -> {:?}",
                                            old, self.current_state
                                        ),
                                    )),
                                    "changing connection manager state",
                                );
                                Ok(())
                            }
                        } {
                            metrics.track_connection_error(
                                &self.remote_address,
                                &ErrorKind::from(err),
                                "sending connection state update",
                            );
                        }
                    } else {
                        metrics.track_connection_error(
                            &self.remote_address,
                            &ErrorKind::ConnectionError(ConnectionManagerError::Fatal(format!(
                                "Invalid state transition: {:?} -> {:?}",
                                self.current_state, state
                            ))),
                            "changing connection manager state",
                        );
                    }
                }
                ConnectionManagerEvent::Error(err) => {
                    metrics.track_connection_error(
                        &self.remote_address,
                        &ErrorKind::ConnectionError(err),
                        "recieved connection manager error",
                    );
                }
            };
        }
    }
}

impl fmt::Debug for VirtualConnection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}:{}",
            self.remote_address.ip(),
            self.remote_address.port()
        )
    }
}

// Helper method, that takes outgoing packets from reliability system and sends them.
fn send_packets(
    address: &SocketAddr,
    packets_result: Result<OutgoingPackets>,
    connection_manager: &mut dyn ConnectionManager,
    socket: &mut SocketWithConditioner,
    metrics: &mut MetricsCollector,
    err_context: &str,
) {
    match packets_result {
        Ok(packets) => {
            for outgoing in packets {
                socket.send_packet_and_log(
                    address,
                    connection_manager,
                    &outgoing.contents(),
                    metrics,
                    err_context,
                );
            }
        }
        Err(err) => metrics.track_connection_error(address, &err, err_context),
    }
}
