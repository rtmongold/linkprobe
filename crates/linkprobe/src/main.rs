use clap::{Parser, ValueEnum};
use linkprobe_core::backends::{Iperf3Engine, LibreSpeedEngine};
use linkprobe_core::{
    DEFAULT_LIBRESPEED_SERVERS_URL, Error, Measurement, MeasurementEngine, Server,
    fetch_librespeed_servers, pick_lowest_latency, server_by_id,
};
use reqwest::blocking::Client;
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

    /// LibreSpeed base URL or iperf3 host. Optional for LibreSpeed (auto picks if omitted).
    #[arg(long)]
    server: Option<String>,

    /// Select LibreSpeed server by list id (from --list).
    #[arg(long)]
    server_id: Option<u64>,

    /// Print LibreSpeed server list and exit.
    #[arg(long)]
    list: bool,

    /// LibreSpeed servers JSON URL.
    #[arg(long, default_value = DEFAULT_LIBRESPEED_SERVERS_URL)]
    servers_url: String,

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

fn list_client() -> Result<Client, Error> {
    Ok(Client::builder()
        .user_agent(concat!("linkprobe/", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(30))
        .build()?)
}

fn resolve_librespeed(cli: &Cli) -> Result<Server, Error> {
    if cli.server.is_some() && cli.server_id.is_some() {
        return Err(Error::Message(
            "use only one of --server or --server-id".into(),
        ));
    }

    if let Some(url) = &cli.server {
        let mut server = Server::librespeed(url.clone());
        if let Some(p) = &cli.dl_path {
            server.dl_path = p.clone();
        }
        if let Some(p) = &cli.ul_path {
            server.ul_path = p.clone();
        }
        if let Some(p) = &cli.ping_path {
            server.ping_path = p.clone();
        }
        return Ok(server);
    }

    let client = list_client()?;
    let servers = fetch_librespeed_servers(&client, &cli.servers_url)?;

    if let Some(id) = cli.server_id {
        return server_by_id(&servers, id);
    }

    let (server, _ms) = pick_lowest_latency(&client, &servers)?;
    Ok(server)
}

fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    if cli.list {
        if !matches!(cli.backend, Backend::LibreSpeed) {
            return Err(Error::Message(
                "--list is only supported for LibreSpeed".into(),
            ));
        }
        let client = list_client()?;
        let servers = fetch_librespeed_servers(&client, &cli.servers_url)?;
        for s in servers {
            println!("{:>4} {}", s.id, s.name);
        }
        return Ok(());
    }

    let (backend_name, server, measurement) = match cli.backend {
        Backend::LibreSpeed => {
            let server = resolve_librespeed(&cli)?;
            let engine = LibreSpeedEngine::new()?;
            let measurement = engine.measure(&server)?;
            ("librespeed", server, measurement)
        }
        Backend::Iperf3 => {
            let host = cli.server.ok_or_else(|| {
                Error::Message("--server is required for --backend iperf3".into())
            })?;
            let server = Server::iperf3(host, cli.port);
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
