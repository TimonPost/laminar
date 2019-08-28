## Heartbeat
Laminar offers the possibility to keep the connection with a client open. 
This is done with heartbeat packets. 
This option is enabled by default.
The behavior of the heart beat can be changed in the [configuration](https://github.com/amethyst/laminar/pull/224). 
It can also be disabled.

A client is considered a connection when it sends a packet. 
If the client does not send a packet for x seconds, laminar sees this as an idling connection, and it is removed as an active connection.
When this happens, the following data is removed: 

1) the reliabity data such as aknowleged packets 
2) the buffers that keep track of the ordering/sequencing. 
3) the RTT counter
4) fragmentation data

Losing this data from the memory is often undesirable.
Therefore, it is important to have a consistent flow of packets between the two endpoints which will prevent disconnection of the client.
The time before the client is disconnected can be changed in the [configuration](https://github.com/amethyst/laminar/blob/master/src/config.rs#L10).

## Why a heartbeat?
With game networking for fast-paced FPS games, you have to deal with a lot of data that has to go from point A to B.
We are talking about numbers of 20/30/60 hz. 
Laminar is based and optimized for the situation where a consistent flow of packets from the server to the client and from the client to the server that are being sent.
In a game, where everything runs at milliseconds and speed is important, you need fast communication and multiple updates per seconds.

What are those scenarios and how can I know if laminar is useful for my use case?
You can think of input synchronization, location updates, state updates, events, etc.  
Let's zoom in on input synchronization of an FPS game. 
The client sends the packages, the server receives it, validates it, and sends an update to all other clients. 
In an FPS game, a lot of input is shared, and it's not a strange idea for a client to share its input and receive updates 60 times a second.   
Laminar is based on this idea, and is optimized for it. 
When you are sending packets once a second, laminar might not be the best solution here. And your probably going to do fine with TCP. 

To add to this, note that clients will be seen as 'disconnected' if they don't send packets for some duration, this duration can be found in the [configuration][config]. 
When there is a scenario's that you are sending packets less frequent, laminar has the option to keep the connection alive by sending an heath beat message at a configurable interval.


- [Original PR](https://github.com/amethyst/laminar/pull/224)