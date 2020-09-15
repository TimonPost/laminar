# Change Log
This document contains information about the releases of this crate.

## [0.4.0] - 2019-09-24
- Better interface separation with clear functionality boundaries.
- Separated unit tests and integration tests.
- Removed `set_link_conditioner` from `Socket`
- Make `last_sent` sent with unreliable packets.
- Change allocations to iterators. 
- Canonicalizes an "established connection" as requiring both a send and a receive from a given endpoint.
- Change `SocketEvent::Connect` to only fire when a connection has been established.
- Add `SocketEvent::Disconnect` to fire when a Timeout fires from an endpoint with an established connection.
- Change `SocketEvent::Timeout` to fire when a Timeout fires from an endpoint with an unestablished connection.
- Change heartbeats to only send after a connection has been established.
- Add a `max_unestablished_connections` config option to prevent connection memory overflow attack

## [0.3.2] - 2019-09-24
- Acknowledgement is sent after all fragments arrived
- Don't read out-of-bounds on malformed headers 

## [0.3.1] - 2019-09-16
- Documentation improvements (docs, book, readme)
- Allow a Socket to be in blocking mode
- Default heartbeat functionality
- Series of patches and bug-fixes for ordering, sequencing. 
- Disconnect the connection after sending N un-acked packets
- Dependency maintenance (removed and increased versions)
- A lot of new unit tests

## [0.3.0] - 2019-06-29
- Moved the packet sender and event receiver into socket struct
- Exposed internal SocketAddr
- Introduced a new method to manually step through the polling loop
- Added a number of #[derive(Debug)] lines to Socket and member types
- Implemented basic DoS mitigation
- Added a customizable sleep to the polling loop. Defaults to 1ms

## [0.2.3] - 2019-06-13
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
