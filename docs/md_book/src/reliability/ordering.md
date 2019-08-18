## Arranging packets

Laminar provides a way to arrange packets, over different streams.

The above sentence contains a lot of important information, let us zoom a little more in at the above sentence.

## Ordering VS Sequencing
Let's define two concepts here:
_"Sequencing: this is the process of only caring about the newest items."_ [1](https://dictionary.cambridge.org/dictionary/english/sequencing)
_"Ordering: this is the process of putting something in a particular order."_ [2](https://dictionary.cambridge.org/dictionary/english/ordering)

- Sequencing: Only the newest items will be passed trough e.g. `1,3,2,5,4` which results in `1,3,5`.
- Ordering: All items are returned in order `1,3,2,5,4` which results in `1,2,3,4,5`.
- Arranging: We call the process for ordering and sequencing 'arranging' of packets

Due to various factors with the internet, anything can happen to the data you send. 
It is not always guaranteed that it will arrive or that it will be in the right order. 
That's why laminar offers these options.

### How ordering works.
If we were to send the following packets: `1,2,3,4,5`, 
but something happens on the internet that ensures that this arrives at its final destination as: `1,5,4,2,3`, 
then laminar makes sure that your packets arrive at the user's destination as `1,2,3,4,5`.

## Arranging Streams
What are these 'arranging streams'?
You can see 'arranging streams' as something to arrange packets that have no relationship at all with one another. 
You could either arrange packets in order or in sequence.

### Simple Example
Think of a highway where you have several lanes where cars are driving.
Because there are these lanes, cars can move on faster.
For example, the cargo drivers drive on the right and the high-speed cars on the left.
The cargo drivers do not influence fast cars and vice versa.

### Real Example
If a game developer wants to send data to a client, he might want to send data either ordered, unordered or sequenced.

'Data' could be the following:
1. Player movement, we want to order player movements because we don't want the player to glitch.
2. Bullet movement, we want to sequence bullet movement because we don't care about old positions of bullets.
3. Chat messages, we want to order chat messages because it is nice to see the text in the right order.

Player movement and chat messages are totally unrelated to each other and you absolutely do not want to interrupt the movement packets if a chat message is not sent.

It would be nice if we could order player movements and chat messages separately. Guess what! This is exactly what 'arranging streams' do.
A game developer can indicate which stream it likes to arrange the packets. 
For example, the game developer can say: "Let me order all chat messages to 'stream 1' and sequence all motion packets on 'stream 2'.

### Example
```rust
// We can specify on which stream and how to order our packets, checkout our book and documentation for more information
let unreliable_sequenced = Packet::unreliable_sequenced(destination, bytes, Some(1));
let reliable_sequenced = Packet::reliable_sequenced(destination, bytes, Some(2));
let reliable_ordered = Packet::reliable_ordered(destination, bytes, Some(3));
```

Take notice of the last `Option` parameter, with this parameter you can specify which streams to order your packets on.
One thing that is important to understand is that 'sequenced streams' are different from 'ordered streams', 
thus specifying `Some(1)` for a sequence stream and `Some(1)` for a ordered packet will not make have those packets ordered on the same stream. 
You can use 254 different ordering or sequencing streams, in reality you'd probably only need a few. When specifying `None`, stream '255' will be used.

## Interesting Reads
- [RakNet Ordering Streams](http://www.raknet.net/raknet/manual/sendingpackets.html)
- [LiteNetLib Implementation](https://github.com/RevenantX/LiteNetLib/issues/67)
