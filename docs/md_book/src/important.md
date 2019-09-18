## Some Important Notices

There are a few important things you need to know in order to use laminar appropriately.
If you do not follow these rules, then it is possible that either laminar is not suitable for your use case, and/or it will not work as expected.

1. Packet Consistency:

    Make sure that the client and the server send messages to each other at a consistent rate, i.e. 30Hz. If you don't do this,
    the connection may close, or cause the reliability and order aspect of laminar to be laggy. For more information checkout [heartbeat implementation](heartbeat.md).

2. Reliability and transferring big data:

    Laminar is not designed for transferring large amounts of data.
    The [fragments](fragmentation.md) of the fragmented packet will not be acknowledged.
    So if a fragment is lost - the whole packet is lost. This will likely be improved in the future, for more information check out [fragmentation](fragmentation.md) and [reliability](reliability/basics.md).

3. DoS Protection

    DoS protection ensures that new clients are unable to use memory resources on the machine.
    If this were the case, some malicious actor could easily spoof packets and DoS our server with new connections.

    **Make sure to respond to a message from another endpoint. Only if we respond, will the connection to the endpoint be stored.**

    In the future we want to have a [handshaking process](https://github.com/amethyst/laminar/issues/156) to simplify this process.


[config]: https://github.com/amethyst/laminar/blob/master/src/config.rs#L8
[DoS]: https://github.com/amethyst/laminar/issues/187
