use std::time::Instant;

use reqwest::blocking::Client;
use serde::Deserialize;
use url::Url;

use crate::Error;
use crate::server::Server;

pub const DEFAULT_LIBRESPEED_SERVERS_URL: &str =
    "https://librespeed.org/backend-servers/servers.php";

pub const DEFAULT_IPERF3_SERVERS_URL: &str =
    "https://export.iperf3serverlist.net/listed_iperf3_servers.json";

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

#[derive(Debug, Clone, Deserialize)]
pub struct Iperf3ListEntry {
    #[serde(rename = "IP/HOST")]
    pub host: String,
    #[serde(rename = "PORT")]
    pub port: String,
    #[serde(rename = "CONTINENT")]
    #[allow(dead_code)]
    pub continent: Option<String>,
    #[serde(rename = "COUNTRY")]
    pub country: Option<String>,
    #[serde(rename = "SITE")]
    pub site: Option<String>,
    #[serde(rename = "PROVIDER")]
    pub provider: Option<String>,
}

fn iperf3_display_name(entry: &Iperf3ListEntry, port: u16) -> String {
    let site = entry
        .site
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or(&entry.host);
    let country = entry
        .country
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or("??");
    let provider = entry
        .provider
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or("unknown");
    format!("{site}, {country} ({provider}) :{port}")
}

pub fn parse_port_range(raw: &str) -> Result<Vec<u16>, Error> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err(Error::Message("empty iperf3 port".into()));
    }
    if let Some((start, end)) = raw.split_once('-') {
        let start: u16 = start
            .trim()
            .parse()
            .map_err(|_| Error::Message(format!("invalid iperf3 port range start: {raw}")))?;
        let end: u16 = end
            .trim()
            .parse()
            .map_err(|_| Error::Message(format!("invalid iperf3 port range end: {raw}")))?;
        if start > end {
            return Err(Error::Message(format!(
                "invalid iperf3 port range: (start > end): {raw}"
            )));
        }
        return Ok((start..=end).collect());
    }
    let port: u16 = raw
        .parse()
        .map_err(|_| Error::Message(format!("invalid iperf3 port: {raw}")))?;
    Ok(vec![port])
}

pub fn parse_iperf3_servers(json: &str) -> Result<Vec<Server>, Error> {
    let entries: Vec<Iperf3ListEntry> = serde_json::from_str(json)?;
    let mut servers = Vec::new();
    let mut next_id = 1_u64;
    for entry in entries {
        for port in parse_port_range(&entry.port)? {
            let id = next_id.to_string();
            next_id += 1;
            servers.push(Server {
                id,
                name: iperf3_display_name(&entry, port),
                base_url: entry.host.clone(),
                country: entry.country.clone(),
                sponsor: entry.provider.clone(),
                port: Some(port),
                ..Server::iperf3(&entry.host, port)
            });
        }
    }
    Ok(servers)
}

pub fn fetch_iperf3_servers(client: &Client, list_url: &str) -> Result<Vec<Server>, Error> {
    let text = client.get(list_url).send()?.error_for_status()?.text()?;
    parse_iperf3_servers(&text)
}

pub fn servers_list_url(backend_is_iperf3: bool, servers_url: &str) -> &str {
    if backend_is_iperf3 && servers_url == DEFAULT_LIBRESPEED_SERVERS_URL {
        DEFAULT_IPERF3_SERVERS_URL
    } else {
        servers_url
    }
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
        .ok_or_else(|| Error::Message(format!("no server with id {id} found")))
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

    #[test]
    fn parses_iperf3_fixture_list() {
        let json = include_str!("../tests/fixtures/iperf3/servers.json");
        let servers = parse_iperf3_servers(json).unwrap();
        assert_eq!(servers.len(), 4);
        assert_eq!(servers[0].id, "1");
        assert_eq!(servers[0].base_url, "41.110.39.130");
        assert_eq!(servers[0].port, Some(5201));
        assert_eq!(servers[0].name, "Algiers, DZ (DATAPACKET) :5201");
        assert_eq!(servers[3].port, Some(5203));
        let picked = server_by_id(&servers, 3).unwrap();
        assert_eq!(picked.base_url, "105.235.237.2");
        assert_eq!(picked.port, Some(5202));
    }

    #[test]
    fn parse_port_range_single_and_span() {
        assert_eq!(parse_port_range("5201").unwrap(), vec![5201]);
        assert_eq!(
            parse_port_range("5201-5203").unwrap(),
            vec![5201, 5202, 5203]
        );
        assert!(parse_port_range("5203-5201").is_err());
    }

    #[test]
    fn servers_list_url_switches_for_iperf3() {
        assert_eq!(
            servers_list_url(true, DEFAULT_LIBRESPEED_SERVERS_URL),
            DEFAULT_IPERF3_SERVERS_URL
        );
        assert_eq!(
            servers_list_url(true, "https://example/list.json"),
            "https://example/list.json"
        );
    }
}
