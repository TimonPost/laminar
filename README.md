# Laminar

[![Latest Version][s1]][l1] [![Build Status][s2]][l2] [![MIT/Apache][s3]][l3] [![docs.rs][s4]][l4] [![Join us on Discord][s5]][l5] ![Lines of Code][s6]

[s1]: https://img.shields.io/crates/v/laminar.svg
[l1]: https://crates.io/crates/laminar
[s2]: https://travis-ci.org/amethyst/laminar.svg?branch=master
[l2]: https://travis-ci.org/amethyst/laminar
[s3]: https://img.shields.io/badge/license-MIT%2FApache-blue.svg
[l3]: ./LICENSE
[s4]: https://docs.rs/laminar/badge.svg
[l4]: https://docs.rs/laminar/
[s5]: https://img.shields.io/discord/425678876929163284.svg?logo=discord
[l5]: https://discord.gg/GnP5Whs
[s6]: https://tokei.rs/b1/github/amethyst/laminar?category=code


A UDP-based protocol that provides partial reliability. Coming soon!

## Table of contents:
- [Useful links](https://github.com/amethyst/laminar#useful-links)
- [Features](https://github.com/amethyst/laminar#features)
- [Examples](https://github.com/amethyst/laminar#examples)
    - [Udp](https://github.com/amethyst/laminar#udp)
- [Notice](https://github.com/amethyst/laminar#notice)
- [Contributing](https://github.com/amethyst/laminar#contributing)
- [Authors](https://github.com/amethyst/laminar/#authors)
- [License](#license)

Add the laminar package to your `Cargo.toml` file.

```toml
[dependencies]
laminar = "0.0.0"
```
And import the laminar modules you want to use.

```rust
extern crate laminar;

// this module contains all socket related logic.
use laminar::net::{UdpSocket, SocketAddr, NetworkConfig, Connection, Quality, TcpSocket, TcpClient, TcpSocketState};
// this module contains packet related logic.
use laminar::packet::{Packet};
```

## Useful Links

- [Documentation](https://docs.rs/laminar/).
- [Cargo Page](https://crates.io/crates/laminar)
- [Examples](https://github.com/amethyst/laminar/tree/master/examples)

## Features
These are the features from this crate:

- Semi-reliable UDP
- UDP
- TCP

## Examples
These are some basic examples demonstrating how to use this crate. See [examples](https://github.com/amethyst/laminar/tree/master/examples) for more.

### Udp API | [see more](https://github.com/amethyst/laminar/blob/master/examples/udp.rs)
This is an example of how to use the UDP API.

_Send packets_

```rust
// create the necessarily config, you can edit it or just use the default.
let config = NetworkConfig::default();

// setup an udp socket and bind it to the client address.
let mut udp_socket = UdpSocket::bind("127.0.0.1:12346", config).unwrap();

// create a packet that can be send with the given destination and raw data.
let packet = Packet::new(destination, vec![1,2,3]);

// send the packet to the endpoint we earlier placed into the packet.
udp_socket.send(packet);
```

_Receive Packets_

```rust
// create the necessarily config, you can edit it or just use the default.
let config = NetworkConfig::default();

// setup an udp socket and bind it to the client address.
let mut udp_socket = UdpSocket::bind("127.0.0.1:12345", config).unwrap();

// Start receiving (blocks the current thread)
let result = udp_socket.recv();

match result {
    Ok(Some(packet)) => {
        let endpoint: SocketAddr = packet.addr();
        let received_data: &[u8] = packet.payload();

        // you can here deserialize your bytes into the data you have passed it when sending.

        println!("Received packet from: {:?} with length {}", endpoint, received_data.len());
    }
    Ok(None) => {
        println!("This could happen when we have'n received all data from this packet yet");
    }
    Err(e) => {
        // We get an error if something when wrong like the address is already in use.
        println!("Something went wrong when receiving, error: {:?}", e);
    }
}
```
## Notice
"This library is not yet stable. It is experimental and things may change frequently."

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.

## Authors

- [Lucio Franco](https://github.com/LucioFranco)
- [Fletcher Haynes](https://github.com/fhaynes)
- [Timon Post](https://github.com/TimonPost)

We want to especially give credits to [gaffer on games](https://gafferongames.com/). 
This library is based on his vision about how an gamenetwork protocol should work.

## License

Licensed under either of
 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
at your option.
