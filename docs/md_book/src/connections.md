# Connections

As mentioned in [Protocols](protocols.md) laminar is built on top of UDP. UDP is a [connectionless protocol](https://en.wikipedia.org/wiki/Connectionless_communication), but for multiplayer games we very often want to connect to a particular endpoint and repeatedly send data back and forth.

Laminar itself needs to maintain a small amount of data for each endpoint it is communicating with. For example, Laminar maintains data about which sent packets the other endpoint has acknowledged as well as an estimated [Rount Trip Time](congestion_avoidence/rtt.md).

In order to support these common use-cases Laminar adds an extremely simple connection model on top of UDP.

Connections are considered established whenever we have both sent and received data to the same endpoint. This means, to establish a connection your server needs to respond to inbound messages it receives. A sample of this is presented

**client**

```rust
let socket = Socket::bind(SERVER_ADDRESS);
let (sender, receiver) = (socket.get_packet_sender(), socket.get_event_receiver());
thread::spawn(move || socket.start_polling());

sender.send(Packet::reliable_unordered(SERVER_ADDRESS, "Ping".as_bytes().to_vec()));

loop {
    if let Ok(event) = receiver.recv() {
        match event {
            SocketEvent::Connect(addr) => {
                println!("Connected to: {}", addr);
            },
            _
        }
    }
}
```

**server**

```rust
let socket = Socket::bind(SERVER_ADDRESS);
let (sender, receiver) = (socket.get_packet_sender(), socket.get_event_receiver());
thread::spawn(move || socket.start_polling());

loop {
    if let Ok(event) = receiver.recv() {
        match event {
            SocketEvent::Packet(packet) => {
                if packet.payload() == b"Ping" {
                    sender.send(Packet::reliable_unordered(
                        packet.addr(),
                        "Pong".as_bytes().to_vec(),
                    )).unwrap();
                }
            },
            SocketEvent::Connect(addr) => {
                println!("Connected to: {}", addr);
            },
            _
        }
    }
}
```

If we don't send the `Pong` in the server, then neither the client nor the server will display the "Connected to" message.

### Packet Flooding Mitigation

Laminar will optimistically track data for endpoints before connections are established. As soon as data is sent or received from a new endpoint Laminar will start tracking the endpoint. In order to prevent packet flooding attacks from causing Laminar to allocate too much memory, the number of unestablished connections that Laminar will optimistically track can be controlled with the `max_unestablished_connections` Config.
