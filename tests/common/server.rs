use crate::common::ClientStub;
use laminar::SocketEvent;
use crate::common::ThroughputMonitoring;
use laminar::Socket;
use crossbeam_channel::Sender;
use crossbeam_channel::Receiver;
use crate::common::PacketFactory;
use crate::common::PacketAsserting;
use laminar::Config;
use crate::common::server_addr;
use std::time::Duration;
use std::thread;
use laminar::DeliveryMethod;
use crate::common::Ordering;
use crate::common::Sequencing;
use std::collections::HashMap;
use std::thread::JoinHandle;
use laminar::Packet;

pub struct Server {
    throughput_monitor: ThroughputMonitoring,
    socket: Socket,
    packet_sender: Sender<Packet>,
    packet_receiver: Receiver<SocketEvent>
}

impl Server {
    pub fn new() -> Server {
        let (mut socket, packet_sender, packet_receiver) = Socket::bind(server_addr(), Config::default()).unwrap();

        Server {
            throughput_monitor: ThroughputMonitoring::new(Duration::from_secs(1)),
            socket,
            packet_receiver,
            packet_sender
        }
    }

    pub fn start_receiving(mut self, mut packet_asserting: Box<PacketAsserting + Send + Sync>) -> ServerHandle {
        let mut socket = self.socket;
        let mut throughput_monitor = self.throughput_monitor;
        let packet_receiver = self.packet_receiver;

        let serve_handle = thread::spawn(move || {
            let handle = thread::spawn(move || socket.start_polling());

            loop {
                let packet_receiver = packet_receiver.recv().unwrap();

                match packet_receiver {
                    SocketEvent::Packet(p) => {
                        packet_asserting.assert_packet(p);

                        throughput_monitor.tick();
                    },
                    SocketEvent::Connect(c) => println!("New connection {:?}", c),
                    SocketEvent::Disconnect(d) => println!("Connection disconnected {:?}", d),
                    SocketEvent::Timeout(t) => println!("Timeout on connection: {:?}", t),
                }

                println!("average p/s {}", throughput_monitor.average());
            }
        });

        ServerHandle::new(serve_handle)
    }
}

pub struct ServerHandle {
    server_handle: JoinHandle<()>,
}

impl ServerHandle {

    pub fn new(server_handle: JoinHandle<()>) -> ServerHandle {
        ServerHandle {
            server_handle
        }
    }

    pub fn spawn_client(&self, client: ClientStub, packet_factory:  Box<PacketFactory + Send + Sync>) {
        thread::spawn(move || {
            let (mut socket, packet_sender, _) = Socket::bind(client.endpoint, Config::default()).unwrap();

            for _ in 0..client.packets_to_send {
                let packet = packet_factory.new_packet();
                let send_result = packet_sender.send(packet);
            }
        });
    }

    pub fn wait_until_finished(self) {
        self.server_handle.join();
    }
}