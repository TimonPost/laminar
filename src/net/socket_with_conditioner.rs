use super::{DatagramSocket, LinkConditioner};
use crate::error::Result;
use std::net::{SocketAddr, UdpSocket};

// Wrap `LinkConditioner` and `UdpSocket` together. LinkConditioner is enabled when building with a "tester" feature.
#[derive(Debug)]
pub struct SocketWithConditioner {
    is_blocking_mode: bool,
    socket: UdpSocket,
    link_conditioner: Option<LinkConditioner>,
}

impl SocketWithConditioner {
    pub fn new(socket: UdpSocket, is_blocking_mode: bool) -> Result<Self> {
        socket.set_nonblocking(!is_blocking_mode)?;
        Ok(SocketWithConditioner {
            is_blocking_mode,
            socket,
            link_conditioner: None,
        })
    }

    #[cfg(feature = "tester")]
    pub fn set_link_conditioner(&mut self, link_conditioner: Option<LinkConditioner>) {
        self.link_conditioner = link_conditioner;
    }
}

/// Provides a `DatagramSocket` implementation for `SocketWithConditioner`.
impl DatagramSocket for SocketWithConditioner {
    // Determinates whether packet will be sent or not based on `LinkConditioner` if enabled.
    fn send_packet(&mut self, addr: &SocketAddr, payload: &[u8]) -> std::io::Result<usize> {
        if cfg!(feature = "tester") {
            if let Some(ref mut link) = &mut self.link_conditioner {
                if !link.should_send() {
                    return Ok(0);
                }
            }
        }
        self.socket.send_to(payload, addr)
    }

    /// Receives a single packet from UDP socket.
    fn receive_packet<'a>(
        &mut self,
        buffer: &'a mut [u8],
    ) -> std::io::Result<(&'a [u8], SocketAddr)> {
        self.socket
            .recv_from(buffer)
            .map(move |(recv_len, address)| (&buffer[..recv_len], address))
    }

    /// Returns the socket address that this socket was created from.
    fn local_addr(&self) -> std::io::Result<SocketAddr> {
        self.socket.local_addr()
    }

    /// Returns whether socket operates in blocking or non-blocking mode.
    fn is_blocking_mode(&self) -> bool {
        self.is_blocking_mode
    }
}
