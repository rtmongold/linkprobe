//! Protocol-agnostic link measurement types and engine trait.

mod discovery;
mod error;
mod measurement;
mod server;

pub mod backends;

pub use discovery::{
    DEFAULT_LIBRESPEED_SERVERS_URL, fetch_librespeed_servers, parse_librespeed_servers,
    pick_lowest_latency, server_by_id,
};
pub use error::Error;
pub use measurement::{Measurement, Throughput};
pub use server::Server;

/// Runs latency / download / upload (and optional loss) against a server.
pub trait MeasurementEngine {
    fn measure(&self, server: &Server) -> Result<Measurement, Error>;
}
