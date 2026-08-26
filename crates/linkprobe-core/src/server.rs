use serde::{Deserialize, Serialize};

/// Target server for a link measurement.
///
/// For LibreSpeed, `base_url` is the instance root (for example `https://host/backend/`).
/// Path fields (`dl_path`, `ul_path`, `ping_path`) are joined relative to that base.
///
/// For iperf3, `base_url` holds the host name or address and [`port`](Self::port) holds the
/// port (default 5201 when `None`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    /// List id or host-derived identifier.
    pub id: String,
    /// Human-readable label (shown in CLI output and Prometheus labels).
    pub name: String,
    /// LibreSpeed base URL or iperf3 host, depending on the backend.
    pub base_url: String,
    /// Country code from a server list, when present.
    pub country: Option<String>,
    /// Sponsor or provider name from a server list, when present.
    pub sponsor: Option<String>,
    /// iperf3 port (`None` for LibreSpeed servers).
    pub port: Option<u16>,
    /// Relative download path (default LibreSpeed: `backend/garbage.php`).
    #[serde(default = "default_dl_path")]
    pub dl_path: String,
    /// Relative upload path (default: `backend/empty.php`).
    #[serde(default = "default_ul_path")]
    pub ul_path: String,
    /// Relative ping path (default: `backend/empty.php`).
    #[serde(default = "default_ping_path")]
    pub ping_path: String,
}

fn default_dl_path() -> String {
    "backend/garbage.php".into()
}

fn default_ul_path() -> String {
    "backend/empty.php".into()
}

fn default_ping_path() -> String {
    "backend/empty.php".into()
}

impl Server {
    /// Build a LibreSpeed target with default relative paths.
    pub fn librespeed(base_url: impl Into<String>) -> Self {
        let base_url = base_url.into();
        Self {
            id: base_url.clone(),
            name: base_url.clone(),
            base_url,
            country: None,
            sponsor: None,
            port: None,
            dl_path: default_dl_path(),
            ul_path: default_ul_path(),
            ping_path: default_ping_path(),
        }
    }

    /// Build an iperf3 target (`base_url` is the host; `port` is stored on the server).
    pub fn iperf3(host: impl Into<String>, port: u16) -> Self {
        let host = host.into();
        Self {
            id: host.clone(),
            name: format!("{host}:{port}"),
            base_url: host,
            country: None,
            sponsor: None,
            port: Some(port),
            dl_path: default_dl_path(),
            ul_path: default_ul_path(),
            ping_path: default_ping_path(),
        }
    }
}
