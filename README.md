# panasonic-kairos-rs

Unofficial Rust HTTP client for the **Panasonic KAIROS REST API**
(Kairos Core AT-KC200 / AT-KC100 / AT-KC2000 / AT-KC1000, spec **v1.7** / firmware **1.7.3**).

This crate is not affiliated with Panasonic Entertainment & Communication Co., Ltd.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

## Features

- `std` (default): blocking client (`ureq`)
- `tokio` (default): async client (`reqwest`)

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

```text
KAIROS_HOST=192.168.10.10 KAIROS_PASSWORD=secret cargo run --example list_inputs
```

## License

MIT. 
