use serde::{Deserialize, Serialize};

use crate::measurement::Measurement;
use crate::server::Server;

/// Outcome of one completed probe, suitable for JSON serialization or OpenMetrics export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunResult {
    /// Backend name (for example `"librespeed"` or `"iperf3"`).
    pub backend: String,
    pub server: Server,
    pub measurement: Measurement,
}

impl RunResult {
    /// Construct a result from a backend label, server, and measurement.
    pub fn new(backend: impl Into<String>, server: Server, measurement: Measurement) -> Self {
        Self {
            backend: backend.into(),
            server,
            measurement,
        }
    }
}
