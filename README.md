# Laminar

[![Build Status][s2]][l2] [![Latest Version][s1]][l1] [![docs.rs][s4]][l4] [![Join us on Discord][s5]][l5] [![MIT/Apache][s3]][l3] ![Lines of Code][s6] ![Coverage][s7]

[s1]: https://img.shields.io/crates/v/laminar.svg
[l1]: https://crates.io/crates/laminar
[s2]: https://jenkins.amethyst-engine.org/buildStatus/icon?job=laminar%2Fmaster
[l2]: https://jenkins.amethyst-engine.org/job/laminar/job/master/badge/icon
[s3]: https://img.shields.io/badge/license-MIT%2FApache-blue.svg
[l3]: docs/LICENSE-MIT
[s4]: https://docs.rs/laminar/badge.svg
[l4]: https://docs.rs/laminar/
[s5]: https://img.shields.io/discord/425678876929163284.svg?logo=discord
[l5]: https://discord.gg/GnP5Whs
[s6]: https://tokei.rs/b1/github/amethyst/laminar?category=code
[s7]: https://codecov.io/gh/amethyst/laminar/branch/master/graphs/badge.svg

Laminar is an application-level transport protocol which provides configurable reliability and ordering guarantees built on top of UDP. 
It focuses on fast-paced fps-games and provides a lightweight, message-based interface.

Laminar was designed to be used within the [Amethyst][amethyst] game engine but is usable without it.

If you are new to laminar or networking in general, We strongly recommend taking a look at the [laminar book][book]

[amethyst]: https://github.com/amethyst/amethyst

# Concepts

This library is loosely based off of [Gaffer on Games][gog] and shares features similar as RakNet, Steam Socket, netcode.io.
The idea is to provide an in rust written, low-level UDP-protocol which supports the use of cases of video games that require multiplayer features.
The library itself provides a few low-level types of packets that provide different types of guarantees. The most
basic are unreliable and reliable packets. Also ordering, sequencing can be done on multiple streams.
For more information, read the projects [README.md][readme], [book][book], [docs][docs] or [examples][examples].

[gog]: https://gafferongames.com/
[readme]: https://github.com/amethyst/laminar/blob/master/README.md
[book]: https://amethyst.github.io/laminar/docs/index.html
[docs]: https://docs.rs/laminar/
[examples]: https://github.com/amethyst/laminar/tree/master/examples
[amethyst]: https://github.com/amethyst/amethyst

## Table of contents:
- [Useful links](#useful-links)
- [Features](#features)
- [Getting Started](#getting-stated)
- [Examples](#examples)
- [Notice](#notice)
- [Contributing](#contribution)
- [Authors](#authors)
- [License](#license)

## Features
These are the features this crate provides:

* [x] Fragmentation
* [x] Unreliable packets
* [x] Unreliable sequenced packets
* [x] Reliable unordered packets
* [x] Reliable ordered packets
* [x] Reliable sequenced packets
* [x] Rtt estimations
* [x] Protocol version monitoring
* [x] Basic connection management
* [x] Heartbeat
* [x] Basic DoS mitigation
* [x] High Timing control
* [x] Protocol Versioning
* [x] Well-tested by integration and unit tests
* [x] Can be used by multiple threads (Sender, Receiver)

### Planned

* [ ] Handshake Protocol
* [ ] Advanced Connection Management
* [ ] Cryptography
* [ ] Congestion Control

## Getting Stated
Add the laminar package to your `Cargo.toml` file.

```toml
[dependencies]
laminar = "0.3"
```

### Useful Links

- [Documentation](https://docs.rs/laminar/).
- [Crates.io](https://crates.io/crates/laminar)
- [Examples](https://github.com/amethyst/laminar/tree/master/examples)
- [Contributing](https://github.com/amethyst/laminar/blob/master/docs/CONTRIBUTING)
- [Book](https://amethyst.github.io/laminar/docs/index.html)

## Examples
Please check out our [examples](https://github.com/amethyst/laminar/tree/master/examples) for more information.

### UDP API | [see more](https://github.com/amethyst/laminar/blob/master/examples/udp.rs)
This is an example of how to use the UDP API.

_Send packets_

```rust
use laminar::{Socket, Packet};

// Creates the socket
let mut socket = Socket::bind("127.0.0.1:12345")?;
let packet_sender = socket.get_packet_sender();
// Starts the socket, which will start a poll mechanism to receive and send messages.
let _thread = thread::spawn(move || socket.start_polling());

// Bytes to sent
let bytes = vec![...];

// Creates packets with different reliabilities
let unreliable = Packet::unreliable(destination, bytes);
let reliable = Packet::reliable_unordered(destination, bytes);

// Specifies on which stream and how to order our packets, checkout our book and documentation for more information
let unreliable = Packet::unreliable_sequenced(destination, bytes, Some(1));
let reliable_sequenced = Packet::reliable_sequenced(destination, bytes, Some(2));
let reliable_ordered = Packet::reliable_ordered(destination, bytes, Some(3));

// Sends the created packets
packet_sender.send(unreliable_sequenced).unwrap();
packet_sender.send(reliable).unwrap();
packet_sender.send(unreliable_sequenced).unwrap();
packet_sender.send(reliable_sequenced).unwrap();
packet_sender.send(reliable_ordered).unwrap();
```

_Receive Packets_
```rust
use laminar::{SocketEvent, Socket};

// Creates the socket
let socket = Socket::bind("127.0.0.1:12346")?;
let event_receiver = socket.get_event_receiver();
// Starts the socket, which will start a poll mechanism to receive and send messages.
let _thread = thread::spawn(move || socket.start_polling());

// Waits until a socket event occurs
let result = event_receiver.recv();

match result {
    Ok(socket_event) => {
        match  socket_event {
            SocketEvent::Packet(packet) => {
                let endpoint: SocketAddr = packet.addr();
                let received_data: &[u8] = packet.payload();
            },
            SocketEvent::Connect(connect_event) => { /* a client connected */ },
            SocketEvent::Timeout(timeout_event) => { /* a client timed out */},
        }
    }
    Err(e) => {
        println!("Something went wrong when receiving, error: {:?}", e);
    }
}
```

## Authors

- [Lucio Franco](https://github.com/LucioFranco)
- [Fletcher Haynes](https://github.com/fhaynes)
- [Timon Post](https://github.com/TimonPost)
- [Justin LeFebvre](https://github.com/jstnlef) 

## Note

This library is not fully stable yet, and there may be breaking changes to the API.
For more advanced examples of using laminar, you can check out the [Amethyst-Network](https://github.com/amethyst/amethyst/tree/master/amethyst_network) crate.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.

## License

Licensed under either of
 * Apache License, Version 2.0 ([LICENSE-APACHE](docs/LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](docs/LICENSE-MIT) or http://opensource.org/licenses/MIT)
at your option.
