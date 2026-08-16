use std::time::Instant;

use reqwest::blocking::Client;
use serde::Deserialize;
use url::Url;

use crate::Error;
use crate::server::Server;

pub const DEFAULT_LIBRESPEED_SERVERS_URL: &str =
    "https://librespeed.org/backend-servers/servers.php";

#[derive(Debug, Clone, Deserialize)]
pub struct LibreSpeedListEntry {
    pub id: Option<u64>,
    pub name: String,
    pub server: String,
    #[serde(rename = "dlURL")]
    pub dl_url: String,
    #[serde(rename = "ulURL")]
    pub ul_url: String,
    #[serde(rename = "pingURL")]
    pub ping_url: String,
    #[serde(rename = "sponsorName")]
    pub sponsor_name: Option<String>,
}

impl LibreSpeedListEntry {
    pub fn into_server(self) -> Server {
        let base_url = normalize_base_url(&self.server);
        let id = self
            .id
            .map(|i| i.to_string())
            .unwrap_or_else(|| base_url.clone());
        Server {
            id,
            name: self.name,
            base_url,
            country: None,
            sponsor: self.sponsor_name,
            port: None,
            dl_path: self.dl_url,
            ul_path: self.ul_url,
            ping_path: self.ping_url,
        }
    }
}

fn normalize_base_url(raw: &str) -> String {
    let s = if let Some(rest) = raw.strip_prefix("//") {
        format!("https://{rest}")
    } else {
        raw.to_string()
    };
    if s.ends_with('/') { s } else { format!("{s}/") }
}

fn join(base: &str, path: &str) -> Result<Url, Error> {
    let base = if base.ends_with('/') {
        base.to_string()
    } else {
        format!("{base}/")
    };
    Ok(Url::parse(&base)?.join(path)?)
}

pub fn parse_librespeed_servers(json: &str) -> Result<Vec<Server>, Error> {
    let entries: Vec<LibreSpeedListEntry> = serde_json::from_str(json)?;
    Ok(entries.into_iter().map(|e| e.into_server()).collect())
}

pub fn fetch_librespeed_servers(client: &Client, list_url: &str) -> Result<Vec<Server>, Error> {
    let text = client.get(list_url).send()?.error_for_status()?.text()?;
    parse_librespeed_servers(&text)
}

/// One GET to ping_path; returns RTT in ms.
pub fn ping_ms(client: &Client, server: &Server) -> Result<f64, Error> {
    let url = join(&server.base_url, &server.ping_path)?;
    let start = Instant::now();
    let resp = client.get(url).send()?.error_for_status()?;
    let _ = resp.bytes()?;
    Ok(start.elapsed().as_secs_f64() * 1000.0)
}

pub fn pick_lowest_latency(client: &Client, servers: &[Server]) -> Result<(Server, f64), Error> {
    let mut best: Option<(Server, f64)> = None;
    for s in servers {
        match ping_ms(client, s) {
            Ok(ms) => {
                if best.as_ref().map(|(_, b)| ms < *b).unwrap_or(true) {
                    best = Some((s.clone(), ms));
                }
            }
            Err(_) => continue,
        }
    }
    best.ok_or_else(|| Error::Message("no LibreSpeed servers responded to ping".into()))
}

pub fn server_by_id(servers: &[Server], id: u64) -> Result<Server, Error> {
    let key = id.to_string();
    servers
        .iter()
        .find(|s| s.id == key)
        .cloned()
        .ok_or_else(|| Error::Message(format!("no LibreSpeed server with id {id} found")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_fixture_list() {
        let json = include_str!("../tests/fixtures/librespeed/servers.json");
        let servers = parse_librespeed_servers(json).unwrap();
        assert_eq!(servers.len(), 2);
        assert_eq!(servers[0].id, "52");
        assert!(servers[0].base_url.starts_with("https://"));
        assert_eq!(
            servers[0].base_url,
            ("https://nyc.speedtest.clouvider.net/backend/")
        ); // no leftover //
        assert_eq!(servers[0].dl_path, "garbage.php");
        assert_eq!(servers[1].ping_path, "backend/empty.php");
    }

    #[test]
    fn normalizes_protocol_relative() {
        let json = r#"[{
            "id": 1,
            "name": "Test",
            "server": "//example.com/backend",
            "dlURL": "garbage.php",
            "ulURL": "empty.php",
            "pingURL": "empty.php"
        }]"#;
        let servers = parse_librespeed_servers(json).unwrap();
        assert_eq!(servers[0].base_url, "https://example.com/backend/");
    }
}
