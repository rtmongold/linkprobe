use serde::{Deserialize, Serialize};

/// Throughput in bits per second.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Throughput {
    /// Bits per second.
    pub bps: f64,
}

impl Throughput {
    /// Construct from a raw bit rate.
    pub fn from_bps(bps: f64) -> Self {
        Self { bps }
    }

    /// Convert to megabits per second (decimal `1_000_000` bps per Mbps).
    pub fn mbps(self) -> f64 {
        self.bps / 1_000_000.0
    }
}

/// Result of one link measurement against a server.
///
/// Fields are optional when the backend does not report them (for example packet loss is only
/// filled for iperf3 UDP runs).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Measurement {
    /// Round-trip latency in milliseconds.
    pub latency_ms: Option<f64>,
    /// Jitter in milliseconds.
    pub jitter_ms: Option<f64>,
    /// Download throughput when measured.
    pub download: Option<Throughput>,
    /// Upload throughput when measured.
    pub upload: Option<Throughput>,
    /// Packet loss as a fraction in \[0.0, 1.0\] when known.
    pub packet_loss: Option<f64>,
}
