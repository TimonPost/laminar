use std::net::SocketAddr;
use std::thread::{self, JoinHandle};
use std::time::Duration;
use std::time::Instant;

use log::info;

use laminar::{Packet, Socket};

/// Represents a client to some endpoint.
pub struct Client {
    /// The sending timeout
    pub sending_timeout: Duration,
    /// The number of packets to send
    pub packets_to_send: u32,
}

impl Client {
    /// Constructs a new `Client`.
    pub fn new(timeout_sending: Duration, packets_to_send: u32) -> Client {
        Client {
            sending_timeout: timeout_sending,
            packets_to_send,
        }
    }

    /// Runs a specific instance of the client running at the given socket address.
    /// This function takes in a closure who constructs a packet which will be sent out to the client.
    pub fn run_instance<F>(&self, create_packet: F, endpoint: SocketAddr) -> ClientHandle
    where
        F: Fn() -> Packet + Send + 'static,
    {
        let timeout = self.sending_timeout;
        let packets_to_send = self.packets_to_send;

        let handle = thread::spawn(move || {
            let mut socket = Socket::bind(endpoint).unwrap();

            info!("Client {:?} starts to send packets.", endpoint);

            for _ in 0..packets_to_send {
                let packet = create_packet();
                socket.send(packet).unwrap();
                socket.manual_poll(Instant::now());

                let beginning_park = Instant::now();
                let mut timeout_remaining = timeout;
                loop {
                    thread::park_timeout(timeout_remaining);
                    let elapsed = beginning_park.elapsed();
                    if elapsed >= timeout {
                        break;
                    }
                    timeout_remaining = timeout - elapsed;
                }
            }
            info!("Client {:?} sent all messages.", endpoint);
        });

        ClientHandle::new(handle)
    }
}

/// This is a handle to a running client which is sending data to some endpoint.
pub struct ClientHandle {
    thread_handle: JoinHandle<()>,
}

impl ClientHandle {
    /// Constructs a new `ClientHandle` by the given thread handle.
    pub fn new(handle: JoinHandle<()>) -> ClientHandle {
        ClientHandle {
            thread_handle: handle,
        }
    }

    /// Waits until the client has sent all of its packets.
    pub fn wait_until_finished(self) {
        self.thread_handle.join().unwrap();
    }
}
