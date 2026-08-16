//! Protocol-agnostic link measurement types and engine trait.

mod error;
mod measurement;
mod server;

pub mod backends;

pub use error::Error;
pub use measurement::{Measurement, Throughput};
pub use server::Server;

/// Runs latency / download / upload (and optional loss) against a server.
pub trait MeasurementEngine {
    fn measure(&self, server: &Server) -> Result<Measurement, Error>;
}
