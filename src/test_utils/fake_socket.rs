use std::{net::SocketAddr, time::Instant};

use crossbeam_channel::{Receiver, Sender};

use crate::net::{ConnectionManager, FactoryImpl, LinkConditioner};
use crate::test_utils::*;
use crate::{error::Result, Config, Packet, SocketEvent};

/// Provides a similar to the real a `Socket`, but with emulated socket implementation.
pub struct FakeSocket {
    handler: ConnectionManager<EmulatedSocket, FactoryImpl>,
}

impl FakeSocket {
    /// Binds to the socket.
    pub fn bind(network: &NetworkEmulator, addr: SocketAddr, config: Config) -> Result<Self> {
        Ok(Self {
            handler: ConnectionManager::new(
                network.new_socket(addr)?,
                FactoryImpl::new(config.clone()),
                config,
            ),
        })
    }

    /// Returns a handle to the packet sender which provides a thread-safe way to enqueue packets
    /// to be processed. This should be used when the socket is busy running its polling loop in a
    /// separate thread.
    pub fn get_packet_sender(&self) -> Sender<Packet> {
        self.handler.event_sender().clone()
    }

    /// Returns a handle to the event receiver which provides a thread-safe way to retrieve events
    /// from the socket. This should be used when the socket is busy running its polling loop in
    /// a separate thread.
    pub fn get_event_receiver(&self) -> Receiver<SocketEvent> {
        self.handler.event_receiver().clone()
    }

    /// Sends a packet.
    pub fn send(&mut self, packet: Packet) -> Result<()> {
        // we can savely unwrap, because receiver will always exist
        self.handler.event_sender().send(packet).unwrap();
        Ok(())
    }

    /// Receives a packet.
    pub fn recv(&mut self) -> Option<SocketEvent> {
        if let Ok(event) = self.handler.event_receiver().try_recv() {
            Some(event)
        } else {
            None
        }
    }

    /// Processes any inbound/outbound packets and handle idle clients.
    pub fn manual_poll(&mut self, time: Instant) {
        self.handler.manual_poll(time);
    }

    /// Returns a number of active connections.
    pub fn connection_count(&self) -> usize {
        self.handler.connections_count()
    }

    /// Sets the link conditioner for this socket. See [LinkConditioner] for further details.
    pub fn set_link_conditioner(&mut self, conditioner: Option<LinkConditioner>) {
        self.handler.socket_mut().set_link_conditioner(conditioner);
    }
}
