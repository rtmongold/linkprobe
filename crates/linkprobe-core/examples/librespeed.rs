//! Run a LibreSpeed probe and print the result as JSON.
//!
//! ```bash
//! cargo run -p linkprobe-core --example librespeed -- https://example-librespeed/
//! ```
//!
//! With no URL argument, auto-picks the lowest-latency public LibreSpeed server.

use linkprobe_core::backends::LibreSpeedEngine;
use linkprobe_core::{
    DEFAULT_LIBRESPEED_SERVERS_URL, MeasurementEngine, RunResult, Server, fetch_librespeed_servers,
    pick_lowest_latency,
};
use reqwest::blocking::Client;
use std::env;

fn main() -> Result<(), linkprobe_core::Error> {
    let server = match env::args().nth(1) {
        Some(url) => Server::librespeed(url),
        None => {
            let client = Client::builder()
                .user_agent(concat!("linkprobe/", env!("CARGO_PKG_VERSION")))
                .timeout(std::time::Duration::from_secs(30))
                .build()?;
            let servers = fetch_librespeed_servers(&client, DEFAULT_LIBRESPEED_SERVERS_URL)?;
            let (server, ms) = pick_lowest_latency(&client, &servers)?;
            eprintln!("auto-picked {} ({ms:.1} ms ping)", server.name);
            server
        }
    };

    let engine = LibreSpeedEngine::new()?;
    let measurement = engine.measure(&server)?;
    let result = RunResult::new("librespeed", server, measurement);
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
