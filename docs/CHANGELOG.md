# Change Log
This document contains information about the releases of this crate.
## [0.2.2] - 2019-16-13
- Remove error 'WouldBlock'

## [0.2.2] - 2019-05-06
- Improved Acknowledgement System
- Fixed bug of not resending dropped packets

## [0.2.1] - 2019-05-06
- Yanked version, incorrect code.

## [0.2.0] - 2019-04-13
- Introduced Ordering, Sequencing of packets
- Packets can be arranged on different streams.
- A channel-based API, ready to switch over to MIO
- Removed all locking and reference counters
- Increased Unit Test coverage
- Removed some dependencies
- Introduced socket events: connect, timeout, packet
- Bug fixes
- Restructured code for better organization

## [0.1.0] - 2018-11-12
The Networking team is happy to announce the release of `0.1.0`` of the [laminar crate](https://github.com/amethyst/laminar). 
It provides UDP networking modified for the needs of game networking. 
Most of the techniques used were published and detailed by [Glenn Fiedler](https://gafferongames.com/). 
We’d like to extend a special thanks to him and his articles.

### Added 

- UDP-based protocol
- Automatic Fragmentation
- RTT estimation
- Connection tracking
- Unreliable and Reliable sending of packets
- Protocol version monitoring
- A link conditioner to simulate packet loss and latency
- Good error handling with **zero** panics
- Well tested by integration and unit tests
- Benchmarks
