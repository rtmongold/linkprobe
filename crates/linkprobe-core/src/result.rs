use serde::{Deserialize, Serialize};

use crate::measurement::Measurement;
use crate::server::Server;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunResult {
    pub backend: String,
    pub server: Server,
    pub measurement: Measurement,
}

impl RunResult {
    pub fn new(backend: impl Into<String>, server: Server, measurement: Measurement) -> Self {
        Self {
            backend: backend.into(),
            server,
            measurement,
        }
    }
}
