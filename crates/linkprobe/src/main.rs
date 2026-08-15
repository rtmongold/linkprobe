use clap::Parser;
use linkprobe_core::Error;

#[derive(Debug, Parser)]
#[command(
    name = "linkprobe",
    version,
    about = "Protocol-agnostic network link measurement"
)]
struct Cli {
    /// Emit machine-readable JSON (when a backend is available).
    #[arg(long)]
    json: bool,
}

fn main() -> Result<(), Error> {
    let _cli = Cli::parse();
    Err(Error::NotImplemented)
}
