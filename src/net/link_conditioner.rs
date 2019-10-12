//! This module provides means to simulate various network conditions for development. The primary focus is
//! for testing applications under adverse conditions such as high packet loss networks, or high latency
//! networks. This is not in heavy use yet, hence the allowing dead code. These will be removed as our testing
//! becomes more sophisticated.

use std::time::Duration;

use rand::Rng;
use rand_pcg::Pcg64Mcg as Random;

/// Network simulator. Used to simulate network conditions as dropped packets and packet delays.
/// For use in [FakeSocket::set_link_conditioner](crate::test_utils::FakeSocket::set_link_conditioner).
#[derive(Clone, Debug)]
pub struct LinkConditioner {
    // Value between 0 and 1, representing the % change a packet will be dropped on sending
    packet_loss: f64,
    // Duration of the delay imposed between packets
    latency: Duration,
    // Random number generator
    random: Random,
}

impl LinkConditioner {
    /// Creates and returns a LinkConditioner
    #[allow(dead_code)]
    pub fn new() -> LinkConditioner {
        LinkConditioner {
            packet_loss: 0.0,
            latency: Duration::default(),
            random: Random::new(0),
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
    pub fn should_send(&mut self) -> bool {
        self.random.gen_range(0.0, 1.0) >= self.packet_loss
    }
}

impl Default for LinkConditioner {
    fn default() -> Self {
        Self::new()
    }
}
