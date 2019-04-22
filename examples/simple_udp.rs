//! This module provides an simple client, server examples with communication over udp.
//! 1. setting up server to receive data.
//! 2. setting up client to send data.
//! 3. serialize data to send and deserialize when received.
use bincode::{deserialize, serialize};
use crossbeam_channel::{Receiver, Sender};
use laminar::{ErrorKind, Packet, Socket, SocketEvent};
use serde_derive::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::{thread, time};

/// The socket address of where the server is located.
const SERVER_ADDR: &'static str = "127.0.0.1:12345";
// The client address from where the data is sent.
const CLIENT_ADDR: &'static str = "127.0.0.1:12346";

fn client_address() -> SocketAddr {
    CLIENT_ADDR.parse().unwrap()
}

fn server_address() -> SocketAddr {
    SERVER_ADDR.parse().unwrap()
}

/// This will run an simple example with client and server communicating.
#[allow(unused_must_use)]
pub fn main() {
    let mut server = Server::new();
    // set up or `Server` that will receive the messages we send with the `Client`
    let handle = thread::spawn(move || loop {
        server.receive();
    });

    thread::sleep(time::Duration::from_millis(100));

    /*  setup or `Client` and send some test data. */
    let mut client = Client::new();

    client.send(DataType::Coords {
        latitude: 10.55454,
        longitude: 10.555,
        altitude: 1.3,
    });

    client.send(DataType::Coords {
        latitude: 3.344,
        longitude: 5.4545,
        altitude: 1.33,
    });

    client.send(DataType::Text {
        string: String::from("Some information"),
    });

    // ==== results ====
    // Moving to lat: 10.555, long: 10.55454, alt: 1.3
    // Moving to lat: 5.4545, long: 3.344, alt: 1.33
    // Received text: "Some information"
    handle.join();
}

#[derive(Serialize, Deserialize)]
enum DataType {
    Coords {
        longitude: f32,
        latitude: f32,
        altitude: f32,
    },
    Text {
        string: String,
    },
}

/// This is an test server we use to receive data from clients.
struct Server {
    _packet_sender: Sender<Packet>,
    event_receiver: Receiver<SocketEvent>,
    _polling_thread: thread::JoinHandle<Result<(), ErrorKind>>,
}

impl Server {
    pub fn new() -> Self {
        // setup an udp socket and bind it to the client address.
        let (mut socket, packet_sender, event_receiver) = Socket::bind(server_address()).unwrap();
        let polling_thread = thread::spawn(move || socket.start_polling());
        Server {
            _packet_sender: packet_sender,
            event_receiver,
            _polling_thread: polling_thread,
        }
    }

    /// Receive and block the current thread.
    pub fn receive(&mut self) {
        // Next start receiving.
        let result = self.event_receiver.recv();

        match result {
            Ok(SocketEvent::Packet(packet)) => {
                let received_data: &[u8] = packet.payload();

                // deserialize bytes to `DataType` we passed in with `Client.send()`.
                let deserialized: DataType = deserialize(&received_data).unwrap();

                self.perform_action(deserialized);
            }
            Ok(SocketEvent::Timeout(address)) => {
                println!("A client timed out: {}", address);
            }
            Ok(_) => {}
            Err(e) => {
                println!("Something went wrong when receiving, error: {:?}", e);
            }
        }
    }

    /// Perform some processing of the data we have received.
    fn perform_action(&self, data_type: DataType) {
        match data_type {
            DataType::Coords {
                longitude,
                latitude,
                altitude,
            } => {
                println!(
                    "Moving to lat: {}, long: {}, alt: {}",
                    longitude, latitude, altitude
                );
            }
            DataType::Text { string } => {
                println!("Received text: {:?}", string);
            }
        }
    }
}

/// This is an test client to send data to the server.
struct Client {
    packet_sender: Sender<Packet>,
    _event_receiver: Receiver<SocketEvent>,
    _polling_thread: thread::JoinHandle<Result<(), ErrorKind>>,
}

impl Client {
    pub fn new() -> Self {
        // setup an udp socket and bind it to the client address.
        let (mut socket, packet_sender, event_receiver) = Socket::bind(client_address()).unwrap();
        let polling_thread = thread::spawn(move || socket.start_polling());

        Client {
            packet_sender,
            _event_receiver: event_receiver,
            _polling_thread: polling_thread,
        }
    }

    pub fn send(&mut self, data_type: DataType) {
        let serialized = serialize(&data_type);

        match serialized {
            Ok(raw_data) => {
                self.packet_sender
                    .send(Packet::reliable_unordered(server_address(), raw_data)).expect("Should be fine");
            }
            Err(e) => println!("Some error occurred: {:?}", e),
        }
    }
}
