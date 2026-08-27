//! HTTP Digest authentication helpers.

use crate::config::Credentials;
use crate::error::{Error, Result};

/// Parse a `WWW-Authenticate` Digest challenge and compute an Authorization header value
/// (without the `Authorization: ` prefix).
pub fn authorization_header(
    credentials: &Credentials,
    method: &str,
    uri: &str,
    www_authenticate: &str,
) -> Result<String> {
    let mut prompt = digest_auth::parse(www_authenticate)
        .map_err(|e| Error::Auth(format!("invalid digest challenge: {e}")))?;

    let context = digest_auth::AuthContext::new_with_method(
        &credentials.username,
        &credentials.password,
        uri,
        None::<&[u8]>,
        method_from_str(method),
    );

    let answer = prompt
        .respond(&context)
        .map_err(|e| Error::Auth(format!("digest respond failed: {e}")))?;

    Ok(answer.to_string())
}

fn method_from_str(method: &str) -> digest_auth::HttpMethod<'static> {
    match method.to_ascii_uppercase().as_str() {
        "POST" => digest_auth::HttpMethod::POST,
        "PUT" => digest_auth::HttpMethod::PUT,
        "DELETE" => digest_auth::HttpMethod::DELETE,
        "HEAD" => digest_auth::HttpMethod::HEAD,
        "OPTIONS" => digest_auth::HttpMethod::OPTIONS,
        "PATCH" => digest_auth::HttpMethod::PATCH,
        _ => digest_auth::HttpMethod::GET,
    }
}

/// Returns true if the header looks like a Digest challenge.
pub fn is_digest_challenge(www_authenticate: &str) -> bool {
    www_authenticate
        .trim()
        .to_ascii_lowercase()
        .starts_with("digest")
}

/// RFC 7617 Basic credential (the value after `Basic `).
pub fn basic_header(credentials: &Credentials) -> String {
    use std::fmt::Write as _;
    let token = simple_base64(&format!(
        "{}:{}",
        credentials.username, credentials.password
    ));
    let mut value = String::from("Basic ");
    let _ = write!(value, "{token}");
    value
}

fn simple_base64(input: &str) -> String {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes = input.as_bytes();
    let mut out = String::new();
    let mut i = 0;
    while i < bytes.len() {
        let b0 = bytes[i];
        let b1 = if i + 1 < bytes.len() { bytes[i + 1] } else { 0 };
        let b2 = if i + 2 < bytes.len() { bytes[i + 2] } else { 0 };
        let triple = ((b0 as u32) << 16) | ((b1 as u32) << 8) | (b2 as u32);
        out.push(TABLE[((triple >> 18) & 0x3F) as usize] as char);
        out.push(TABLE[((triple >> 12) & 0x3F) as usize] as char);
        if i + 1 < bytes.len() {
            out.push(TABLE[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
        if i + 2 < bytes.len() {
            out.push(TABLE[(triple & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
        i += 3;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_header_matches_known_value() {
        let creds = Credentials::new("Kairos", "password");
        assert_eq!(basic_header(&creds), "Basic S2Fpcm9zOnBhc3N3b3Jk");
    }
}
