use std::sync::RwLockWriteGuard;
use std::time::Duration;

use net::connection::VirtualConnection;

use net::NetworkConfig;
use sequence_buffer::CongestionData;

/// Represents the quality of an network.
pub enum NetworkQuality {
    Good,
    Bad,
}

/// This type helps with calculating the round trip time from any packet.
/// It is able to smooth out the network jitter if there is any.
pub struct NetworkQualityMeasurer {
    config: NetworkConfig,
}

impl NetworkQualityMeasurer {
    pub fn new(config: NetworkConfig) -> NetworkQualityMeasurer {
        NetworkQualityMeasurer { config }
    }

    /// This will calculate the round trip time (rtt) from the given acknowledgement.
    /// Where after it updates the rtt from the given connection.
    pub fn update_connection_rtt(
        &self,
        connection: &mut RwLockWriteGuard<VirtualConnection>,
        ack_seq: u16,
    ) {
        let mut smoothed_rrt = 0.0;
        {
            let mut congestion_data = connection.congestion_avoidance_buffer.get_mut(ack_seq);
            smoothed_rrt = self.get_smoothed_rtt(congestion_data);
        }

        connection.rtt = smoothed_rrt;
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

    /// Converts an duration to milliseconds.
    ///
    /// `as_milliseconds` is not supported yet supported in rust stable.
    /// See this stackoverflow post for more info: https://stackoverflow.com/questions/36816072/how-do-i-get-a-duration-as-a-number-of-milliseconds-in-rust
    fn as_milliseconds(&self, duration: Duration) -> u64 {
        let nanos = duration.subsec_nanos() as u64;
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
        let exceeded_rrt_time = rtt as i64 - self.config.rtt_max_value as i64;
        exceeded_rrt_time as f32 * self.config.rtt_smoothing_factor
    }
}

#[cfg(test)]
mod test {
    use net::connection::{VirtualConnection};
    use net::NetworkConfig;
    use sequence_buffer::CongestionData;
    use super::{NetworkQualityMeasurer, RwLockWriteGuard};
    use std::net::ToSocketAddrs;
    use std::time::{Duration, Instant};
    use std::sync::RwLock;

    static TEST_HOST_IP: &'static str = "127.0.0.1";
    static TEST_BAD_HOST_IP: &'static str = "800.0.0.1";
    static TEST_PORT: &'static str = "20000";

    #[test]
    fn test_create_connection() {
        let mut addr = format!("{}:{}", TEST_HOST_IP, TEST_PORT)
            .to_socket_addrs()
            .unwrap();
        let _new_conn = VirtualConnection::new(addr.next().unwrap());
    }

    #[test]
    fn convert_duration_to_milliseconds_test() {
        let network_quality = NetworkQualityMeasurer::new(NetworkConfig::default());
        let milliseconds1 = network_quality.as_milliseconds(Duration::from_secs(1));
        let milliseconds2 = network_quality.as_milliseconds(Duration::from_millis(1500));
        let milliseconds3 = network_quality.as_milliseconds(Duration::from_millis(1671));

        assert_eq!(milliseconds1, 1000);
        assert_eq!(milliseconds2, 1500);
        assert_eq!(milliseconds3, 1671);
    }

    #[test]
    fn smooth_out_rtt() {
        let mut config = NetworkConfig::default();
        // for test purpose make sure we set smoothing factor to 10%.
        config.rtt_smoothing_factor = 0.10;
        config.rtt_max_value = 250;

        let network_quality = NetworkQualityMeasurer::new(config.clone());
        let smoothed_rtt = network_quality.smooth_out_rtt(300);

        // 300ms has exceeded 50ms over the max allowed rtt. So we check if or smoothing factor is now 10% from 50.
        assert_eq!(smoothed_rtt, 5.0);
    }
}
