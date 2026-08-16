use std::io::Read;
use std::time::Instant;

use reqwest::blocking::{Body, Client};
use url::Url;

use crate::measurement::{Measurement, Throughput};
use crate::server::Server;
use crate::{Error, MeasurementEngine};

const PING_SAMPLES: usize = 10;
const DOWNLOAD_CHUNK_SIZE: u32 = 4; // 4 MiB from garbage.php
const UPLOAD_BYTES: usize = 2 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct LibreSpeedEngine {
    client: Client,
}

impl LibreSpeedEngine {
    pub fn new() -> Result<Self, Error> {
        let client = Client::builder()
            .user_agent(concat!("linkprobe/", env!("CARGO_PKG_VERSION")))
            .timeout(std::time::Duration::from_secs(60))
            .build()?;
        Ok(Self { client })
    }

    fn join(base: &str, path: &str) -> Result<Url, Error> {
        let base = if base.ends_with('/') {
            base.to_string()
        } else {
            format!("{base}/")
        };
        Ok(Url::parse(&base)?.join(path)?)
    }

    fn ping_jitter(&self, server: &Server) -> Result<(f64, f64), Error> {
        let url = Self::join(&server.base_url, &server.ping_path)?;
        let mut samples_ms = Vec::with_capacity(PING_SAMPLES);

        for _ in 0..PING_SAMPLES {
            let start = Instant::now();
            let resp = self.client.get(url.clone()).send()?;
            let _ = resp.bytes()?;
            samples_ms.push(start.elapsed().as_secs_f64() * 1000.0);
        }

        let latency = samples_ms.iter().sum::<f64>() / samples_ms.len() as f64;
        let mut jitter_acc = 0.0;
        for w in samples_ms.windows(2) {
            jitter_acc += (w[1] - w[0]).abs();
        }
        let jitter = if samples_ms.len() > 1 {
            jitter_acc / (samples_ms.len() - 1) as f64
        } else {
            0.0
        };
        Ok((latency, jitter))
    }

    fn download(&self, server: &Server) -> Result<Throughput, Error> {
        let mut url = Self::join(&server.base_url, &server.dl_path)?;
        url.query_pairs_mut()
            .append_pair("ckSize", &DOWNLOAD_CHUNK_SIZE.to_string());

        let start = Instant::now();
        let resp = self.client.get(url).send()?;
        let bytes = resp.bytes()?;
        let secs = start.elapsed().as_secs_f64().max(1e-6);
        Ok(Throughput::from_bps((bytes.len() as f64) * 8.0 / secs))
    }

    fn upload(&self, server: &Server) -> Result<Throughput, Error> {
        let url = Self::join(&server.base_url, &server.ul_path)?;
        let payload = vec![0_u8; UPLOAD_BYTES];
        let len = payload.len() as f64;

        let start = Instant::now();
        let resp = self
            .client
            .post(url)
            .header(reqwest::header::CONTENT_TYPE, "application/octet-stream")
            .body(Body::from(payload))
            .send()?;
        // Drain response so the request fully completes.
        let mut sink = std::io::sink();
        std::io::copy(&mut resp.take(64 * 1024), &mut sink)
            .map_err(|e| Error::Message(format!("failed reading upload response: {e}")))?;
        let secs = start.elapsed().as_secs_f64().max(1e-6);
        Ok(Throughput::from_bps(len * 8.0 / secs))
    }
}

impl Default for LibreSpeedEngine {
    fn default() -> Self {
        Self::new().expect("failed to build reqwest client")
    }
}

impl MeasurementEngine for LibreSpeedEngine {
    fn measure(&self, server: &Server) -> Result<Measurement, Error> {
        let (latency_ms, jitter_ms) = self.ping_jitter(server)?;
        let download = self.download(server)?;
        let upload = self.upload(server)?;
        Ok(Measurement {
            latency_ms: Some(latency_ms),
            jitter_ms: Some(jitter_ms),
            download: Some(download),
            upload: Some(upload),
            packet_loss: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server as MockServer;

    #[test]
    fn measures_against_mock_librespeed() {
        let mut server = MockServer::new();

        let ping = server
            .mock("GET", "/backend/empty.php")
            .with_status(200)
            .with_body("")
            .expect_at_least(PING_SAMPLES)
            .create();

        let dl_body = vec![1_u8; DOWNLOAD_CHUNK_SIZE as usize * 1024 * 1024];
        let download = server
            .mock("GET", "/backend/garbage.php")
            .match_query(mockito::Matcher::UrlEncoded(
                "ckSize".into(),
                DOWNLOAD_CHUNK_SIZE.to_string(),
            ))
            .with_status(200)
            .with_body(dl_body)
            .create();

        let upload = server
            .mock("POST", "/backend/empty.php")
            .with_status(200)
            .with_body("")
            .create();

        let engine = LibreSpeedEngine::new().unwrap();
        let target = Server::librespeed(server.url());
        let m = engine.measure(&target).unwrap();

        assert!(m.latency_ms.unwrap() >= 0.0);
        assert!(m.jitter_ms.unwrap() >= 0.0);
        assert!(m.download.unwrap().bps > 0.0);
        assert!(m.upload.unwrap().bps > 0.0);

        ping.assert();
        download.assert();
        upload.assert();
    }
}
