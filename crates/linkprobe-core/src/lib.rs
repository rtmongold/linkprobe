//! Protocol-agnostic network link measurement for Rust.
//!
//! `linkprobe-core` provides shared types, server discovery, measurement backends, and
//! OpenMetrics formatting. The [`linkprobe`](https://github.com/rtmongold/linkprobe) CLI
//! crate adds argument parsing, MQTT publish, and an HTTP scrape endpoint on top of this
//! library.
//!
//! Use this crate when you want to embed link measurement in an agent, dashboard, or test
//! harness. Use the CLI when you want a ready-made probe with JSON, Prometheus, and MQTT
//! exporters.
//!
//! # Backends
//!
//! All engines implement [`MeasurementEngine`] and run **synchronously** (blocking HTTP or a
//! subprocess). Pick the backend that matches your endpoint:
//!
//! | Backend | Type | Requires | Typical fields |
//! | --- | --- | --- | --- |
//! | LibreSpeed | [`LibreSpeedEngine`](backends::LibreSpeedEngine) | Outbound HTTPS | latency, jitter, download, upload |
//! | iperf3 | [`Iperf3Engine`](backends::Iperf3Engine) | `iperf3` on `PATH` | latency, jitter, download, upload; UDP adds packet loss |
//!
//! Optional fields on [`Measurement`] mean the backend did not report that metric for the run
//! (for example TCP iperf3 has no packet loss).
//!
//! # Example
//!
//! ```no_run
//! use linkprobe_core::backends::LibreSpeedEngine;
//! use linkprobe_core::{MeasurementEngine, Server};
//!
//! let server = Server::librespeed("https://example-librespeed/");
//! let engine = LibreSpeedEngine::new()?;
//! let measurement = engine.measure(&server)?;
//!
//! if let Some(ms) = measurement.latency_ms {
//!     println!("latency: {ms:.1} ms");
//! }
//! # Ok::<(), linkprobe_core::Error>(())
//! ```
//!
//! Discovery helpers such as [`fetch_librespeed_servers`] and [`rank_by_latency`] need a
//! network connection. See [`FAILOVER_EXTRA`] for list rotation behavior used by the CLI.

mod discovery;
mod error;
pub mod export;
mod measurement;
mod result;
mod server;

pub mod backends;

pub use discovery::{
    DEFAULT_IPERF3_SERVERS_URL, DEFAULT_LIBRESPEED_SERVERS_URL, FAILOVER_EXTRA,
    failover_candidates, fetch_iperf3_servers, fetch_librespeed_servers, parse_iperf3_servers,
    parse_librespeed_servers, pick_lowest_latency, rank_by_latency, server_by_id, servers_list_url,
};
pub use error::Error;
pub use export::{format_openmetrics, format_openmetrics_failed};
pub use measurement::{Measurement, Throughput};
pub use result::RunResult;
pub use server::Server;

/// Runs latency, download, upload, and optional packet-loss measurement against a [`Server`].
///
/// A single call performs the full probe for that backend (ping/jitter plus throughput tests).
/// Implement this trait to add new measurement protocols alongside
/// [`LibreSpeedEngine`](backends::LibreSpeedEngine) and [`Iperf3Engine`](backends::Iperf3Engine).
pub trait MeasurementEngine {
    fn measure(&self, server: &Server) -> Result<Measurement, Error>;
}
