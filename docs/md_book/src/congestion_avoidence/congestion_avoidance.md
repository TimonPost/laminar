# Congestion Avoidance
So lets start at what this congestion avoidance is, if we send just packets without caring about the internet speed of the client we can flood the network. 
Since the router tries to deliver all packages it buffers up all packets in cache. 
We do not want the router to buffer up packets instead it should drop them.
We need to try to avoid sending too much bandwidth in the first place, and then if we detect congestion, we attempt to back off and send even less.

There are a few methods we can implement to defeat congestion.
1. With [RTT](./rtt.md)
2. With [packet loss](./packet_loss.md).

