# panasonic-kairos-rs

Unofficial Rust client for the **Panasonic KAIROS REST API** and **Simple Control Protocol**
(Kairos Core AT-KC200 / AT-KC100 / AT-KC2000 / AT-KC1000, spec **v1.7** / firmware **1.7.3**).

This crate is not affiliated with Panasonic Entertainment & Communication Co., Ltd.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

## Features

- `std` (default): blocking REST (`ureq`) and blocking Simple Control (`std::net`, port 3005)
- `tokio` (default): async REST (`reqwest`) and async Simple Control (`tokio::net`)

## Usage

```toml
[dependencies]
panasonic-kairos = "0.1"
```

```rust
use panasonic_kairos::{http::Client, Credentials, HttpConfig};

let client = Client::connect(
    HttpConfig::new("192.168.10.10").with_credentials(Credentials::password("secret")),
)?;

let inputs = client.list_inputs()?;
client.set_layer_source_a("Main", "Background", "IP1")?;
client.play_macro("GM-1")?;
client.recall_snapshot("Main", "SNP1Main")?;
```

Async:

```rust
use panasonic_kairos::{http_async::Client, Credentials, HttpConfig};

let client = Client::connect(
    HttpConfig::new("192.168.10.10").with_credentials(Credentials::password("secret")),
)?;
let scenes = client.list_scenes().await?;
```

Simple Control (TCP :3005):

```rust
use panasonic_kairos::{tcp_async::Client, simple::SOURCE_A, TcpConfig};

let mut tcp = Client::connect(TcpConfig::new("192.168.10.10")).await?;
tcp.force_source("Main", "Background", SOURCE_A, "IP1").await?;
tcp.player("RR1", "play").await?;
tcp.keep_alive().await?;
```

```text
KAIROS_HOST=192.168.10.10 KAIROS_PASSWORD=secret cargo run --example list_inputs
```

## License

MIT. 
