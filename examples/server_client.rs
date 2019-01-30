//! Note that the terms "client" and "server" here are purely what we logically associate with them.
//! Technically, they both work the same.
//! Note that in practice you don't want to implement a chat client using UDP.
use std::io::stdin;

use laminar::{
    config::NetworkConfig,
    error::NetworkError,
    net::{LaminarSocket, SocketEvent},
    Packet,
};
use std::thread;

const SERVER: &str = "127.0.0.1:12351";

fn server() -> Result<(), NetworkError> {
    let (mut socket, packet_sender, event_receiver) =
        LaminarSocket::bind(SERVER, NetworkConfig::default())?;
    let _thread = thread::spawn(move || socket.start_polling());

    println!("Listening for connections to {}", SERVER);

    loop {
        match event_receiver.recv().expect("Should get a message") {
            SocketEvent::Packet(packet) => {
                let msg = packet.payload();

                if msg == b"Bye!" {
                    break;
                }

                let msg = String::from_utf8_lossy(msg);
                let ip = packet.addr().ip();

                println!("Received {:?} from {:?}", msg, ip);

                packet_sender
                    .send(Packet::reliable_unordered(
                        packet.addr(),
                        "Copy that!".as_bytes().to_vec(),
                    ))
                    .unwrap();
            }
            SocketEvent::TimeOut(address) => {
                println!("Client timed out: {}", address);
            }
            _ => {}
        }
    }

    Ok(())
}

fn client() -> Result<(), NetworkError> {
    let addr = "127.0.0.1:12352";
    let (mut socket, packet_sender, event_receiver) =
        LaminarSocket::bind(addr, NetworkConfig::default())?;
    println!("Connected on {}", addr);
    let _thread = thread::spawn(move || socket.start_polling());

    let server = SERVER.parse().unwrap();

    println!("Type a message and press Enter to send. Send `Bye!` to quit.");

    let stdin = stdin();
    let mut s_buffer = String::new();

    loop {
        s_buffer.clear();
        stdin.read_line(&mut s_buffer)?;
        let line = s_buffer.replace(|x| x == '\n' || x == '\r', "");

        packet_sender
            .send(Packet::reliable_unordered(
                server,
                line.clone().into_bytes(),
            ))
            .unwrap();

        if line == "Bye!" {
            break;
        }

        match event_receiver.recv().unwrap() {
            SocketEvent::Packet(packet) => {
                if packet.addr() == server {
                    println!("Server sent: {}", String::from_utf8_lossy(packet.payload()));
                } else {
                    println!("Unknown sender.");
                }
            }
            SocketEvent::TimeOut(_) => {}
            _ => println!("Silence.."),
        }
    }

    Ok(())
}

fn main() -> Result<(), NetworkError> {
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
