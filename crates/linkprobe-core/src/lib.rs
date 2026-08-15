//! Protocol-agnostic link measurement types and engine trait.

mod error;
mod measurement;
mod server;

pub use error::Error;
pub use measurement::{Measurement, Throughput};
pub use server::Server;

use std::future::Future;

/// Runs latency / download / upload (and optional loss) against a server.
pub trait MeasurementEngine {
    fn measure(&self, server: &Server) -> impl Future<Output = Result<Measurement, Error>> + Send;
}
