# Fragmentation
Fragmentation is dividing large packets into smaller fragments so that it can be send over the network.

TCP will automatically divide packets into smaller parts if you sent large amounts of data. But UDP does'nt support this fragmentation. 
That is why we need to implement it or self. 

How large may a packet be? We call this [MTU](https://en.wikipedia.org/wiki/Maximum_transmission_unit) which stands for maximum transmission unit. 
On the Internet today (2016, IPv4) the real-world MTU is 1500 bytes. 
When an packet is larger than 1500 bytes we need to split it up into different fragments.
Why 1500? That’s the default MTU for MacOS X and Windows. 

So when we send a packet we need to follow this algorithm:
1. Check the size of the payload. 
2. If the payload size is greater than the max allowed MTU divide the payload of the packet into x fragments. 
3. Create a normal packet.
4. Send the fragments with the sequence number of the above parents packet it's sequence number.
5. When receiving the fragment store it in an buffer. 
6. Once we have received all fragments combine the fragments and construct the payload for the user.

You should take note that each fragment will not be acknowledged with our implementation. 
So if you would send 200.000 bytes (+- 133 fragments) the risk of one fragment being dropped will be huge. 
If you really want to sent large amounts of data over the line go for TCP instead, since that protocol is build for reliability and large data. 

When sending small packets with the size of about 4000 bytes (4 fragments) this method will work fine. And won't probably cause any problems.





