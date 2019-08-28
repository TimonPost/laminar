# Reliability

So let's talk about reliability. 
This is a very important concept which could be at first sight difficult but which will be very handy later on.
j
As you know we have two opposites, TCP on one hand and UDP on the other. 
TCP has a lot of feature UDP does not have, like shown below.

_TCP_
- Guarantee of delivery.
- Guarantee for order.
- Packets will not be dropped.
- Duplication not possible.
- Automatic [fragmentation](./../fragmentation.md).

_UDP_
- Unreliable.
- No guarantee for delivery.
- No guarantee for order.
- No way of getting the dropped packet.
- Duplication possible.
- No [fragmentation](./../fragmentation.md).

So it would be useful if we could somehow specify the features we want on top of UDP. 
Like that you say: I want the guarantee for my packets to arrive, however they don't need to be in order. 
Or, I don't care if my packet arrives but I do want to receive only new ones.

Before continuing, it would be helpful to understand the difference between ordering and sequencing: [ordering documentation](ordering.md)

## The 5 Reliability Guarantees
Laminar provides 5 different ways for you to send your data:

| Reliability Type             | Packet Drop     | Packet Duplication | Packet Order     | Packet Fragmentation |Packet Delivery|
| :-------------:              | :-------------: | :-------------:    | :-------------:  | :-------------:      | :-------------:
|   **Unreliable Unordered**   |       Any       |      Yes           |     No           |      No              |   No
|   **Unreliable Sequenced**   |    Any + old    |      No            |     Sequenced    |      No              |   No
|   **Reliable Unordered**     |       No        |      No            |     No           |      Yes             |   Yes
|   **Reliable Ordered**       |       No        |      No            |     Ordered      |      Yes             |   Yes
|   **Reliable Sequenced**     |    Only old     |      No            |     Sequenced    |      Yes             |   Only newest


## Unreliable
Unreliable: Packets can be dropped, duplicated or arrive in any order.

**Details**

| Packet Drop     | Packet Duplication | Packet Order     | Packet Fragmentation | Packet Delivery |
| :-------------: | :-------------:    | :-------------:  | :-------------:      | :-------------: |
|       Any       |      Yes           |     No           |      No              |   No

Basically just bare UDP. The packet may or may not be delivered.

## Unreliable Sequenced
Unreliable Sequenced: Packets can be dropped, but could not be duplicated and arrive in sequence.

*Details*

| Packet Drop     | Packet Duplication | Packet Order     | Packet Fragmentation | Packet Delivery |
| :-------------: | :-------------:    | :-------------:  | :-------------:      | :-------------: |
|    Any + old    |      No            |     Sequenced    |      No              |   No

Basically just bare UDP, free to be dropped, but has some sequencing to it so that only the newest packets are kept.

## Reliable Unordered
Reliable UnOrder: All packets will be sent and received, but without order.

*Details*

|   Packet Drop   | Packet Duplication | Packet Order     | Packet Fragmentation | Packet Delivery |
| :-------------: | :-------------:    | :-------------:  | :-------------:      | :-------------: |
|       No        |      No            |     No           |      Yes             |   Yes

Basically, this is almost TCP without ordering of packets.

## Reliable Ordered
Reliable Unordered: All packets will be sent and received, but in the order in which they arrived.

*Details*

|   Packet Drop   | Packet Duplication | Packet Order     | Packet Fragmentation | Packet Delivery |
| :-------------: | :-------------:    | :-------------:  | :-------------:      | :-------------: |
|       No        |      No            |     Ordered      |      Yes             |   Yes

Basically this is almost like TCP.

## Reliable Sequenced
Reliable; All packets will be sent and received but arranged in sequence.
Which means that only the newest packets will be let through, older packets will be received but they won't get to the user.

*Details*

|   Packet Drop   | Packet Duplication | Packet Order     | Packet Fragmentation | Packet Delivery |
| :-------------: | :-------------:    | :-------------:  | :-------------:      | :-------------: |
|    Only old     |      No            |     Sequenced    |      Yes             |   Only newest

Basically this is almost TCP-like but then sequencing instead of ordering.


### Example
```rust
use laminar::Packet;

// You can create packets with different reliabilities
let unreliable = Packet::unreliable(destination, bytes);
let reliable = Packet::reliable_unordered(destination, bytes);

// We can specify on which stream and how to order our packets, checkout our book and documentation for more information
let unreliable = Packet::unreliable_sequenced(destination, bytes, Some(1));
let reliable_sequenced = Packet::reliable_sequenced(destination, bytes, Some(2));
let reliable_ordered = Packet::reliable_ordered(destination, bytes, Some(3));
```

# Related
- [RakNet Reliability Types](http://www.jenkinssoftware.com/raknet/manual/reliabilitytypes.html)