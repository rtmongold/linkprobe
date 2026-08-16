use clap::{Parser, ValueEnum};
use linkprobe_core::backends::{Iperf3Engine, LibreSpeedEngine};
use linkprobe_core::{Error, Measurement, MeasurementEngine, Server};
use serde::Serialize;

#[derive(Debug, Clone, ValueEnum)]
enum Backend {
    LibreSpeed,
    Iperf3,
}

#[derive(Debug, Parser)]
#[command(
    name = "linkprobe",
    version,
    about = "Protocol-agnostic network link measurement"
)]
struct Cli {
    /// Measurement backend.
    #[arg(long, value_enum, default_value_t = Backend::LibreSpeed)]
    backend: Backend,

    /// LibreSpeed (or compatible) base URL, e.g. https://speed.example/
    #[arg(long)]
    server: String,

    /// Iperf3 port (default: 5201).
    #[arg(long, default_value_t = 5201)]
    port: u16,

    /// iperf3 test duration per direction, seconds (default: 5).
    #[arg(long, default_value_t = 5)]
    duration: u64,

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
    backend: &'a str,
    server: &'a Server,
    measurement: &'a Measurement,
}

fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    let (backend_name, server, measurement) = match cli.backend {
        Backend::LibreSpeed => {
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
            ("librespeed", server, measurement)
        }
        Backend::Iperf3 => {
            let server = Server::iperf3(cli.server, cli.port);
            let engine = Iperf3Engine::new().with_duration_secs(cli.duration);
            let measurement = engine.measure(&server)?;
            ("iperf3", server, measurement)
        }
    };

    if cli.json {
        let out = RunResult {
            backend: backend_name,
            server: &server,
            measurement: &measurement,
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&out).map_err(|e| Error::Message(e.to_string()))?
        );
    } else {
        println!("Backend:   {}", backend_name);
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
