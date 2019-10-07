use crate::net::{LinkConditioner, SocketImpl};
use crate::test_utils::*;
use crate::{error::Result, Config, Packet, SocketEvent};
use crossbeam_channel::{Receiver, Sender};

use std::{cell::RefCell, net::SocketAddr, rc::Rc, time::Instant};

/// Provides a similar to the real a `Socket`, but with emulated socket implementation.
pub struct FakeSocket {
    handler: SocketImpl<EmulatedSocket, EmulatedSocket>,
    // store Rc to link conditioner, so we can set it in the `EmulatedSocket`.
    link_conditioner: Rc<RefCell<Option<LinkConditioner>>>,
}

impl FakeSocket {
    /// Binds to the socket.
    pub fn bind(network: &NetworkEmulator, addr: SocketAddr, config: Config) -> Result<Self> {
        let link_conditioner = Rc::new(RefCell::new(Default::default()));
        let mut socket = network.new_socket(addr)?;
        socket.set_link_conditioner(link_conditioner.clone());
        Ok(Self {
            handler: SocketImpl::new(socket.clone(), socket.clone(), config),
            link_conditioner,
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
        *self.link_conditioner.borrow_mut() = conditioner;
    }
}
