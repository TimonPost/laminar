# Packet Headers 
In this topic we'll discuss the different headers we are pre-pending to the data sent via laminar.
We use different headers in different scenario's, we do this to reduce the packet size. 

Take a look over here: [image](LINK) for the complete design.

- `Standard header`
    
    The first header is the `StandardHeader`, this is included for each packet. 
It contains information like: protocol version, packet type, delivery and ordering guarantees. 

- `AckedHeader`
    
    This header will be included to the header if the packet is reliable. 
It contains information for our acknowledgement system. 

- `FragmentHeader`
    
    This header will be included if the packet payload is bigger than the MTU and thus needs to be [fragmented](./fragmentation.md).
    
- `ArrangingHeader`
    
    This header will be included if the packet needs to be arranged e.g ordered, sequenced. 
    It contains information like the stream it will be arranged on and an identifier for this packet. 