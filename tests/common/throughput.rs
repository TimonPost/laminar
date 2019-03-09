use std::time::Duration;
use std::time::Instant;

struct ThroughputEntry {
    measured_throughput: u32,
    start: Instant,
}

impl ThroughputEntry {
    pub fn new(measured_throughput: u32, time: Instant) -> ThroughputEntry {
        ThroughputEntry {
            measured_throughput,
            start: time
        }
    }
}

pub struct ThroughputMonitoring {
    throughput_duration: Duration,
    timer: Instant,
    current_throughput: u32,
    measured_throughput: Vec<ThroughputEntry>
}

impl ThroughputMonitoring {
    pub fn new(throughput_duration: Duration) -> ThroughputMonitoring {
        ThroughputMonitoring {
            throughput_duration,
            timer: Instant::now(),
            current_throughput: 0,
            measured_throughput: Vec::new()
        }
    }

    pub fn tick(&mut self) {
        if self.timer.elapsed() >= self.throughput_duration {
            self.measured_throughput.push(ThroughputEntry::new(self.current_throughput, self.timer));
            self.current_throughput = 0;
            self.timer = Instant::now();
        } else {
            self.current_throughput += 1;
        }
    }

    pub fn average(&self) -> u32 {
        self.measured_throughput.iter().map(|x| x.measured_throughput).sum::<u32>() / self.measured_throughput.len() as u32
    }

    pub fn reset(&mut self) {
        self.current_throughput = 0;
        self.measured_throughput.clear();
    }

    pub fn current_throughput(&self) -> u32 {
        self.current_throughput
    }
}