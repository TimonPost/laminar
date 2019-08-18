# This book is still in development and is just a draft. And there's still a lot of maintenance to be done.

## Introduction

Welcome! This book will teach you everything you need to know about of the networking in laminar.
We will discuss important parts of network programming, why we made certain decisions and some explanations about networking concepts in general.

Laminar is free and open source software, distributed under a dual license of [MIT][ml]
and [Apache][al]. This means that the engine is given to you at no cost
and its source code is completely yours to tinker with. The code is available on
[GitHub][am]. Contributions and feature requests will always be welcomed!Kan

[ml]: https://github.com/amethyst/laminar/blob/master/docs/LICENSE-MIT
[al]: https://github.com/amethyst/laminar/blob/master/docs/LICENSE-APACHE
[am]: https://github.com/amethyst/laminar/tree/master

## Motivation
With this library we want to make optimal use of the fundamental rust features. 
By doing this library might become a good replacement for other game protocol implementations written with other languages. 
This library is written for the amethyst game-engine, but I fully intend to use it separately for other projects. 
We do this because there are few options for a reliable UDP implementation written in rust.

## Similar Projects
We used some inspiration from other similar projects.

- [NetCode IO, C++ with go,rust, C# bindings](https://github.com/networkprotocol/netcode.io)
- [RakNet, C++](https://github.com/SLikeSoft/SLikeNet)
- [Steam Network Socket, , C++](https://github.com/ValveSoftware/GameNetworkingSockets)
- [LiteNetLib, C#](https://github.com/RevenantX/LiteNetLib)
- [ENet, C](http://enet.bespin.org/)

## Contributing
We are always happy to welcome new contributors!

If you want to contribute, or have questions, let us know either on [GitHub][db], or on [Discord][di] (#net).

[di]: https://discord.gg/amethyst
[db]: https://github.com/amethyst/laminar/
