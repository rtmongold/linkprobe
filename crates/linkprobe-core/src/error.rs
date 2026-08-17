use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Message(String),

    #[error("not implemented")]
    NotImplemented,

    #[error("{phase} failed: {source}")]
    Probe {
        phase: &'static str,
        #[source]
        source: Box<Error>,
    },

    #[error("iperf3 not found on PATH (install iperf3 to use --backend iperf3)")]
    Iperf3Missing,

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
