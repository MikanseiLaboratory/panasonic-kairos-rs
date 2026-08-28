//! Unofficial client for Panasonic KAIROS.
//!
//! Covers the REST API (GET / PATCH) documented for Kairos Core
//! (AT-KC200, AT-KC100, AT-KC2000, AT-KC1000) version **1.7.3**,
//! and the Simple Control Protocol (TCP port 3005).
//!
//! PUT, POST, and DELETE are not part of the REST protocol and must not be sent.
//! Trailing slashes are rejected by the device (`404 Invalid url`).
//!
//! # Features
//!
//! - `std` (default): blocking HTTP (`ureq`) and blocking Simple Control (`std::net`)
//! - `tokio` (default): async HTTP (`reqwest`) and async Simple Control (`tokio::net`)
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
pub mod simple;
pub mod types;

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
pub mod http;

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
pub mod tcp;

#[cfg(feature = "tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
pub mod http_async;

#[cfg(feature = "tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
pub mod tcp_async;

mod transport;

pub use config::{
    Credentials, HttpConfig, TcpConfig, DEFAULT_HOST, DEFAULT_PORT, DEFAULT_SIMPLE_PORT,
    DEFAULT_USERNAME,
};
pub use error::{Error, Result};
pub use id::ResourceId;
pub use types::*;
