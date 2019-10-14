use crossbeam_channel::{self, Receiver, Sender, TryRecvError};
use std::{
    self,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, ToSocketAddrs, UdpSocket},
    thread::{sleep, yield_now},
    time::{Duration, Instant},
};

use crate::{
    config::Config,
    error::Result,
    net::{ConnectionManager, DatagramSocket, SocketWithConditioner},
    packet::Packet,
};

#[cfg(feature = "tester")]
use crate::net::LinkConditioner;

use super::{FactoryImpl, SocketEvent};

/// A reliable UDP socket implementation with configurable reliability and ordering guarantees.
#[derive(Debug)]
pub struct Socket {
    handler: ConnectionManager<SocketWithConditioner, FactoryImpl>,
}

impl Socket {
    /// Binds to the socket and then sets up `ActiveConnections` to manage the "connections".
    /// Because UDP connections are not persistent, we can only infer the status of the remote
    /// endpoint by looking to see if they are still sending packets or not.
    pub fn bind<A: ToSocketAddrs>(addresses: A) -> Result<Self> {
        Self::bind_with_config(addresses, Config::default())
    }

    /// Binds to any local port on the system, if available.
    pub fn bind_any() -> Result<Self> {
        Self::bind_any_with_config(Config::default())
    }

    /// Binds to any local port on the system, if available, with a given config.
    pub fn bind_any_with_config(config: Config) -> Result<Self> {
        let loopback = Ipv4Addr::new(127, 0, 0, 1);
        let address = SocketAddrV4::new(loopback, 0);
        let socket = UdpSocket::bind(address)?;
        Self::bind_internal(socket, config)
    }

    /// Binds to the socket and then sets up `ActiveConnections` to manage the "connections".
    /// Because UDP connections are not persistent, we can only infer the status of the remote
    /// endpoint by looking to see if they are still sending packets or not
    ///
    /// This function allows you to configure laminar with the passed configuration.
    pub fn bind_with_config<A: ToSocketAddrs>(addresses: A, config: Config) -> Result<Self> {
        let socket = UdpSocket::bind(addresses)?;
        Self::bind_internal(socket, config)
    }

    fn bind_internal(socket: UdpSocket, config: Config) -> Result<Self> {
        Ok(Socket {
            handler: ConnectionManager::new(
                SocketWithConditioner::new(socket, config.blocking_mode)?,
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

    /// Sends a single user event.
    pub fn send(&mut self, event: Packet) -> Result<()> {
        self.handler
            .event_sender()
            .send(event)
            .expect("Receiver must exists.");
        Ok(())
    }

    /// Receives a single connection event.
    pub fn recv(&mut self) -> Option<SocketEvent> {
        match self.handler.event_receiver().try_recv() {
            Ok(event) => Some(event),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => panic!["This can never happen"],
        }
    }

    /// Runs the polling loop with the default '1ms' sleep duration. This should run in a spawned thread
    /// since calls to `self.manual_poll` are blocking.
    pub fn start_polling(&mut self) {
        self.start_polling_with_duration(Some(Duration::from_millis(1)))
    }

    /// Runs the polling loop with a specified sleep duration. This should run in a spawned thread
    /// since calls to `self.manual_poll` are blocking.
    pub fn start_polling_with_duration(&mut self, sleep_duration: Option<Duration>) {
        // Nothing should break out of this loop!
        loop {
            self.handler.manual_poll(Instant::now());
            match sleep_duration {
                None => yield_now(),
                Some(duration) => sleep(duration),
            };
        }
    }

    /// Processes any inbound/outbound packets and handle idle clients.
    pub fn manual_poll(&mut self, time: Instant) {
        self.handler.manual_poll(time);
    }

    /// Returns the local socket address
    pub fn local_addr(&self) -> Result<SocketAddr> {
        Ok(self.handler.socket().local_addr()?)
    }

    /// Sets the link conditioner for this socket. See [LinkConditioner] for further details.
    #[cfg(feature = "tester")]
    pub fn set_link_conditioner(&mut self, link_conditioner: Option<LinkConditioner>) {
        self.handler
            .socket_mut()
            .set_link_conditioner(link_conditioner);
    }
}
