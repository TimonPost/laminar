# Reliability

So let's talk about reliability. 
This is a very important concept which could be at first sight difficult but which will be very handy later on.

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
- No way of getting dropped packet.
- Duplication possible.
- No [fragmentation](./../fragmentation.md).

So handy would be if you somehow could specify which features you want on top of UDP. 
You could say for example I want the guarantee for my packets to arrive, however they don't need to be in order. 

Laminar (will) provide(s) different kind of reliabilities as listed below:

### Unreliable Unordered
Unreliable. Packets can be dropped, duplicated or arrive without order.

 **Details**

| Packet Drop     | Packet Duplication | Packet Order     | Packet Fragmentation | Packet Delivery |
| :-------------: | :-------------:    | :-------------:  | :-------------:      | :-------------: |
|       Yes       |        Yes         |      No          |      No              |       No        |

Basically just bare UDP, free to be dropped, used for very unnecessary data, great for 'general' position 

### Unreliable Ordered
Unreliable. Packets can be dropped, duplicated or arrive with order.

**Details**

| Packet Drop      | Packet Duplication  | Packet Order      | Packet Fragmentation | Packet Delivery |
| :-------------:  | :-------------:     | :-------------:  | :-------------:       | :-------------: |
|      Yes        |    Yes               |      Yes          |      No              |       No        |

Basically just bare UDP, free to be dropped, used for very unnecessary data, great for 'general' position updates but packets will be ordered.


### Reliable Unordered
Reliable. All packets will be sent and received, but without order.

*Details*

|   Packet Drop   | Packet Duplication | Packet Order     | Packet Fragmentation | Packet Delivery |
| :-------------: | :-------------:    | :-------------:  | :-------------:      | :-------------: |
|       No        |      No            |      No          |      Yes             |       Yes       |

Basically this is almost TCP like without ordering of packets.
Receive every packet and immediately give to application, order does not matter.

### Reliable Ordered
Reliable. All packets will be sent and received, with order.

*Details*

|   Packet Drop   | Packet Duplication | Packet Order     | Packet Fragmentation | Packet Delivery |
| :-------------: | :-------------:    | :-------------:  | :-------------:      | :-------------: |
|       No        |      No            |      Yes         |      Yes             |       Yes       |

Basically this is almost has all features TCP has.
Receive every packet (file downloading for example) in order (any missing keeps the later ones buffered.
 
### Sequenced
Unreliable. Packets can be dropped, but never duplicated and arrive in order.

*Details*

|   Packet Drop   | Packet Duplication | Packet Order     | Packet Fragmentation | Packet Delivery |
| :-------------: | :-------------:    | :-------------:  | :-------------:      | :-------------: |
|       Yes       |      No            |      Yes         |      Yes             |       No        |

Toss away any packets that are older than the most recent (like a position update, you don't care about older ones),
packets may be dropped, just the application may not receive older ones if a newer one came in first.
 
 -----------------------------------------------------------------------------------
 However all those options are listed above only a few will be supported for laminar version `0.1.0` like: UnreliableUnordered, ReliableUnordered, SequencedUnordered. 
 However for laminar version `0.2.0` we are planning to support: UnreliableOrdered, ReliableOrdered, SequencedOrdered also.
 