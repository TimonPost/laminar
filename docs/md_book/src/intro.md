# This book is still in active development.

## Introduction

Welcome! This book will teach you everything you need to know about networking with Laminar.
We will discuss important parts of network programming, why we made certain decisions and some explanations about networking concepts in general.

Laminar is free and open source software, distributed under a dual license of [MIT][ml]
and [Apache][al]. This means that the engine is provided to you at no cost
and its source code is completely yours to tinker with. The code is available on
[GitHub][am]. Contributions and feature requests will always be welcomed!

[ml]: https://github.com/amethyst/laminar/blob/master/docs/LICENSE-MIT
[al]: https://github.com/amethyst/laminar/blob/master/docs/LICENSE-APACHE
[am]: https://github.com/amethyst/laminar/tree/master

## Motivation
Laminar is fully written in Rust and therefore has no garbage collector, no data-races, and is completely memory safe.
That's why Laminar is a good candidate to be a safe and better replacement for other reliable-UDP implementations.
This library is originally written for use in the Amethyst game engine, however, Laminar can operate fully without Amethyst.

## Similar Projects
We used some inspiration from other similar projects.

- [NetCode IO, C++ with Go, Rust, C# bindings](https://github.com/networkprotocol/netcode.io)
- [RakNet, C++](https://github.com/SLikeSoft/SLikeNet)
- [Steam Network Socket, , C++](https://github.com/ValveSoftware/GameNetworkingSockets)
- [LiteNetLib, C#](https://github.com/RevenantX/LiteNetLib)
- [ENet, C](http://enet.bespin.org/)

## Contributing
We are always happy to welcome new contributors!

If you want to contribute, or have questions, let us know either on [GitHub][db], or on [Discord][di] (#net).

[di]: https://discord.gg/amethyst
[db]: https://github.com/amethyst/laminar/
