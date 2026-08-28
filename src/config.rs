//! Connection configuration.

use crate::error::Result;
use url::Url;

/// Factory-default IPv4 address.
pub const DEFAULT_HOST: &str = "192.168.10.10";
/// Factory-default REST API port.
pub const DEFAULT_PORT: u16 = 1234;
/// Factory-default Simple Control Protocol port.
pub const DEFAULT_SIMPLE_PORT: u16 = 3005;
/// REST API username (fixed by the vendor spec).
pub const DEFAULT_USERNAME: &str = "Kairos";

/// HTTP Basic / Digest credentials.
///
/// The username is always [`DEFAULT_USERNAME`] unless overridden. The password
/// must be set on the device (Kairos v1.3.2 and later).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Credentials {
    /// Username (default `Kairos`).
    pub username: String,
    /// REST API password configured on the device.
    pub password: String,
}

impl Default for Credentials {
    fn default() -> Self {
        Self {
            username: DEFAULT_USERNAME.into(),
            password: String::new(),
        }
    }
}

impl Credentials {
    /// Credentials with the vendor username and the given password.
    pub fn password(password: impl Into<String>) -> Self {
        Self {
            username: DEFAULT_USERNAME.into(),
            password: password.into(),
        }
    }

    /// Fully custom credentials.
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
        }
    }
}

/// HTTP client configuration.
#[derive(Debug, Clone)]
pub struct HttpConfig {
    /// Device host (`host`, `host:port`, or a full URL).
    pub host: String,
    /// Authentication credentials.
    pub credentials: Credentials,
    /// Use HTTPS instead of HTTP.
    pub https: bool,
    /// Request timeout in milliseconds (`0` = no timeout).
    pub timeout_ms: u64,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self::new(DEFAULT_HOST)
    }
}

impl HttpConfig {
    /// Create config for `host`. Port [`DEFAULT_PORT`] is appended when omitted.
    pub fn new(host: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            credentials: Credentials::default(),
            https: false,
            timeout_ms: 10_000,
        }
    }

    /// Set credentials.
    pub fn with_credentials(mut self, credentials: Credentials) -> Self {
        self.credentials = credentials;
        self
    }

    /// Enable HTTPS.
    pub fn with_https(mut self, https: bool) -> Self {
        self.https = https;
        self
    }

    /// Set timeout in milliseconds.
    pub fn with_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// Base URL `http(s)://{host}:{port}` with no trailing slash.
    pub fn base_url(&self) -> Result<Url> {
        Ok(Url::parse(&normalize_base(&self.host, self.https))?)
    }

    /// Absolute URL for a path that already starts with `/`.
    pub fn url_for_path(&self, path: &str) -> Result<Url> {
        let mut url = self.base_url()?;
        url.set_path(path.trim_end_matches('/'));
        url.set_query(None);
        Ok(url)
    }

    /// Absolute URL built from unencoded path segments.
    pub fn url_for_segments(&self, segments: &[&str]) -> Result<Url> {
        let mut url = self.base_url()?;
        {
            let mut path = url
                .path_segments_mut()
                .map_err(|_| crate::Error::Protocol("cannot-be-a-base URL".into()))?;
            path.clear();
            for segment in segments {
                path.push(segment);
            }
        }
        Ok(url)
    }
}

/// Simple Control Protocol (TCP) client configuration.
#[derive(Debug, Clone)]
pub struct TcpConfig {
    /// Device host name or IPv4/IPv6 address (no scheme).
    pub host: String,
    /// TCP port. Kairos listens on [`DEFAULT_SIMPLE_PORT`] by default.
    pub port: u16,
    /// I/O timeout in milliseconds (`0` = no timeout).
    pub timeout_ms: u64,
}

impl Default for TcpConfig {
    fn default() -> Self {
        Self::new(DEFAULT_HOST)
    }
}

impl TcpConfig {
    /// Config for `host` on [`DEFAULT_SIMPLE_PORT`].
    pub fn new(host: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            port: DEFAULT_SIMPLE_PORT,
            timeout_ms: 10_000,
        }
    }

    /// Override the TCP port.
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Set timeout in milliseconds.
    pub fn with_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// `host:port` suitable for [`std::net::TcpStream::connect`].
    pub fn addr(&self) -> String {
        let host = self.host.trim().trim_matches(|c| c == '[' || c == ']');
        if host.matches(':').count() >= 2 {
            format!("[{host}]:{}", self.port)
        } else {
            format!("{host}:{}", self.port)
        }
    }
}

fn normalize_base(host: &str, https: bool) -> String {
    let trimmed = host.trim().trim_end_matches('/');
    if trimmed.contains("://") {
        return trimmed.to_string();
    }
    let scheme = if https { "https" } else { "http" };
    if has_explicit_port(trimmed) {
        format!("{scheme}://{trimmed}")
    } else {
        format!("{scheme}://{trimmed}:{DEFAULT_PORT}")
    }
}

fn has_explicit_port(host: &str) -> bool {
    if host.starts_with('[') {
        return host.contains("]:");
    }
    match host.rfind(':') {
        Some(i) => host[i + 1..].chars().all(|c| c.is_ascii_digit()) && !host[i + 1..].is_empty(),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_port_is_appended() {
        let url = HttpConfig::new("192.168.10.10").base_url().unwrap();
        assert_eq!(url.as_str(), "http://192.168.10.10:1234/");
    }

    #[test]
    fn explicit_port_is_kept() {
        let url = HttpConfig::new("192.168.10.10:8080").base_url().unwrap();
        assert_eq!(url.as_str(), "http://192.168.10.10:8080/");
    }

    #[test]
    fn segments_are_encoded() {
        let url = HttpConfig::new("192.168.10.10")
            .url_for_segments(&["scenes", "M1- Main"])
            .unwrap();
        assert_eq!(url.path(), "/scenes/M1-%20Main");
        assert!(!url.as_str().ends_with('/'));
    }

    #[test]
    fn simple_control_addr() {
        assert_eq!(TcpConfig::new("192.168.10.10").addr(), "192.168.10.10:3005");
        assert_eq!(TcpConfig::new("core").with_port(4000).addr(), "core:4000");
    }
}
