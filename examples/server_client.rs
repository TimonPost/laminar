//! Note that the terms "client" and "server" here are purely what we logically associate with them.
//! Technically, they both work the same.
//! Note that in practice you don't want to implement a chat client using UDP.

extern crate laminar;

use std::io::stdin;

use laminar::{error::Result, NetworkConfig, Packet, UdpSocket};

const SERVER: &str = "localhost:12351";

fn server() -> Result<()> {
    let mut socket = UdpSocket::bind(SERVER, NetworkConfig::default())?;

    println!("Listening for connections to {}", SERVER);

    loop {
        match socket.recv()? {
            Some(packet) => {
                let msg = packet.payload();

                if msg == b"Bye!" {
                    break;
                }

                let msg = String::from_utf8_lossy(msg);
                let ip = packet.addr().ip();

                println!("Received {:?} from {:?}", msg, ip);

                socket.send(Packet::reliable_unordered(
                    packet.addr(),
                    "Copy that!".as_bytes().to_vec(),
                ))?;
            }
            None => {}
        }
    }

    Ok(())
}

fn client() -> Result<()> {
    let mut socket = UdpSocket::bind("localhost:12352", NetworkConfig::default())?;

    let server = SERVER.parse()?;

    println!("Type a message and press Enter to send. Send `Bye!` to quit.");

    let stdin = stdin();
    let mut s_buffer = String::new();

    loop {
        s_buffer.clear();
        stdin.read_line(&mut s_buffer)?;
        let line = s_buffer.replace(|x| x == '\n' || x == '\r', "");

        socket.send(Packet::reliable_unordered(
            server,
            line.clone().into_bytes(),
        ))?;

        if line == "Bye!" {
            break;
        }

        let back = socket.recv()?;

        match back {
            Some(packet) => {
                if packet.addr() == server {
                    println!("Server sent: {}", String::from_utf8_lossy(packet.payload()));
                } else {
                    println!("Unknown sender.");
                }
            }
            None => println!("Silence.."),
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let stdin = stdin();

    println!("Please type in `server` or `client`.");

    let mut s = String::new();
    stdin.read_line(&mut s)?;

    if s.starts_with("s") {
        println!("Starting server..");
        server()
    } else {
        println!("Starting client..");
        client()
    }
}
