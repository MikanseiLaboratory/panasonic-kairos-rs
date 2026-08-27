# panasonic-kairos-rs

Unofficial Rust HTTP client for the **Panasonic KAIROS REST API**
(Kairos Core AT-KC200 / AT-KC100 / AT-KC2000 / AT-KC1000, spec **v1.7** / firmware **1.7.3**).

This crate is not affiliated with Panasonic Entertainment & Communication Co., Ltd.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

## Why this layout

Vendor docs ship as a PDF. Feeding that PDF straight to an LLM is a poor source of truth:
tables wrap, JSON examples are truncated with `…`, and a one-off chat has no schema to regress against.

The durable approach used here:

1. Read the PDF once.
2. Distill it into OpenAPI — [`openapi/kairos-rest-v1.7.yaml`](openapi/kairos-rest-v1.7.yaml).
3. Implement the client from that file, not from the PDF.
4. Keep the original PDF **out** of the repository (vendor copyright).

Later model sessions should be pointed at the YAML (and this crate), not the PDF.

## Protocol notes

- GET and PATCH only. PUT / POST / DELETE return `400`.
- Trailing `/` is rejected (`404 Invalid url`).
- PATCH is RFC 7396 `application/merge-patch+json`.
- Auth: HTTP Basic and Digest. Username is `Kairos`. Password is set on the device (v1.3.2+).
- Factory default: `http://192.168.10.10:1234`.
- Address objects by **UUID** when names can collide (scenes).

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

## Scope

Implemented against spec v1.7:

| Area | GET | PATCH |
| --- | --- | --- |
| Inputs | `/inputs`, `/inputs/{id}` | — |
| Macros | `/macros`, `/macros/{id}` | `state: play` |
| AUX | `/aux`, `/aux/{id}` | `source` |
| Multiviewers | `/multiviewers`, `/multiviewers/{id}`, `/sdp` | `preset` |
| Scenes | `/scenes`, `/scenes/{id}` | layer `sourceA`/`sourceB`, action `play`, scene macro `play`, snapshot `recall` |

## License

MIT. KAIROS is a trademark of Panasonic.
