use std::path::PathBuf;
use std::process::Command;

use serde::Deserialize;

use crate::measurement::{Measurement, Throughput};
use crate::server::Server;
use crate::{Error, MeasurementEngine};

const DEFAULT_DURATION_SECS: u64 = 5;
const DEFAULT_PORT: u16 = 5201;

#[derive(Debug, Clone)]
pub struct Iperf3Engine {
    binary: PathBuf,
    duration_secs: u64,
}

impl Iperf3Engine {
    pub fn new() -> Self {
        Self {
            binary: PathBuf::from("iperf3"),
            duration_secs: DEFAULT_DURATION_SECS,
        }
    }

    pub fn with_binary(mut self, binary: impl Into<PathBuf>) -> Self {
        self.binary = binary.into();
        self
    }

    pub fn with_duration_secs(mut self, secs: u64) -> Self {
        self.duration_secs = secs.max(1);
        self
    }

    fn port(server: &Server) -> u16 {
        server.port.unwrap_or(DEFAULT_PORT)
    }

    fn run_json(&self, server: &Server, reverse: bool) -> Result<Iperf3Json, Error> {
        let mut cmd = Command::new(&self.binary);
        cmd.arg("-c")
            .arg(&server.base_url)
            .arg("-p")
            .arg(Self::port(server).to_string())
            .arg("-t")
            .arg(self.duration_secs.to_string())
            .arg("-J");
        if reverse {
            cmd.arg("-R");
        }

        let output = cmd.output().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                Error::Message(
                    "iperf3 not found on PATH (install iperf3 to use --backend iperf3)".into(),
                )
            } else {
                Error::Io(e)
            }
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Message(format!(
                "iperf3 exited {}: {stderr}",
                output.status
            )));
        }

        Ok(serde_json::from_slice(&output.stdout)?)
    }

    /// Parse bps from fixture/live JSON. `reverse` → prefer received (download).
    pub fn bps_from_json(json: &Iperf3Json, reverse: bool) -> Result<f64, Error> {
        let end = json
            .end
            .as_ref()
            .ok_or_else(|| Error::Message("iperf3 JSON missing end block".into()))?;

        let bps = if reverse {
            end.sum_received
                .as_ref()
                .or(end.sum_sent.as_ref())
                .map(|s| s.bits_per_second)
        } else {
            end.sum_sent
                .as_ref()
                .or(end.sum_received.as_ref())
                .map(|s| s.bits_per_second)
        };

        bps.ok_or_else(|| Error::Message("iperf3 JSON missing bps".into()))
    }
}

impl Default for Iperf3Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl MeasurementEngine for Iperf3Engine {
    fn measure(&self, server: &Server) -> Result<Measurement, Error> {
        let dl = self.run_json(server, true)?;
        let ul = self.run_json(server, false)?;
        Ok(Measurement {
            latency_ms: None,
            jitter_ms: None,
            download: Some(Throughput::from_bps(Self::bps_from_json(&dl, true)?)),
            upload: Some(Throughput::from_bps(Self::bps_from_json(&ul, false)?)),
            packet_loss: None,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct Iperf3Json {
    pub end: Option<IperfEnd>,
}

#[derive(Debug, Deserialize)]
pub struct IperfEnd {
    pub sum_sent: Option<IperfSum>,
    pub sum_received: Option<IperfSum>,
}

#[derive(Debug, Deserialize)]
pub struct IperfSum {
    pub bits_per_second: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_forward_and_reverse_fixtures() {
        let forward = include_str!("../../tests/fixtures/iperf3/forward.json");
        let reverse = include_str!("../../tests/fixtures/iperf3/reverse.json");
        let fwd: Iperf3Json = serde_json::from_str(forward).unwrap();
        let rev: Iperf3Json = serde_json::from_str(reverse).unwrap();

        let up = Iperf3Engine::bps_from_json(&fwd, false).unwrap();
        let down = Iperf3Engine::bps_from_json(&rev, true).unwrap();
        assert!(up > 0.0);
        assert!(down > 0.0);
    }
}
