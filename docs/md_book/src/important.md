## Some Important Notices

There are a few important things you need to know in order to use laminar in a good way. 
If you do not follow these rules then it is possible that either laminar is not suitable for your use case, or laminar does not work properly.

1. Packet Consistency

    Make sure that the client and the server send consistent messages to each other, if you don't do this, 
the connection may break, and cause the reliability and order aspect of laminar not to work. For more information checkout [heartbeat implementation](heartbeat.md).

2. Reliability, transferring big data
    Laminar is not tested for transferring large files. 
    The [fragments](fragmentation.md) of the fragmented packet will not be acknowledged. 
    So if a fragment is lost, the whole packet is lost.  For more information checkout [fragmentation](fragmentation.md) and [reliability](./reliability/basicis.md).
3. DDOS Protection

    DDOS protection ensures that a client that sends something is not simply identified as a trustworthy connection. 
If this was the case, someone could easily spoof packets and DDOS our server with new connections. 

    **Make sure the server responds to a message from the client, if the server responds, then the client will only be added to the connections. 
    And you will be able to normally communicate with the client**
    
    In the future we want to have a [handshaking process](https://github.com/amethyst/laminar/issues/156) to simplify this process.


[config]: https://github.com/amethyst/laminar/blob/master/src/config.rs#L8
[DDOS]: https://github.com/amethyst/laminar/issues/187