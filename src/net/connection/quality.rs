use crate::config::Config;
use crate::sequence_buffer::CongestionData;

use std::sync::Arc;
use std::time::Duration;

/// Represents the quality of a network.
#[allow(dead_code)]
pub enum NetworkQuality {
    /// Connection is generally good, minimal packet loss or latency
    Good,
    /// Connection is generally bad, having an impact on game performance
    Bad,
}

/// This type helps with calculating the round trip time from any packet.
/// It is able to smooth out the network jitter if there is any.
pub struct RttMeasurer {
    config: Arc<Config>,
}

impl RttMeasurer {
    /// Creates and returns a new RttMeasurer
    pub fn new(config: Arc<Config>) -> RttMeasurer {
        RttMeasurer { config }
    }

    /// This will calculate the round trip time (rtt) from the given acknowledgement.
    /// Where after it updates the rtt from the given connection.
    pub fn get_rtt(&self, congestion_data: Option<&mut CongestionData>) -> f32 {
        self.get_smoothed_rtt(congestion_data)
    }

    /// This will get the smoothed round trip time (rtt) from the time we last heard from an packet.
    fn get_smoothed_rtt(&self, congestion_avoidance_entry: Option<&mut CongestionData>) -> f32 {
        match congestion_avoidance_entry {
            Some(avoidance_data) => {
                let elapsed_time = avoidance_data.sending_time.elapsed();

                let rtt_time = self.as_milliseconds(elapsed_time);

                self.smooth_out_rtt(rtt_time)
            }
            None => 0.0,
        }
    }

    /// Converts a duration to milliseconds.
    ///
    /// `as_milliseconds` is not supported yet supported in rust stable.
    /// See this stackoverflow post for more info: https://stackoverflow.com/questions/36816072/how-do-i-get-a-duration-as-a-number-of-milliseconds-in-rust
    fn as_milliseconds(&self, duration: Duration) -> u64 {
        let nanos = u64::from(duration.subsec_nanos());
        (1000 * 1000 * 1000 * duration.as_secs() + nanos) / (1000 * 1000)
    }

    /// Smooth out round trip time (rtt) value by the specified smoothing factor.
    ///
    /// First we subtract the max allowed rtt.
    /// This way we can see by how many we are off from the max allowed rtt.
    /// Then we multiply with or smoothing factor.
    ///
    /// We do this so that if one packet has an bad rtt it will not directly bring down the or network quality estimation.
    /// The default is 10% smoothing so if in total or packet is 50 milliseconds later than max allowed rtt we will increase or rtt estimation with 5.
    fn smooth_out_rtt(&self, rtt: u64) -> f32 {
        let exceeded_rrt_time = rtt as i64 - i64::from(self.config.rtt_max_value);
        exceeded_rrt_time as f32 * self.config.rtt_smoothing_factor
    }
}

#[cfg(test)]
mod test {
    use super::RttMeasurer;
    use crate::config::Config;
    use crate::net::connection::VirtualConnection;
    use std::net::ToSocketAddrs;
    use std::sync::Arc;
    use std::time::Duration;

    static TEST_HOST_IP: &'static str = "127.0.0.1";
    static TEST_PORT: &'static str = "20000";

    #[test]
    fn test_create_connection() {
        let mut addr = format!("{}:{}", TEST_HOST_IP, TEST_PORT)
            .to_socket_addrs()
            .unwrap();
        let _new_conn = VirtualConnection::new(addr.next().unwrap(), Arc::new(Config::default()));
    }

    #[test]
    fn convert_duration_to_milliseconds_test() {
        let network_quality = RttMeasurer::new(Arc::new(Config::default()));
        let milliseconds1 = network_quality.as_milliseconds(Duration::from_secs(1));
        let milliseconds2 = network_quality.as_milliseconds(Duration::from_millis(1500));
        let milliseconds3 = network_quality.as_milliseconds(Duration::from_millis(1671));

        assert_eq!(milliseconds1, 1000);
        assert_eq!(milliseconds2, 1500);
        assert_eq!(milliseconds3, 1671);
    }

    #[test]
    fn smooth_out_rtt() {
        let mut config = Config::default();
        // for test purpose make sure we set smoothing factor to 10%.
        config.rtt_smoothing_factor = 0.10;
        config.rtt_max_value = 250;

        let network_quality = RttMeasurer::new(Arc::new(config));
        let smoothed_rtt = network_quality.smooth_out_rtt(300);

        // 300ms has exceeded 50ms over the max allowed rtt. So we check if or smoothing factor is now 10% from 50.
        assert_eq!(smoothed_rtt, 5.0);
    }
}
