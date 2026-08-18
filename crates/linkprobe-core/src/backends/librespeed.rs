use std::time::Instant;

use reqwest::blocking::{Body, Client};
use url::Url;

use crate::measurement::{Measurement, Throughput};
use crate::server::Server;
use crate::{Error, MeasurementEngine};

const PING_SAMPLES: usize = 10;
const DOWNLOAD_CHUNK_SIZE: u32 = 4; // 4 MiB from garbage.php
const UPLOAD_BYTES: usize = 2 * 1024 * 1024;
const HTTP_ATTEMPTS: usize = 3;

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

    fn is_retryable(err: &reqwest::Error) -> bool {
        err.is_timeout() || err.is_connect() || err.is_decode() || err.is_body()
    }

    fn with_retries<T>(
        phase: &'static str,
        mut op: impl FnMut() -> Result<T, reqwest::Error>,
    ) -> Result<T, Error> {
        let mut last = None;
        for _ in 0..HTTP_ATTEMPTS {
            match op() {
                Ok(v) => return Ok(v),
                Err(e) if Self::is_retryable(&e) => last = Some(e),
                Err(e) => return Err(Error::from_reqwest(phase, e)),
            }
        }
        Err(Error::from_reqwest(phase, last.expect("retry loop")))
    }

    fn ping_jitter(&self, server: &Server) -> Result<(f64, f64), Error> {
        let url = Self::join(&server.base_url, &server.ping_path)?;
        let mut samples_ms = Vec::with_capacity(PING_SAMPLES);

        for _ in 0..PING_SAMPLES {
            let start = Instant::now();
            Self::with_retries("ping", || {
                let resp = self.client.get(url.clone()).send()?.error_for_status()?;
                let _ = resp.bytes()?;
                Ok(())
            })?;
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

        let (n, secs) = Self::with_retries("download", || {
            let start = Instant::now();
            let resp = self.client.get(url.clone()).send()?.error_for_status()?;
            let n = resp.bytes()?.len();
            Ok((n, start.elapsed().as_secs_f64().max(1e-6)))
        })?;
        Ok(Throughput::from_bps((n as f64) * 8.0 / secs))
    }

    fn upload(&self, server: &Server) -> Result<Throughput, Error> {
        let url = Self::join(&server.base_url, &server.ul_path)?;
        let len = UPLOAD_BYTES as f64;

        let secs = Self::with_retries("upload", || {
            let payload = vec![0_u8; UPLOAD_BYTES];
            let start = Instant::now();
            let resp = self
                .client
                .post(url.clone())
                .header(reqwest::header::CONTENT_TYPE, "application/octet-stream")
                .body(Body::from(payload))
                .send()?
                .error_for_status()?;
            let _ = resp.bytes()?.len();
            Ok(start.elapsed().as_secs_f64().max(1e-6))
        })?;
        Ok(Throughput::from_bps(len * 8.0 / secs))
    }

    pub fn measure_with_failover(
        &self,
        candidates: &[Server],
    ) -> Result<(Server, Measurement), Error> {
        if candidates.is_empty() {
            return Err(Error::Message("no LibreSpeed candidates".into()));
        }
        let mut last_err: Option<Error> = None;
        for (i, server) in candidates.iter().enumerate() {
            match self.measure(server) {
                Ok(m) => return Ok((server.clone(), m)),
                Err(e) => {
                    if i + 1 < candidates.len() {
                        eprintln!("linkprobe: {} failed, trying next server", server.name);
                    }
                    last_err = Some(e);
                }
            }
        }
        Err(last_err.expect("non-empty candidates"))
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

    #[test]
    fn failovers_to_second_server() {
        let mut bad = MockServer::new();
        let mut good = MockServer::new();

        let _ping_bad = bad
            .mock("GET", "/backend/empty.php")
            .with_status(200)
            .with_body("")
            .expect_at_least(PING_SAMPLES)
            .create();
        let _dl_bad = bad
            .mock("GET", "/backend/garbage.php")
            .match_query(mockito::Matcher::UrlEncoded(
                "ckSize".into(),
                DOWNLOAD_CHUNK_SIZE.to_string(),
            ))
            .with_status(500)
            .with_body("nope")
            .create();

        let _ping_good = good
            .mock("GET", "/backend/empty.php")
            .with_status(200)
            .with_body("")
            .expect_at_least(PING_SAMPLES)
            .create();
        let dl_body = vec![1_u8; DOWNLOAD_CHUNK_SIZE as usize * 1024 * 1024];
        let _dl_good = good
            .mock("GET", "/backend/garbage.php")
            .match_query(mockito::Matcher::UrlEncoded(
                "ckSize".into(),
                DOWNLOAD_CHUNK_SIZE.to_string(),
            ))
            .with_status(200)
            .with_body(dl_body)
            .create();
        let _ul_good = good
            .mock("POST", "/backend/empty.php")
            .with_status(200)
            .with_body("")
            .create();

        let engine = LibreSpeedEngine::new().unwrap();
        let a = Server::librespeed(bad.url());
        let b = Server::librespeed(good.url());
        let (used, m) = engine.measure_with_failover(&[a, b.clone()]).unwrap();
        assert_eq!(used.base_url, b.base_url);
        assert!(m.download.unwrap().bps > 0.0);
    }
}
