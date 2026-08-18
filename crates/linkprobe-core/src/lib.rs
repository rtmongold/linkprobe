//! Protocol-agnostic link measurement types and engine trait.

mod discovery;
mod error;
pub mod export;
mod measurement;
mod result;
mod server;

pub mod backends;

pub use discovery::{
    DEFAULT_IPERF3_SERVERS_URL, DEFAULT_LIBRESPEED_SERVERS_URL, fetch_iperf3_servers,
    fetch_librespeed_servers, parse_iperf3_servers, parse_librespeed_servers, pick_lowest_latency,
    server_by_id, servers_list_url,
};
pub use error::Error;
pub use export::{format_openmetrics, format_openmetrics_failed};
pub use measurement::{Measurement, Throughput};
pub use result::RunResult;
pub use server::Server;

/// Runs latency / download / upload (and optional loss) against a server.
pub trait MeasurementEngine {
    fn measure(&self, server: &Server) -> Result<Measurement, Error>;
}
