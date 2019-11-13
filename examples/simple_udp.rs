//! This module provides an simple client, server examples with communication over udp.
//! 1. setting up server to receive data.
//! 2. setting up client to send data.
//! 3. serialize data to send and deserialize when received.
use std::net::SocketAddr;
use std::time::Instant;

use bincode::{deserialize, serialize};
use serde_derive::{Deserialize, Serialize};

use laminar::{Packet, Socket, SocketEvent};

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
    let mut server = Socket::bind(server_address()).unwrap();

    /*  setup our `Client` and send some test data. */
    let mut client = Socket::bind(client_address()).unwrap();

    client.send(Packet::unreliable(
        server_address(),
        serialize(&DataType::Coords {
            latitude: 10.55454,
            longitude: 10.555,
            altitude: 1.3,
        })
        .unwrap(),
    ));

    client.send(Packet::unreliable(
        server_address(),
        serialize(&DataType::Coords {
            latitude: 3.344,
            longitude: 5.4545,
            altitude: 1.33,
        })
        .unwrap(),
    ));

    client.send(Packet::unreliable(
        server_address(),
        serialize(&DataType::Text {
            string: String::from("Some information"),
        })
        .unwrap(),
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
        match pkt {
            SocketEvent::Packet(pkt) => {
                println!["{:?}", deserialize::<DataType>(pkt.payload()).unwrap()]
            }
            _ => {}
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
