//! Error types for the KAIROS REST client.

use thiserror::Error;

/// Crate-wide result alias.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors returned by the client.
#[derive(Debug, Error)]
pub enum Error {
    /// Invalid host / URL configuration.
    #[error("invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),

    /// HTTP authentication failed (typically 401).
    #[error("authentication failed: {0}")]
    Auth(String),

    /// Unexpected HTTP status code.
    #[error("HTTP {status}: {message}")]
    Http {
        /// Status code.
        status: u16,
        /// Response body or reason.
        message: String,
    },

    /// JSON serialization / deserialization failure.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Protocol framing or unexpected payload.
    #[error("protocol error: {0}")]
    Protocol(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Feature-gated transport is unavailable, or the HTTP stack failed.
    #[error("transport error: {0}")]
    Transport(String),
}

impl Error {
    /// Create an HTTP status error.
    pub fn http(status: u16, message: impl Into<String>) -> Self {
        Self::Http {
            status,
            message: message.into(),
        }
    }
}

#[cfg(feature = "std")]
impl From<ureq::Error> for Error {
    fn from(err: ureq::Error) -> Self {
        match err {
            ureq::Error::Status(code, resp) => {
                let body = resp.into_string().unwrap_or_default();
                if code == 401 {
                    Self::Auth(body)
                } else {
                    Self::http(code, body)
                }
            }
            ureq::Error::Transport(t) => Self::Transport(t.to_string()),
        }
    }
}

#[cfg(feature = "tokio")]
impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        if err.is_status() {
            let status = err.status().map(|s| s.as_u16()).unwrap_or(0);
            if status == 401 {
                Self::Auth(err.to_string())
            } else {
                Self::http(status, err.to_string())
            }
        } else {
            Self::Transport(err.to_string())
        }
    }
}
