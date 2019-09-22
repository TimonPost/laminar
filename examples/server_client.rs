//! Note that the terms "client" and "server" here are purely what we logically associate with them.
//! Technically, they both work the same.
//! Note that in practice you don't want to implement a chat client using UDP.
use std::io::stdin;
use std::thread;
use std::time::Instant;

use laminar::{
    managers::SimpleConnectionManagerFactory, ConnectionEvent, ErrorKind, Packet, ReceiveEvent,
    SendEvent, Socket,
};

const SERVER: &str = "127.0.0.1:12351";

fn server() -> Result<(), ErrorKind> {
    // create socket manager, that will use SimpleConnectionManager, that actually initiates connection by exchanging methods
    let mut socket = Socket::bind(SERVER, Box::new(SimpleConnectionManagerFactory(false)))?;
    let (sender, receiver) = (socket.get_event_sender(), socket.get_event_receiver());
    let _thread = thread::spawn(move || socket.start_polling());

    loop {
        if let Ok(ConnectionEvent(addr, event)) = receiver.recv() {
            match event {
                ReceiveEvent::Connected(data) => {
                    println!(
                        "{:?} -> Connected msg:{}",
                        addr,
                        String::from_utf8_lossy(data.as_ref())
                    );
                }
                ReceiveEvent::Packet(packet) => {
                    let msg = packet.payload();
                    if msg == b"Bye!" {
                        break;
                    }
                    let msg = String::from_utf8_lossy(msg);
                    println!("{:?} -> Packet msg:{}", addr, msg);

                    sender
                        .send(ConnectionEvent(
                            packet.addr(),
                            SendEvent::Packet(Packet::reliable_unordered(
                                packet.addr(),
                                [b"Echo: ", msg.as_bytes()].concat(),
                            )),
                        ))
                        .expect("This should send");
                }
                _ => println!("{:?} -> {:?}", addr, event),
            }
        }
    }

    Ok(())
}

fn client() -> Result<(), ErrorKind> {
    let addr = "127.0.0.1:12352";
    let mut socket = Socket::bind(addr, Box::new(SimpleConnectionManagerFactory(false)))?;
    println!("Connected on {}", addr);

    let sender = socket.get_event_sender();
    let _thread = thread::spawn(move || loop {
        socket.manual_poll(Instant::now());

        if let Some(ConnectionEvent(addr, event)) = socket.recv() {
            match event {
                ReceiveEvent::Connected(data) => {
                    println!(
                        "{:?} -> Connected msg:{}",
                        addr,
                        String::from_utf8_lossy(data.as_ref())
                    );
                }
                ReceiveEvent::Packet(packet) => {
                    let msg = String::from_utf8_lossy(packet.payload());
                    println!("{:?} -> Packet msg:{}", addr, msg);
                }
                _ => println!("{:?} -> {:?}", addr, event),
            }
        }
    });

    let stdin = stdin();
    let mut s_buffer = String::new();
    s_buffer.clear();

    let server = SERVER.parse().unwrap();
    println!("Type a `:c<message>` to connect");
    println!("Type a `<message>` to send a packet");
    println!("Type a `:d` to disconnect");
    println!("Type a `:q` to quit.");

    loop {
        stdin.read_line(&mut s_buffer)?;
        let line = s_buffer.replace(|x| x == '\n' || x == '\r', "");
        if line == ":q" {
            break;
        } else if line.starts_with(":c") {
            sender
                .send(ConnectionEvent(
                    server,
                    SendEvent::Connect(Box::from(line.split_at(2).1.as_bytes())),
                ))
                .expect("sending should not fail");
        } else if line == ":d" {
            sender
                .send(ConnectionEvent(server, SendEvent::Disconnect))
                .expect("sending should not fail");
        } else {
            sender
                .send(ConnectionEvent(
                    server,
                    SendEvent::Packet(Packet::reliable_unordered(server, line.into_bytes())),
                ))
                .expect("sending should not fail");
        }
        s_buffer.clear();
    }

    Ok(())
}

fn main() -> Result<(), ErrorKind> {
    let stdin = stdin();

    println!("Please type in `server` or `client`.");

    let mut s = String::new();
    stdin.read_line(&mut s)?;

    if s.starts_with('s') {
        println!("Starting server..");
        server()
    } else {
        println!("Starting client..");
        client()
    }
}
