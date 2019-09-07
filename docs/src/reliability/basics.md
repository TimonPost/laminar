# Introduction
The internet is a dangerous place, and before you know it your data is gone or your data arrives duplicated because your data is split up along the way to its final destination. 
In order to have more control over the way in which the data is transported, we have invented protocols. 

In this chapter we will consider how laminar gives you more control over the transport of data.

## Important
TCP is made for reliability and does this very well. 
We have been asked many times by people why reliability does not work well or is slow in laminar.
Important to know is that laminar has reliability as an option but is not focused on trying to be faster and better than TCP. 
For fast-paced multiplayer online games, it is not desirable to use TCP because a delay in a packet can have a major impact on all subsequent packets.
Reliability, after all, is less important for fast-paced FPS games; UDP. 
TCP should be used when the need for reliability trumps the need for low latency
That said, laminar will support acknowledgement of fragments in the future. Checkout [fragmentation](../fragmentation.md) for more info.

- [Ordering](ordering.md)
How can we control the way the data is ordered.
- [Reliability](reliability.md)
How can we control the arrival of our data.