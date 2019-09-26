use super::LinkConditioner;
use crate::{
    error::Result,
    net::{SocketReceiver, SocketSender},
};

use std::{
    cell::RefCell,
    collections::hash_map::Entry,
    collections::{HashMap, VecDeque},
    net::SocketAddr,
    rc::Rc,
};

/// This type allows to share global state between all sockets, created from the same instance of `NetworkEmulator`.
type GlobalBindings = Rc<RefCell<HashMap<SocketAddr, VecDeque<(SocketAddr, Vec<u8>)>>>>;

/// Enables to create the emulated socket, that share global state stored by this network emulator.
#[derive(Debug, Default)]
pub struct NetworkEmulator {
    network: GlobalBindings,
}

impl NetworkEmulator {
    /// Created an emulated socket by binding to a address.
    /// If other socket already was bound to this address, error will be returned instead.
    pub fn new_socket(&self, address: SocketAddr) -> Result<EmulatedSocket> {
        match self.network.borrow_mut().entry(address) {
            Entry::Occupied(_) => Err(std::io::Error::new(
                std::io::ErrorKind::AddrInUse,
                "Cannot bind to address",
            )
            .into()),
            Entry::Vacant(entry) => {
                entry.insert(Default::default());
                Ok(EmulatedSocket {
                    network: self.network.clone(),
                    address,
                    conditioner: None,
                })
            }
        }
    }

    /// Clear all packets from a socket that is bound to provided address.
    pub fn clear_packets(&self, addr: SocketAddr) {
        if let Some(packets) = self.network.borrow_mut().get_mut(&addr) {
            packets.clear();
        }
    }
}

/// Implementation of a socket, that is created by `NetworkEmulator`.
#[derive(Debug, Clone)]
pub struct EmulatedSocket {
    network: GlobalBindings,
    address: SocketAddr,
    conditioner: Option<LinkConditioner>,
}

impl EmulatedSocket {
    /// Set the link conditioner for this socket. See [LinkConditioner] for further details.
    pub fn set_link_conditioner(&mut self, conditioner: Option<LinkConditioner>) {
        self.conditioner = conditioner;
    }
}

impl SocketSender for EmulatedSocket {
    /// Sends a packet to and address if there is a socket bound to it. Otherwise it will simply be ignored.
    fn send_packet(&mut self, addr: &SocketAddr, payload: &[u8]) -> Result<usize> {
        let send = if let Some(conditioner) = &mut self.conditioner {
            conditioner.should_send()
        } else {
            true
        };
        if send {
            if let Some(binded) = self.network.borrow_mut().get_mut(addr) {
                binded.push_back((self.address, payload.to_vec()));
            }
            Ok(payload.len())
        } else {
            Ok(0)
        }
    }
}

impl SocketReceiver for EmulatedSocket {
    /// Receive a packet from this socket.
    fn receive_packet<'a>(
        &mut self,
        buffer: &'a mut [u8],
    ) -> Result<Option<(&'a [u8], SocketAddr)>> {
        Ok(
            if let Some((addr, payload)) = self
                .network
                .borrow_mut()
                .get_mut(&self.address)
                .unwrap()
                .pop_front()
            {
                let slice = &mut buffer[..payload.len()];
                slice.copy_from_slice(payload.as_ref());
                Some((slice, addr))
            } else {
                None
            },
        )
    }
    /// Returns the socket address that this socket was created from.
    fn local_addr(&self) -> Result<SocketAddr> {
        Ok(self.address)
    }
}
