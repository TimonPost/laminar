use std::fmt::Debug;
use std::fmt::{self, Display};
use std::time::Duration;
use std::time::Instant;

/// Entry for throughput monitor with measured information.
struct ThroughputEntry {
    measured_throughput: u32,
    start: Instant,
}

impl ThroughputEntry {
    /// Construct a new throughput entry.
    pub fn new(measured_throughput: u32, time: Instant) -> ThroughputEntry {
        ThroughputEntry {
            measured_throughput,
            start: time,
        }
    }
}

/// Helper to monitor throughput.
///
/// Throughput is calculated at some duration.
/// For each duration an entry is created to keep track of the history of throughput.
///
/// With this type you can calculate the average or get the total and last measured throughput.
pub struct ThroughputMonitoring {
    throughput_duration: Duration,
    timer: Instant,
    current_throughput: u32,
    measured_throughput: Vec<ThroughputEntry>,
}

impl ThroughputMonitoring {
    /// Construct a new instance of `ThroughputMonitoring`
    pub fn new(throughput_duration: Duration) -> ThroughputMonitoring {
        ThroughputMonitoring {
            throughput_duration,
            timer: Instant::now(),
            current_throughput: 0,
            measured_throughput: Vec::new(),
        }
    }

    /// This will increase the throughput by one, when the `throughput_duration` has elapsed since the last call, then an throughput entry will be created.
    pub fn tick(&mut self) -> bool {
        if self.timer.elapsed() >= self.throughput_duration {
            self.measured_throughput
                .push(ThroughputEntry::new(self.current_throughput, self.timer));
            self.current_throughput = 0;
            self.timer = Instant::now();
            return true;
        } else {
            self.current_throughput += 1;
            return false;
        }
    }

    /// Returns the average throughput over all throughput up-till now.
    pub fn average(&self) -> u32 {
        if self.measured_throughput.len() != 0 {
            return self
                .measured_throughput
                .iter()
                .map(|x| x.measured_throughput)
                .sum::<u32>()
                / self.measured_throughput.len() as u32;
        }
        0
    }

    /// Reset the throughput history.
    pub fn reset(&mut self) {
        self.current_throughput = 0;
        self.measured_throughput.clear();
    }

    /// Returns the last measured throughput.
    pub fn last_throughput(&self) -> u32 {
        self.measured_throughput.last().unwrap().measured_throughput
    }

    /// Returns the totals measured throughput ticks.
    pub fn total_measured(&self) -> u32 {
        self.measured_throughput
            .iter()
            .map(|x| x.measured_throughput)
            .sum::<u32>()
    }
}

impl Debug for ThroughputMonitoring {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            f,
            "Current Throughput: {}, Elapsed Time: {:?}, Average Throughput: {}",
            self.last_throughput(),
            self.timer.elapsed(),
            self.average()
        )
    }
}

impl Display for ThroughputMonitoring {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            f,
            "Current Throughput: {}, Elapsed Time: {:?}, Average Throughput: {}",
            self.last_throughput(),
            self.timer.elapsed(),
            self.average()
        )
    }
}
