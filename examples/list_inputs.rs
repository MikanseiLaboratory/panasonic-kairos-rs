//! List inputs on a KAIROS device.
//!
//! ```text
//! KAIROS_HOST=192.168.10.10 KAIROS_PASSWORD=secret cargo run --example list_inputs --features std
//! ```

use panasonic_kairos::{http::Client, Credentials, HttpConfig};

fn main() -> panasonic_kairos::Result<()> {
    let host = std::env::var("KAIROS_HOST").unwrap_or_else(|_| "192.168.10.10".into());
    let password = std::env::var("KAIROS_PASSWORD").unwrap_or_default();
    let client =
        Client::connect(HttpConfig::new(host).with_credentials(Credentials::password(password)))?;

    for input in client.list_inputs()? {
        println!(
            "{:>4}  {:<12} tally={}  {}",
            input.index, input.name, input.tally, input.uuid
        );
    }
    Ok(())
}
