# Release Notes
This document contains information about the releases of this crate.

## Version 0.1.0
The Networking team is happy to announce the release of 0.1.0 of the [laminar crate](https://github.com/amethyst/laminar). 
It provides UDP networking modified for the needs of game networking. 
Most of the techniques used were published and detailed by [Glenn Fiedler](https://gafferongames.com/). 
We’d like to extend a special thanks to him and his articles.

**Features**
- UDP-based protocol
- Automatic Fragmentation
- RTT estimation
- Connection tracking.
- Unreliable and Reliable sending of packets.
- Protocol version monitoring.
- A link conditioner to simulate packet loss and latency
- Good error handling with **zero** panics.
- Well tested by integration and unit tests
- Benchmarks