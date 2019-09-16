//! Note that the terms "client" and "server" here are purely what we logically associate with them.
//! Technically, they both work the same.
//! Note that in practice you don't want to implement a chat client using UDP.
use std::io::stdin;
use std::thread;
use std::time::Instant;

use laminar::{ErrorKind, Packet, Socket, SocketEventSender, managers::SimpleSocketManager, ConnectionEvent, ReceiveEvent, SendEvent};

const SERVER: &str = "127.0.0.1:12351";

fn server() -> Result<(), ErrorKind> {
    let mut socket = Socket::bind(SERVER, Box::new(SimpleSocketManager))?;
    let (sender, receiver) = (SocketEventSender(socket.get_event_sender()), socket.get_event_receiver());
    let _thread = thread::spawn(move || socket.start_polling());

    loop {
        if let Ok(ConnectionEvent(addr, event)) = receiver.recv() {

            match event {
                ReceiveEvent::Created => {
                    println!("Connection created {:?}", addr);
                },
                ReceiveEvent::Connected(data) => {
                    println!("Connected {:?} with message: {}",addr, String::from_utf8_lossy(data.as_ref()));
                    //sender.disconnect(addr);
                },
                ReceiveEvent::Packet(packet) => {
                    let msg = packet.payload();

                    if msg == b"Bye!" {
                        break;
                    }

                    let msg = String::from_utf8_lossy(msg);
                    let ip = packet.addr().ip();

                    println!("Received {:?} from {:?}", msg, ip);

                    sender
                        .send(Packet::reliable_unordered(
                            packet.addr(),
                            ["Echo: ".as_bytes(), msg.as_bytes()].concat(),
                        ))
                        .expect("This should send");
                },
                ReceiveEvent::Disconnected(reason) => {
                    println!("Disconnected {:?} reason: {:?}", addr, reason);
                },
                ReceiveEvent::Destroyed(reason) => {
                    println!("Connection destroyed {:?} reason: {:?}", addr, reason);
                }
            }
        }
    }

    Ok(())
}

fn client() -> Result<(), ErrorKind> {
    let addr = "127.0.0.1:12352";
    let mut socket = Socket::bind(addr, Box::new(SimpleSocketManager))?;
    println!("Connected on {}", addr);

    let server = SERVER.parse().unwrap();

    println!("Type a message and press Enter to send. Send `Bye!` to quit.");

    let stdin = stdin();
    let mut s_buffer = String::new();

    s_buffer.clear();
    stdin.read_line(&mut s_buffer)?;
    let line = s_buffer.replace(|x| x == '\n' || x == '\r', "");
    socket.send(ConnectionEvent(server, SendEvent::Connect(Box::from(line.as_bytes()))))?;

    loop {

        socket.manual_poll(Instant::now());

        if line == "Bye!" {
            break;
        }

        if let Some(ConnectionEvent(addr, event)) = socket.recv() {
            match event {
                ReceiveEvent::Created => {
                    println!("Connection created {:?}", addr);
                },
                ReceiveEvent::Connected(data) => {
                    println!("Connected {:?} with message: {}",addr, String::from_utf8_lossy(data.as_ref()));
                    socket.send(ConnectionEvent(server, SendEvent::Packet(Packet::reliable_unordered(server, "Copy that!".as_bytes().to_vec()))))?;
                },
                ReceiveEvent::Packet(packet) => {
                    let msg = packet.payload();

                    if msg == b"Bye!" {
                        break;
                    }

                    let msg = String::from_utf8_lossy(msg);
                    let ip = packet.addr().ip();

                    println!("Received {:?} from {:?}", msg, ip);
                    socket.send(ConnectionEvent(server, SendEvent::Disconnect))?;

                    // sender
                    //     .send(Packet::reliable_unordered(
                    //         packet.addr(),
                    //         "Copy that!".as_bytes().to_vec(),
                    //     ))
                    //     .expect("This should send");
                },
                ReceiveEvent::Disconnected(reason) => {
                    println!("Disconnected {:?} reason: {:?}", addr, reason);
                },
                ReceiveEvent::Destroyed(reason) => {
                    println!("Connection destroyed {:?} reason: {:?}", addr, reason);
                }
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), ErrorKind> {
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
