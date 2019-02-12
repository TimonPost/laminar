//! Note that the terms "client" and "server" here are purely what we logically associate with them.
//! Technically, they both work the same.
//! Note that in practice you don't want to implement a chat client using UDP.
use std::io::stdin;

use laminar::{Config, NetworkError, Packet, Socket, SocketEvent};
use std::thread;

const SERVER: &str = "127.0.0.1:12351";

fn server() -> Result<(), NetworkError> {
    let (mut socket, _, event_receiver) = Socket::bind(SERVER, Config::default())?;
    let _thread = thread::spawn(move || socket.start_polling());

    println!("Listening for connections to {}", SERVER);

    loop {
        println!("[SERVER] receiving event");
        match event_receiver.recv().expect("Should get a message") {
            SocketEvent::Packet(packet) => {
                let msg = packet.payload();

                if msg == b"Bye!" {
                    break;
                }

                let msg = String::from_utf8_lossy(msg);
                let ip = packet.addr().ip();

                println!("Received {:?} from {:?}", msg, ip);
            }
            SocketEvent::Timeout(address) => {
                println!("Client timed out: {}", address);
            }
            _ => println!("Event not recognized"),
        }
    }

    Ok(())
}

fn client() -> Result<(), NetworkError> {
    let addr = "127.0.0.1:12352";
    let (mut socket, packet_sender, _) = Socket::bind(addr, Config::default())?;
    let _thread = thread::spawn(move || socket.start_polling());
    println!("Connected on {}", addr);

    let server = SERVER.parse().unwrap();

    loop {
        println!("[CLIENT] Sending 100 packets");
        for _ in 0..100 {
            packet_sender
                .send(Packet::reliable_unordered(
                    server,
                    vec![1, 2, 3, 41, 2, 3, 41, 2, 3, 41, 2, 3, 41, 2, 3, 41, 2, 3, 4],
                ))
                .unwrap();
        }
        thread::sleep(::std::time::Duration::from_millis(500));
        println!("[CLIENT] Sent 1000 packets");
    }
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
