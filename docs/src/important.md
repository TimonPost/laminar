## Some Important Notices

There are a few important things you need to know in order to use laminar appropriately.
If you do not follow these rules, then it is possible that either laminar is not suitable for your use case, and/or it will not work as expected.

1. Packet Consistency
    Make sure that the client and the server send messages to each other at a consistent rate, e.g. 30Hz. If you don't do this,
    the connection may break, and cause the reliability and order aspect of laminar not to work. For more information checkout [heartbeat implementation](heartbeat.md).

2. Reliability, transferring big data
    Laminar is not designed for transferring large files.
    The [fragments](fragmentation.md) of the fragmented packet will not be acknowledged. 
    So if a fragment is lost, the whole packet is lost. Although this will be improved in the future, for more information checkout [fragmentation](fragmentation.md) and [reliability](reliability/basics.md).
3. DDOS Protection

    DDOS protection ensures that a client that sends something is not simply identified as a trustworthy connection. 
    If this were the case, someone could easily spoof packets and DDOS our server with new connections.

    **Make sure the server responds to a message from the client. Only if the server responds, will the connection to the client be tracked.**
    
    In the future we want to have a [handshaking process](https://github.com/amethyst/laminar/issues/156) to simplify this process.


[config]: https://github.com/amethyst/laminar/blob/master/src/config.rs#L8
[DDOS]: https://github.com/amethyst/laminar/issues/187