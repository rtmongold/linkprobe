//! Format a sample [`RunResult`] as OpenMetrics text (no network).
//!
//! ```bash
//! cargo run -p linkprobe-core --example openmetrics
//! ```

use linkprobe_core::{Measurement, RunResult, Server, Throughput, format_openmetrics};

fn main() {
    let result = RunResult::new(
        "librespeed",
        Server::librespeed("https://example-librespeed/"),
        Measurement {
            latency_ms: Some(12.5),
            jitter_ms: Some(1.2),
            download: Some(Throughput::from_bps(100_000_000.0)),
            upload: Some(Throughput::from_bps(40_000_000.0)),
            packet_loss: None,
        },
    );
    print!("{}", format_openmetrics(&result));
}
