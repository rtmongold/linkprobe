use clap::Parser;
use linkprobe_core::backends::LibreSpeedEngine;
use linkprobe_core::{Error, Measurement, MeasurementEngine, Server};
use serde::Serialize;

#[derive(Debug, Parser)]
#[command(
    name = "linkprobe",
    version,
    about = "Protocol-agnostic network link measurement"
)]
struct Cli {
    /// LibreSpeed (or compatible) base URL, e.g. https://speed.example/
    #[arg(long)]
    server: String,

    /// Emit machine-readable JSON.
    #[arg(long)]
    json: bool,

    /// Override download path (default: backend/garbage.php).
    #[arg(long)]
    dl_path: Option<String>,

    /// Override upload path (default: backend/empty.php).
    #[arg(long)]
    ul_path: Option<String>,

    /// Override ping path (default: backend/empty.php).
    #[arg(long)]
    ping_path: Option<String>,
}

#[derive(Debug, Serialize)]
struct RunResult<'a> {
    server: &'a Server,
    measurement: &'a Measurement,
}

fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    let mut server = Server::librespeed(cli.server);
    if let Some(p) = cli.dl_path {
        server.dl_path = p;
    }
    if let Some(p) = cli.ul_path {
        server.ul_path = p;
    }
    if let Some(p) = cli.ping_path {
        server.ping_path = p;
    }

    let engine = LibreSpeedEngine::new()?;
    let measurement = engine.measure(&server)?;

    if cli.json {
        let out = RunResult {
            server: &server,
            measurement: &measurement,
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&out).map_err(|e| Error::Message(e.to_string()))?
        );
    } else {
        println!("Server:    {}", server.name);
        if let Some(ms) = measurement.latency_ms {
            println!("Latency:   {ms:.1} ms");
        }
        if let Some(ms) = measurement.jitter_ms {
            println!("Jitter:    {ms:.1} ms");
        }
        if let Some(dl) = measurement.download {
            println!("Download:  {:.1} Mbps", dl.mbps());
        }
        if let Some(ul) = measurement.upload {
            println!("Upload:    {:.1} Mbps", ul.mbps());
        }
    }
    Ok(())
}
