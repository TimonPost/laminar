use crate::{
    config::Config,
    error::Result,
    net::{events::SocketEvent, LinkConditioner, SocketImpl, SocketReceiver, SocketSender},
    packet::Packet,
};
use crossbeam_channel::{self, Receiver, Sender, TryRecvError};
use std::{
    self,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, ToSocketAddrs, UdpSocket},
    sync::{Arc, Mutex},
    thread::{sleep, yield_now},
    time::{Duration, Instant},
};

// Wrap `LinkConditioner` and `UdpSocket` together. LinkConditioner is enabled when building with a "tester" feature.
#[derive(Debug)]
struct SocketWithConditioner {
    socket: UdpSocket,
    link_conditioner: Arc<Mutex<Option<LinkConditioner>>>,
}

impl SocketWithConditioner {
    pub fn new(socket: UdpSocket, link_conditioner: Arc<Mutex<Option<LinkConditioner>>>) -> Self {
        SocketWithConditioner {
            socket,
            link_conditioner,
        }
    }
}
/// Provides a `SocketSender` implementation for `SocketWithConditioner`
impl SocketSender for SocketWithConditioner {
    // When `LinkConditioner` is enabled, it will determine whether packet will be sent or not.
    fn send_packet(&mut self, addr: &SocketAddr, payload: &[u8]) -> std::io::Result<usize> {
        if cfg!(feature = "tester") {
            if let Some(ref mut link) = *self
                .link_conditioner
                .lock()
                .expect("Lock is already held by current thread.")
            {
                if !link.should_send() {
                    return Ok(0);
                }
            }
        }
        self.socket.send_to(payload, addr)
    }
}

/// Provides a `SocketReceiver` implementation for `UdpSocket`
impl SocketReceiver for UdpSocket {
    /// Receive a single packet from UDP socket.
    fn receive_packet<'a>(
        &mut self,
        buffer: &'a mut [u8],
    ) -> std::io::Result<(&'a [u8], SocketAddr)> {
        self.recv_from(buffer)
            .map(move |(recv_len, address)| (&buffer[..recv_len], address))
    }
    /// Returns the socket address that this socket was created from.
    fn local_addr(&self) -> std::io::Result<SocketAddr> {
        self.local_addr()
    }
}

/// A reliable UDP socket implementation with configurable reliability and ordering guarantees.
#[derive(Debug)]
pub struct Socket {
    // Stores an instance of `SocketImpl` where `SocketSender` uses a `UdpSocket` (with `LinkConditioner`, if enabled) and SocketReceiver` is a `UdpSocket`.
    handler: SocketImpl<SocketWithConditioner, UdpSocket>,
    link_conditioner: Arc<Mutex<Option<LinkConditioner>>>,
}

impl Socket {
    /// Binds to the socket and then sets up `ActiveConnections` to manage the "connections".
    /// Because UDP connections are not persistent, we can only infer the status of the remote
    /// endpoint by looking to see if they are still sending packets or not
    pub fn bind<A: ToSocketAddrs>(addresses: A) -> Result<Self> {
        Self::bind_with_config(addresses, Config::default())
    }

    /// Bind to any local port on the system, if available
    pub fn bind_any() -> Result<Self> {
        Self::bind_any_with_config(Config::default())
    }

    /// Bind to any local port on the system, if available, with a given config
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
        socket.set_nonblocking(!config.blocking_mode)?;
        let link_conditioner = Arc::new(Mutex::new(Default::default()));
        Ok(Socket {
            handler: SocketImpl::new(
                SocketWithConditioner::new(
                    socket.try_clone().expect("Cannot clone a socket"),
                    link_conditioner.clone(),
                ),
                socket,
                config,
            ),
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

    /// Send a packet
    pub fn send(&mut self, packet: Packet) -> Result<()> {
        self.handler
            .event_sender()
            .send(packet)
            .expect("Receiver must exists.");
        Ok(())
    }

    /// Receive a packet
    pub fn recv(&mut self) -> Option<SocketEvent> {
        match self.handler.event_receiver().try_recv() {
            Ok(pkt) => Some(pkt),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => panic!["This can never happen"],
        }
    }

    /// Entry point to the run loop. This should run in a spawned thread since calls to `poll.poll`
    /// are blocking. We will default this to sleeping for 1ms.
    pub fn start_polling(&mut self) {
        self.start_polling_with_duration(Some(Duration::from_millis(1)))
    }

    /// Run the polling loop with a specified sleep duration. This should run in a spawned thread
    /// since calls to `poll.poll` are blocking.
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

    /// Process any inbound/outbound packets and handle idle clients
    pub fn manual_poll(&mut self, time: Instant) {
        self.handler.manual_poll(time);
    }

    /// Returns the local socket address
    pub fn local_addr(&self) -> Result<SocketAddr> {
        Ok(self.handler.local_addr()?)
    }

    /// Set the link conditioner for this socket. See [LinkConditioner] for further details.
    #[cfg(feature = "tester")]
    pub fn set_link_conditioner(&mut self, link_conditioner: Option<LinkConditioner>) {
        *self
            .link_conditioner
            .lock()
            .expect("Lock is already held by current thread.") = link_conditioner;
    }
}
