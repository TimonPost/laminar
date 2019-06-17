//! This module provides means to simulate various network conditions for development. The primary focus is
//! for testing applications under adverse conditions such as high packet loss networks, or high latency
//! networks. This is not in heavy use yet, hence the allowing dead code. These will be removed as our testing
//! becomes more sophisticated.

use rand::prelude::random;
use std::time::Duration;

#[derive(Debug)]
pub struct LinkConditioner {
    // Value between 0 and 1, representing the % change a packet will be dropped on sending
    packet_loss: f64,
    // Duration of the delay imposed between packets
    latency: Duration,
}

impl LinkConditioner {
    /// Creates and returns a LinkConditioner
    #[allow(dead_code)]
    pub fn new() -> LinkConditioner {
        LinkConditioner {
            packet_loss: 0.0,
            latency: Duration::default(),
        }
    }

    /// Sets the packet loss rate of Link Conditioner
    #[allow(dead_code)]
    pub fn set_packet_loss(&mut self, rate: f64) {
        self.packet_loss = rate;
    }

    /// Sets the latency the link conditioner should apply to each packet
    #[allow(dead_code)]
    pub fn set_latency(&mut self, latency: Duration) {
        self.latency = latency
    }

    /// Function that checks to see if a packet should be dropped or not
    pub fn should_send(&self) -> bool {
        random::<f64>() >= self.packet_loss
    }
}
