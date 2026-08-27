//! Unofficial HTTP client for the Panasonic KAIROS REST API.
//!
//! Covers the GET / PATCH surface documented for Kairos Core
//! (AT-KC200, AT-KC100, AT-KC2000, AT-KC1000) version **1.7.3**.
//!
//! PUT, POST, and DELETE are not part of the protocol and must not be sent.
//! Trailing slashes are rejected by the device (`404 Invalid url`).
//!
//! # Features
//!
//! - `std` (default): blocking HTTP (`ureq`) with Basic / Digest auth
//! - `tokio` (default): async HTTP (`reqwest`) with Basic / Digest auth
//!
//! # Example
//!
//! ```no_run
//! # #[cfg(feature = "std")]
//! # fn main() -> panasonic_kairos::Result<()> {
//! use panasonic_kairos::{http::Client, Credentials, HttpConfig};
//!
//! let client = Client::connect(
//!     HttpConfig::new("192.168.10.10").with_credentials(Credentials::password("secret")),
//! )?;
//! for input in client.list_inputs()? {
//!     println!("{} tally={}", input.name, input.tally);
//! }
//! # Ok(())
//! # }
//! # #[cfg(not(feature = "std"))]
//! # fn main() {}
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]

pub mod auth;
pub mod config;
pub mod error;
pub mod id;
pub mod types;

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
pub mod http;

#[cfg(feature = "tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
pub mod http_async;

mod transport;

pub use config::{Credentials, HttpConfig, DEFAULT_HOST, DEFAULT_PORT, DEFAULT_USERNAME};
pub use error::{Error, Result};
pub use id::ResourceId;
pub use types::*;
