# Networking protocols

When building any networked application the first and possibly the important decision to make is which protocol to use and when. Laminar is built on top of UDP. Let’s first take a quick look at both TCP and UDP, then we'll explain why Laminar uses UDP.

## IP

All communication over the internet is happening over IP (Internet Protocol).
This protocol only passes packets across the network without any guarantee that it will arrive at the destination.
Sometimes IP passes along multiple copies of the same packet and these packets make their way to the destination via different paths, causing packets to arrive out of order and in duplicate.

So to be able to communicate over the network we make use of existing protocols that provides some more certainty.
We will first take a look at TCP where after we checkout UPD.

## TCP/IP

TCP stands for “transmission control protocol”. IP stands for “internet protocol”.
Together they form the backbone for almost everything you do online, from web browsing to IRC to email, it’s all built on top of TCP/IP.

TCP is a connection-oriented protocol, which means a connection is established and maintained until the application programs at each end have finished exchanging messages.
TCP provides full reliable, ordered communication between two machines. The data you send is guaranteed to arrive and in order.
The TCP protocol will also split up and reassemble packets if those are too large.

**Characteristics**

- Reliable
- Ordered
- Automatic [fragmentation](fragmentation.md) of packets
- Stream based
- Control Flow ([Congestion Avoidance](congestion_avoidence/congestion_avoidance.md))

## UDP

UDP stands for “user datagram protocol” and it’s another protocol built on top of IP, but unlike TCP, instead of adding lots of features and complexity, UDP is a very thin layer over IP.

Like IP, UDP is an unreliable protocol. In practice however, most packets that are sent will get through, but you’ll usually have around 1-5% packet loss, and occasionally you’ll get periods where no packets get through at all (remember there are lots of computers between you and your destination where things can go wrong…)

**Characteristics**

- Not Reliable
- Not Ordered
- No [fragmentation](fragmentation.md) of packets
- No control flow ([Congestion Avoidance](congestion_avoidence/congestion_avoidance.md))
- Packet loss could happen.
- Message based

## Why UDP and not TCP | More

Those of you familiar with TCP know that it already has its own concept of connection, reliability-ordering and congestion avoidance, so why are we rewriting our own mini version of TCP on top of UDP?

The issue is that multiplayer action games rely on a steady stream of packets sent at rates of 10 to 30 packets per second, and for the most part, the data contained in these packets is so time sensitive that only the most recent data is useful.
This includes data such as player inputs, the position, orientation and velocity of each player character, and the state of physics objects in the world.

The problem with TCP is that it abstracts data delivery as a reliable ordered stream. Because of this, if a packet is lost, TCP has to stop and wait for that packet to be resent.
This interrupts the steady stream of packets because more recent packets must wait in a queue until the resent packet arrives, so packets are received in the same order they were sent.

What we need is a different type of reliability.

Instead of having all data treated as a reliable ordered stream, we want to send packets at a steady rate and get notified when packets are received by the other computer.
This allows time sensitive data to get through without waiting for resent packets, while letting us make our own decision about how to handle packet loss at the application level.

What TCP does is maintain a sliding window where the ACK sent is the sequence number of the next packet it expects to receive, in order. If TCP does not receive an ACK for a given packet, it stops and re-sends a packet with that sequence number again. This is exactly the behavior we want to avoid!

It is not possible to implement a reliability system with these properties using TCP, so we have no choice but to roll our own reliability on top of UDP. TCP itself is built on UDP.

## When use TCP

Of course there could be use-cases for TCP like chat, asset streaming, etc. We can setup a TCP socket for this that is distinct from UDP.

We could also make our UDP channel reliable as described below so when we detect package lost on the client we could construct a new package
