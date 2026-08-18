mod mqtt;

use clap::{Parser, ValueEnum};
use linkprobe_core::backends::{Iperf3Engine, LibreSpeedEngine};
use linkprobe_core::{
    DEFAULT_LIBRESPEED_SERVERS_URL, Error, MeasurementEngine, RunResult, Server,
    fetch_iperf3_servers, fetch_librespeed_servers, format_openmetrics, pick_lowest_latency,
    server_by_id, servers_list_url,
};
use reqwest::blocking::Client;

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

    /// Select server by list id (from --list).
    #[arg(long)]
    server_id: Option<u64>,

    /// Print server list and exit.
    #[arg(long)]
    list: bool,

    /// Server list JSON URL (LibreSpeed or iperf3 depending on the --backend).
    #[arg(long, default_value = DEFAULT_LIBRESPEED_SERVERS_URL)]
    servers_url: String,

    /// Iperf3 port (default: 5201).
    #[arg(long, default_value_t = 5201)]
    port: u16,

    /// iperf3 test duration per direction, seconds (default: 5).
    #[arg(long, default_value_t = 5)]
    duration: u64,

    /// iperf3 UDP mode (-u -b). TCP is the default.
    #[arg(long)]
    udp: bool,

    /// iperf3 UDP target bitrate (-b). Default: 10M.
    #[arg(long, default_value = "10M")]
    bandwidth: String,

    /// Emit machine-readable JSON.
    #[arg(long)]
    json: bool,

    /// Write OpenMetrics text. Omit PATH or use `-` for stdout.
    #[arg(long, value_name = "PATH", num_args = 0..=1, default_missing_value = "-")]
    prometheus_text: Option<String>,

    /// MQTT broker URL, e.g. mqtt://127.0.0.1:1883
    #[arg(long)]
    mqtt_url: Option<String>,

    /// MQTT topic (default: linkprobe/result).
    #[arg(long, default_value = "linkprobe/result")]
    mqtt_topic: String,

    /// MQTT username (optional).
    #[arg(long)]
    mqtt_username: Option<String>,

    ///MQTT password (optional).
    #[arg(long)]
    mqtt_password: Option<String>,

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

fn resolve_iperf3(cli: &Cli) -> Result<Server, Error> {
    if cli.server.is_some() && cli.server_id.is_some() {
        return Err(Error::Message(
            "use only one of --server or --server-id".into(),
        ));
    }

    if let Some(host) = &cli.server {
        return Ok(Server::iperf3(host.clone(), cli.port));
    }

    let client = list_client()?;
    let url = servers_list_url(true, &cli.servers_url);
    let servers = fetch_iperf3_servers(&client, url)?;

    if let Some(id) = cli.server_id {
        return server_by_id(&servers, id);
    }

    Err(Error::Message(
        "--server or --server-id is required for --backend iperf3".into(),
    ))
}

fn write_prometheus_text(path: &str, text: &str) -> Result<(), Error> {
    if path == "-" {
        print!("{text}");
        return Ok(());
    }
    let tmp = format!("{path}.tmp");
    std::fs::write(&tmp, text)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Error> {
    let cli = Cli::parse();

    if cli.udp && !matches!(cli.backend, Backend::Iperf3) {
        return Err(Error::Message(
            "--udp is only supported for --backend iperf3".into(),
        ));
    }

    if cli.list {
        let client = list_client()?;
        let servers = match cli.backend {
            Backend::LibreSpeed => fetch_librespeed_servers(&client, &cli.servers_url)?,
            Backend::Iperf3 => {
                let list_url = servers_list_url(true, &cli.servers_url);
                fetch_iperf3_servers(&client, list_url)?
            }
        };
        for s in servers {
            use std::io::Write;
            if writeln!(std::io::stdout(), "{:>4} {}", s.id, s.name).is_err() {
                break;
            }
        }
        return Ok(());
    }

    let result = match cli.backend {
        Backend::LibreSpeed => {
            let server = resolve_librespeed(&cli)?;
            let engine = LibreSpeedEngine::new()?;
            let measurement = engine.measure(&server)?;
            RunResult::new("librespeed", server, measurement)
        }
        Backend::Iperf3 => {
            let server = resolve_iperf3(&cli)?;
            let engine = Iperf3Engine::new().with_duration_secs(cli.duration);
            let measurement = engine.measure(&server)?;
            RunResult::new("iperf3", server, measurement)
        }
    };

    let prom_stdout = cli.prometheus_text.as_deref() == Some("-");
    if !prom_stdout {
        if cli.json {
            println!(
                "{}",
                serde_json::to_string_pretty(&result).map_err(|e| Error::Message(e.to_string()))?
            );
        } else {
            println!("Backend:   {}", result.backend);
            println!("Server:    {}", result.server.name);
            if let Some(ms) = result.measurement.latency_ms {
                println!("Latency:   {ms:.1} ms");
            }
            if let Some(ms) = result.measurement.jitter_ms {
                println!("Jitter:    {ms:.1} ms");
            }
            if let Some(loss) = result.measurement.packet_loss {
                println!("Packet loss: {:.1}%", loss * 100.0);
            }
            if let Some(dl) = result.measurement.download.clone() {
                println!("Download:  {:.1} Mbps", dl.mbps());
            }
            if let Some(ul) = result.measurement.upload.clone() {
                println!("Upload:    {:.1} Mbps", ul.mbps());
            }
        }
    }

    if let Some(path) = &cli.prometheus_text {
        let text = format_openmetrics(&result);
        write_prometheus_text(path, &text)?;
    }

    if let Some(url) = &cli.mqtt_url {
        let payload = serde_json::to_string(&result).map_err(|e| Error::Message(e.to_string()))?;
        mqtt::publish_json(
            url,
            &cli.mqtt_topic,
            cli.mqtt_username.as_deref(),
            cli.mqtt_password.as_deref(),
            &payload,
        )?;
    }
    Ok(())
}
