# Fragmentation
Fragmentation is dividing large packets into smaller fragments so that it can be sent over the network.

TCP will automatically divide packets into smaller parts if you send large amounts of data. But UDP doesn't support fragmentation out-of-the-box. 
Fortunately, laminar does.  

Fragmentation will be applied to packets larger than the [MTU](https://en.wikipedia.org/wiki/Maximum_transmission_unit) with the following reliability types `Reliable Unordered`, `Reliable Ordered`, `Reliable Sequenced`. 

What is this [MTU](https://en.wikipedia.org/wiki/Maximum_transmission_unit)? This stands for 'maximum transmission unit'. 
On the Internet today (2016, IPv4) the real-world MTU is 1500 bytes. 
When a packet is larger than 1500 bytes we need to split it up into different fragments.
Why 1500? That’s the default MTU for MacOS X and Windows. 

You should take note that each fragment will not be acknowledged with our implementation. 
So if you would send 200.000 bytes (+- 133 fragments) the risk of one fragment being dropped will be huge. 
If you really want to send large amounts of data over the line go for TCP instead, since that protocol is built for reliability and large data. 

When sending small packets with the size of about 4000 bytes (4 fragments) this method will work fine. And won't probably cause any problems. 
We are planning to support also [sending larger packets](https://gafferongames.com/post/sending_large_blocks_of_data/) with acknowledgments.

## Interesting Reads
- [Gaffer about Fragmentation](https://gafferongames.com/post/packet_fragmentation_and_reassembly/)
- [Wikipedia](https://en.wikipedia.org/wiki/IP_fragmentation)
- [MTU](https://en.wikipedia.org/wiki/Maximum_transmission_unit)