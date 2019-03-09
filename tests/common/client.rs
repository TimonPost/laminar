use laminar::Config;
use laminar::Packet;
use laminar::Socket;
use std::net::SocketAddr;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

pub struct Client {
    pub timeout_sending: Duration,
    pub endpoint: SocketAddr,
    pub packets_to_send: u32,
}

impl Client {
    pub fn new(timeout_sending: Duration, endpoint: SocketAddr, packets_to_send: u32) -> Client {
        Client {
            timeout_sending,
            endpoint,
            packets_to_send,
        }
    }

    pub fn run_instance<F>(&self, create_packet: F) -> ClientHandle
    where
        F: Fn() -> Packet + Send + 'static,
    {
        let timeout = self.timeout_sending;
        let endpoint = self.endpoint;
        let packets_to_send = self.packets_to_send;

        let handle = thread::spawn(move || {
            let (mut socket, packet_sender, _) = Socket::bind(endpoint, Config::default()).unwrap();

            let _thread = thread::spawn(move || socket.start_polling());

            for _ in 0..packets_to_send {
                let packet = create_packet();
                if let Err(e) = packet_sender.send(packet) {
                    println!("Client can not send packet {:?}", e);
                }

                thread::sleep(timeout);
            }
        });

        ClientHandle::new(handle)
    }
}

pub struct ClientHandle {
    thread_handle: JoinHandle<()>,
}

impl ClientHandle {
    pub fn new(handle: JoinHandle<()>) -> ClientHandle {
        ClientHandle {
            thread_handle: handle,
        }
    }

    pub fn wait_until_finished(self) {
        self.thread_handle.join().unwrap();
    }
}
