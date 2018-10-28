//! This module provides means to simulate various network conditions for development. The primary focus is
//! for testing applications under adverse conditions such as high packet loss networks, or high latency
//! networks.

use rand::prelude::random;

pub struct LinkConditioner {
    // Value between 0 and 1, representing the % change a packet will be dropped on sending
    packet_loss: f64,
    // Value in milliseconds, representing the delay imposed between packets
    latency: u32,
}

impl LinkConditioner {
    /// Creates and returns a LinkConditioner
    pub fn new() -> LinkConditioner {
        LinkConditioner {
            packet_loss: 0.0,
            latency: 0,
        }
    }

    /// Sets the packet loss rate of Link Conditioner
    pub fn set_packet_loss(&mut self, rate: f64) {
        self.packet_loss = rate;
    }

    /// Function that checks to see if a packet should be dropped or not
    pub fn should_send(&self) -> bool {
        let r = random::<f64>();
        if r <= self.packet_loss {
            true
        } else {
            false
        }
    }
}
