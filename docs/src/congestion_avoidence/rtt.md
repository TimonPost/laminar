# Round Trip Time (RTT)
The time between you sending the packet and you receiving an acknowledgment from the other side is called RTT. 
To avoid congestion we first need to find a way to calculate the `RTT` value of our connection so we can decide on top of that value if we have bad or good internet speeds.

_Smoothing factor_

So you could say: "very simple, measure the time between sending and receiving you got the `RTT` and you're done right?" No! This is because a packet can travel any path over the internet the `RTT` can always defer every time you calculate it. And imagine a short internet lag we will directly get a huge RTT back. So we need to smooth out that RTT factor by some amount. Gaffer says that 10% of the RTT will be just fine. With this smoothed RTT we will be able to add it to our current RTT. 

_Allowed RTT value_

So now we have the smoothed RTT and our current RTT, GREAT! But RTT on its own is not bad. So there may be some max allowed RTT. We need to subtract that amount from our measured RTT multiplied by the smoothing factor. 

The formula would look like the following:

```
// rtt_max_value is in ms
// rtt_smoothing_factor is in %
let new_rtt_value = (rtt - rtt_max_value) * rtt_smoothing_factor.
```
Lets look at an example with numbers. The RTT values are in milliseconds.

_bad internet_
```
// this will result into: 5
let new_rtt_value = (300 - 250) * 0.10.
```

_good internet_
```
// this will result into: -15
let new_rtt_value = (100 - 250) * 0.10.
```

As you see when our calculation is under 250ms we get a negative result, which is in this case positive. 
When our calculation is above 250ms it will be positive, which is in this case negative.

So each time we receive an acknowledgment we can add our result, of the above formula, to the RTT time saved in the connection.

## Interesting Reads
- [Wikipedia](https://en.wikipedia.org/wiki/Round-trip_delay_time) 
