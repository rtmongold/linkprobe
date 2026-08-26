use thiserror::Error;

/// Failure from discovery, measurement, or export helpers.
///
/// Match on this instead of string-matching `Display` output:
///
/// - [`Error::Iperf3Missing`](Self::Iperf3Missing) — `iperf3` binary not found when using
///   [`Iperf3Engine`](crate::backends::Iperf3Engine)
/// - [`Error::Probe`](Self::Probe) — a named measurement phase failed (for example `"download"`);
///   the `source` chain holds the underlying error
/// - [`Error::Http`](Self::Http), [`Error::Io`](Self::Io), [`Error::Json`](Self::Json) —
///   transparent wrappers for reqwest, I/O, and JSON errors
#[derive(Debug, Error)]
pub enum Error {
    /// General failure with a message (unknown server id, invalid CLI combination in the binary crate, etc.).
    #[error("{0}")]
    Message(String),

    #[error("not implemented")]
    NotImplemented,

    /// A measurement phase failed; inspect `phase` and `source`.
    #[error("{phase} failed: {source}")]
    Probe {
        phase: &'static str,
        #[source]
        source: Box<Error>,
    },

    #[error("iperf3 not found on PATH (install iperf3 to use --backend iperf3)")]
    Iperf3Missing,

    /// MQTT publish failure (CLI crate).
    #[error("mqtt: {0}")]
    Mqtt(String),

    #[error(transparent)]
    Http(#[from] reqwest::Error),

    #[error(transparent)]
    Url(#[from] url::ParseError),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

impl Error {
    pub fn probe(phase: &'static str, source: impl Into<Error>) -> Self {
        Error::Probe {
            phase,
            source: Box::new(source.into()),
        }
    }

    pub fn from_reqwest(phase: &'static str, err: reqwest::Error) -> Self {
        let hint = if err.is_timeout() {
            format!("{err} (timed out)")
        } else if err.is_connect() {
            format!("{err} (connection failed)")
        } else if err.is_decode() || err.is_body() {
            format!(
                "{err} (connection closed before the response finished; retry or pick another server)"
            )
        } else {
            err.to_string()
        };
        Error::Probe {
            phase,
            source: Box::new(Error::Message(hint)),
        }
    }
}
