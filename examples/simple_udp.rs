//! This module provides an simple client, server examples with communication over udp.
//! 1. setting up server to receive data.
//! 2. setting up client to send data.
//! 3. serialize data to send and deserialize when received.
use bincode::{deserialize, serialize};
use laminar::{
    managers::SimpleConnectionManagerFactory, ConnectionEvent, Packet, ReceiveEvent, SendEvent,
    Socket,
};
use serde_derive::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::time::Instant;

/// The socket address of where the server is located.
const SERVER_ADDR: &str = "127.0.0.1:12345";
// The client address from where the data is sent.
const CLIENT_ADDR: &str = "127.0.0.1:12346";

fn client_address() -> SocketAddr {
    CLIENT_ADDR.parse().unwrap()
}

fn server_address() -> SocketAddr {
    SERVER_ADDR.parse().unwrap()
}

// helper function to reduce boiler plate
fn create_packet<T>(addr: SocketAddr, data: &T) -> ConnectionEvent<SendEvent>
where
    T: serde::Serialize,
{
    ConnectionEvent(
        addr,
        SendEvent::Packet(Packet::unreliable(addr, serialize(data).unwrap())),
    )
}

/// This will run an simple example with client and server communicating.
#[allow(unused_must_use)]
pub fn main() {
    let mut server = Socket::bind(
        server_address(),
        Box::new(SimpleConnectionManagerFactory(true)),
    )
    .unwrap();

    /*  setup or `Client` and send some test data. */
    let mut client = Socket::bind(
        client_address(),
        Box::new(SimpleConnectionManagerFactory(true)),
    )
    .unwrap();
    client.send(create_packet(
        server_address(),
        &DataType::Coords {
            latitude: 10.55454,
            longitude: 10.555,
            altitude: 1.3,
        },
    ));

    client.send(create_packet(
        server_address(),
        &DataType::Coords {
            latitude: 3.344,
            longitude: 5.4545,
            altitude: 1.33,
        },
    ));

    client.send(create_packet(
        server_address(),
        &DataType::Text {
            string: String::from("Some information"),
        },
    ));

    // Send the queued send operations
    client.manual_poll(Instant::now());

    // Check for any new packets
    server.manual_poll(Instant::now());

    // ==== results ====
    // Coords { longitude: 10.555, latitude: 10.55454, altitude: 1.3 }
    // Coords { longitude: 5.4545, latitude: 3.344, altitude: 1.33 }
    // Text { string: "Some information" }
    while let Some(pkt) = server.recv() {
        if let ConnectionEvent(_addr, ReceiveEvent::Packet(pkt)) = pkt {
            println!["{:?}", deserialize::<DataType>(pkt.payload()).unwrap()]
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
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
