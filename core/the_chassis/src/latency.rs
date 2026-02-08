//! Módulo de medición de latencia

use std::time::{Duration, Instant};

pub struct LatencyTracker {
    samples: Vec<Duration>,
    max_samples: usize,
}

impl LatencyTracker {
    pub fn new(max_samples: usize) -> Self {
        Self {
            samples: Vec::with_capacity(max_samples),
            max_samples,
        }
    }

    pub fn record(&mut self, latency: Duration) {
        if self.samples.len() >= self.max_samples {
            self.samples.remove(0);
        }
        self.samples.push(latency);
    }

    pub fn average_ms(&self) -> u128 {
        if self.samples.is_empty() {
            return 0;
        }
        let total: Duration = self.samples.iter().sum();
        total.as_millis() / self.samples.len() as u128
    }

    pub fn is_optimal(&self) -> bool {
        self.average_ms() < 150
    }
}
