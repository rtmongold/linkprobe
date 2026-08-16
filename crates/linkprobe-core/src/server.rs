use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub id: String,
    pub name: String,
    /// Base URL of the LibreSpeed (or compatible) instance, e.g. `https://example/`.
    pub base_url: String,
    pub country: Option<String>,
    pub sponsor: Option<String>,
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
    pub fn librespeed(base_url: impl Into<String>) -> Self {
        let base_url = base_url.into();
        Self {
            id: base_url.clone(),
            name: base_url.clone(),
            base_url,
            country: None,
            sponsor: None,
            dl_path: default_dl_path(),
            ul_path: default_ul_path(),
            ping_path: default_ping_path(),
        }
    }
}
