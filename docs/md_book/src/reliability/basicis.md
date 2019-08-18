# Introduction
The internet is a dangerous place, and before you know it your data is gone or your data arrives duplicated because your data is split up along the way to its final destination. 
In order to have more control over the way in which the data is transported, we have invented protocols. 

In this chapter we will consider how laminar gives you more control over the transport of data.

## Important
TCP is made for reliability and does this very well. 
We have been asked many times by people why reliability does not work well or is slow in laminar.
Important to know is that laminar has reliability as an option but it is not focused on that and isn't made to be faster and better than TCP. 
In the fast phased game industry, it is not possible to use TCP. Because a delay in a packet can have a major impact on all subsequent packets.
Reliability, after all, is less important for fast-phased FPS games; UDP. 
Consider to use TCP when 1) you need less frequent reliable packet communication 2) big file transfers 3) when not working with fast phased FPS games.
That said, laminar will support acknowledgement of fragments in the future. Checkout [fragmentation](../fragmentation.md) for more info.

- [Ordering](ordering.md)
How can we control the way the data is ordered.
- [Reliability](reliability.md)
How can we control the arrival of our data.