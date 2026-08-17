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
    udp: bool,
    bandwidth: String,
}

impl Iperf3Engine {
    pub fn new() -> Self {
        Self {
            binary: PathBuf::from("iperf3"),
            duration_secs: DEFAULT_DURATION_SECS,
            udp: false,
            bandwidth: "10M".into(),
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

    pub fn with_udp(mut self, udp: bool) -> Self {
        self.udp = udp;
        self
    }

    pub fn with_bandwidth(mut self, bandwidth: impl Into<String>) -> Self {
        let b = bandwidth.into();
        self.bandwidth = if b.is_empty() { "10M".into() } else { b };
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
        if self.udp {
            cmd.arg("-u").arg("-b").arg(&self.bandwidth);
        }
        if reverse {
            cmd.arg("-R");
        }

        let output = cmd.output().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                Error::Iperf3Missing
            } else {
                Error::probe("iperf3", e)
            }
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::probe(
                "iperf3",
                Error::Message(format!("exited {}: {stderr}", output.status)),
            ));
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

    fn receiver_sum(end: &IperfEnd) -> Option<&IperfSum> {
        end.sum_received
            .as_ref()
            .or(end.sum.as_ref())
            .or(end.sum_sent.as_ref())
    }

    pub fn udp_stats(json: &Iperf3Json) -> (Option<f64>, Option<f64>) {
        let Some(end) = json.end.as_ref() else {
            return (None, None);
        };
        let Some(sum) = Self::receiver_sum(end) else {
            return (None, None);
        };
        let jitter = sum.jitter_ms;
        let loss = sum.lost_percent.map(|p| (p / 100.0).clamp(0.0, 1.0));
        (jitter, loss)
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
        let (jitter_ms, packet_loss) = if self.udp {
            let (j1, l1) = Self::udp_stats(&dl);
            let (j2, l2) = Self::udp_stats(&ul);
            let jitter = match (j1, j2) {
                (Some(a), Some(b)) => Some((a + b) / 2.0),
                (a, b) => a.or(b),
            };
            let loss = match (l1, l2) {
                (Some(a), Some(b)) => Some((a + b) / 2.0),
                (a, b) => a.or(b),
            };
            (jitter, loss)
        } else {
            (None, None)
        };
        Ok(Measurement {
            latency_ms: None,
            jitter_ms,
            download: Some(Throughput::from_bps(Self::bps_from_json(&dl, true)?)),
            upload: Some(Throughput::from_bps(Self::bps_from_json(&ul, false)?)),
            packet_loss,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct Iperf3Json {
    pub end: Option<IperfEnd>,
}

#[derive(Debug, Deserialize)]
pub struct IperfEnd {
    pub sum: Option<IperfSum>,
    pub sum_sent: Option<IperfSum>,
    pub sum_received: Option<IperfSum>,
}

#[derive(Debug, Deserialize)]
pub struct IperfSum {
    pub bits_per_second: f64,
    pub jitter_ms: Option<f64>,
    pub lost_percent: Option<f64>,
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

    #[test]
    fn parses_udp_jitter_and_loss() {
        let json = include_str!("../../tests/fixtures/iperf3/udp.json");
        let parsed: Iperf3Json = serde_json::from_str(json).unwrap();
        let (jitter, loss) = Iperf3Engine::udp_stats(&parsed);
        assert_eq!(jitter, Some(1.5));
        assert!((loss.unwrap() - 0.02).abs() < 1e-9);
        assert!(Iperf3Engine::bps_from_json(&parsed, true).unwrap() > 0.0);
    }
}
