use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Throughput {
    /// Bits per second.
    pub bps: f64,
}

impl Throughput {
    pub fn from_bps(bps: f64) -> Self {
        Self { bps }
    }

    pub fn mbps(self) -> f64 {
        self.bps / 1_000_000.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Measurement {
    /// Round-trip latency in milliseconds.
    pub latency: Option<Duration>,
    /// Jitter in milliseconds.
    pub jitter: Option<Duration>,
    pub download: Option<Throughput>,
    pub upload: Option<Throughput>,
    /// Fraction in [0.0, 1.0] when known.
    pub packet_loss: Option<f64>,
}
