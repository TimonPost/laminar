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
- No way of getting the dropped packet.
- Duplication possible.
- No [fragmentation](./../fragmentation.md).

So it would be useful if we could somehow specify the features we want on top of UDP. 
Like that you say: I want the guarantee for my packets to arrive, however they don't need to be in order. 
Or, I don't care if my packet arrives but I do want to receive only new ones.

Please check out [ordering documentation](ordering.md), it describes what ordering and sequencing is.    

Laminar provides 5 different ways for you to send your data:

| Reliability Type                 | Packet Drop | Packet Duplication | Packet Order  | Packet Fragmentation |Packet Delivery|
| :-------------:                  | :-------------: | :-------------:    | :-------------:  | :-------------:  | :-------------:
|       **Unreliable**              |       Yes       |       Yes          |      No          |      No          |       No
|       **Unreliable Sequenced**    |       Yes       |      No            |      Sequenced   |      No          |       No
|       **Reliable Unordered**      |       No        |      No            |      No          |      Yes         |       Yes
|       **Reliable Ordered**        |       No        |      No            |      Ordered     |      Yes         |       Yes
|       **Reliable Sequenced**      |       No        |      No            |      Sequenced   |      Yes         |       Yes


## Unreliable
Unreliable: Packets can be dropped, duplicated or arrive without order.

**Details**

| Packet Drop     | Packet Duplication | Packet Order     | Packet Fragmentation | Packet Delivery |
| :-------------: | :-------------:    | :-------------:  | :-------------:      | :-------------: |
|       Yes       |        Yes         |      No          |      No              |       No        |

Basically just bare UDP. The packet may or may not be delivered.

// todo: add use cases

## Unreliable Sequenced
Unreliable Sequenced: Packets can be dropped, but could not be duplicated and arrive in sequence.

*Details*

| Packet Drop     | Packet Duplication | Packet Order     | Packet Fragmentation | Packet Delivery |
| :-------------: | :-------------:    | :-------------:  | :-------------:      | :-------------: |
|       Yes       |        Yes         |      Sequenced          |      No              |       No        |

Basically just bare UDP, free to be dropped, but has some sequencing to it so that only the newest packets are kept.

// todo: add use cases

## Reliable Unordered
Reliable: All packets will be sent and received, but without order.

*Details*

|   Packet Drop   | Packet Duplication | Packet Order     | Packet Fragmentation | Packet Delivery |
| :-------------: | :-------------:    | :-------------:  | :-------------:      | :-------------: |
|       No        |      No            |      No          |      Yes             |       Yes       |

Basically, this is almost TCP without ordering of packets.

// todo: add use cases
## Reliable Ordered
Reliable; All packets will be sent and received, with order.

*Details*

|   Packet Drop   | Packet Duplication | Packet Order     | Packet Fragmentation | Packet Delivery |
| :-------------: | :-------------:    | :-------------:  | :-------------:      | :-------------: |
|       No        |      No            |      Ordered     |      Yes             |       Yes       |

Basically this is almost like TCP.

// todo: add use cases

## Reliable Sequenced
Reliable; All packets will be sent and received but arranged in sequence.
Which means that only the newest packets will be let through, older packets will be received but they won't get to the user.

*Details*

|   Packet Drop   | Packet Duplication | Packet Order     | Packet Fragmentation | Packet Delivery |
| :-------------: | :-------------:    | :-------------:  | :-------------:      | :-------------: |
|       No        |      No            |      Sequenced     |      Yes             |       Yes       |

Basically this is almost TCP-like but then sequencing instead of ordering.

// todo: add use cases

## Interesting Reads
- [RakNet Reliability Types](http://www.jenkinssoftware.com/raknet/manual/reliabilitytypes.html)