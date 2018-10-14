Yet to be written out... 

There are different implementations that would be an option for this crate. Here are a vew shown: 

- Reliable Ordered 

    Receive every packet (file downloading for example) in order (any missing keeps the later ones buffered until they are received)
- Reliable Sequenced 

    Toss away any packets that are older than the most recent (like a position update, you don't care about older ones), but all are received, just the application may not receive older ones if a newer one came in first.
- Reliable Unordered 

    Receive every packet and immediately give to application, order does not matter.
- Unreliable Ordered 

    Will be passed to the application in order with a specified waiting time to see if any older ones come in within that bound else are dropped.
- Unreliable Sequenced 

    Like Reliable Sequenced but don't resend any missing ones.
- Unreliable Unordered 

    Free to be dropped, used for very unnecessary data, great for 'general' position updates with an occasional reliable one.
