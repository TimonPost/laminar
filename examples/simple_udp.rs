extern crate laminar;
extern crate bincode;

use self::laminar::net::{UdpSocket, SocketAddr};
use self::laminar::packet::Packet;

use self::bincode::{ serialize, deserialize };
use super::{client_address, server_address};
use std::{time, thread};

use std::time::Instant;

/// This will run an simple example with sending and receiving data.
///
/// We cover:
/// 1. setting up server to receive data.
/// 2. setting up client to send data.
/// 3. serialize data to send and deserialize when received.
pub fn run_simple_example()
{
    // set up or `Server` that will receive the messages we send with the `Client`
    let handle = thread::spawn(|| {
        loop {
            let mut server = Server::new();
            server.receive();
        }
    });

    thread::sleep(time::Duration::from_millis(100));

    /*  setup or `Client` and send some test data. */
    let mut client = Client::new();

    let now = Instant::now();
    client.send(DataType::Coords { latitude: 10.55454, longitude: 10.555, altitude: 1.3});
    println!(" ==== Message took {:?} to send ====", now.elapsed());

    let now = Instant::now();
    client.send(DataType::Coords { latitude: 3.344, longitude: 5.4545, altitude: 1.33});
    println!("==== Message took {:?} to send ====", now.elapsed());

    let now = Instant::now();
    client.send(DataType::Text { string: String::from("Some information") });
    println!("==== Message took {:?} to send ====", now.elapsed());

    /// ==== result ====
    // Moving to lat: 10.555, long: 10.55454, alt: 1.3
    // Moving to lat: 5.4545, long: 3.344, alt: 1.33
    // Received text: "Some information"

    handle.join();
}

#[derive(Serialize, Deserialize)]
enum DataType {
    Coords { longitude: f32, latitude: f32, altitude: f32 },
    Text { string: String },
}

/// This is an test server we use to receive data from clients.
struct Server
{
    udp_socket: UdpSocket
}

impl Server
{
    pub fn new() -> Self
    {
        // setup an udp socket and bind it to the client address.
        let mut udp_socket: UdpSocket = UdpSocket::bind(server_address()).unwrap();

        // next we could specify if or socket should block the current thread when receiving data or not (default = true)
        udp_socket.set_blocking(true);

        Server { udp_socket }
    }

    /// Receive and block the current thread.
    pub fn receive(&mut self)
    {
        // Next start receiving.
        let result= self.udp_socket.recv();

        match result {
            Ok(Some(packet)) => {
                let endpoint: SocketAddr = packet.addr;
                let received_data: Box<[u8]> = packet.payload;

                // deserialize bytes to `DataType` we passed in with `Client.send()`.
                let deserialized: DataType = deserialize(&received_data).unwrap();

                self.perform_action(deserialized);
            },
            Ok(None) => {
                println!("This could happen when we have'n received all data from this packet yet");
            },
            Err(e) => {
                println!("Something went wrong when receiving, error: {:?}", e);
            }
        }
    }

    /// Perform some processing of the data we have received.
    fn perform_action(&self, data_type: DataType)
    {
        match data_type {
            DataType::Coords { longitude, latitude, altitude } => {
                println!("Moving to lat: {}, long: {}, alt: {}", longitude, latitude, altitude);
            },
            DataType::Text { string } => {
                println!("Received text: {:?}", string);
            }
        }
    }
}

/// This is an test client to send data to the server.
struct Client
{
    udp_socket: UdpSocket
}

impl Client
{
    pub fn new() -> Self
    {
        // setup an udp socket and bind it to the client address.
        let mut udp_socket = UdpSocket::bind(client_address()).unwrap();

        // next we could specify if or socket should block the current thread when receiving data or not (default = true)
        udp_socket.set_blocking(true);

        Client { udp_socket }
    }

    pub fn send(&mut self, data_type: DataType)
    {
        let serialized = serialize(&data_type);

        match serialized {
            Ok(raw_data) => {
                self.udp_socket.send(Packet::new(server_address(), raw_data));
            },
            Err(e) => println!("Some error occurred: {:?}", e)
        }
    }
}

