//! Shared request helpers used by blocking and async clients.

use crate::id::ResourceId;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub(crate) const MERGE_PATCH_JSON: &str = "application/merge-patch+json";

pub(crate) fn path_and_query(url: &url::Url) -> String {
    let mut s = url.path().to_string();
    if let Some(q) = url.query() {
        s.push('?');
        s.push_str(q);
    }
    s
}

pub(crate) fn decode_json<T: DeserializeOwned>(bytes: &[u8]) -> crate::Result<T> {
    Ok(serde_json::from_slice(bytes)?)
}

pub(crate) fn encode_patch<T: Serialize>(body: &T) -> crate::Result<Vec<u8>> {
    Ok(serde_json::to_vec(body)?)
}

pub(crate) fn decode_one<T: DeserializeOwned>(bytes: &[u8]) -> crate::Result<T> {
    #[derive(serde::Deserialize)]
    #[serde(untagged)]
    enum OneOrMany<T> {
        One(T),
        Many(Vec<T>),
    }
    match serde_json::from_slice::<OneOrMany<T>>(bytes)? {
        OneOrMany::One(v) => Ok(v),
        OneOrMany::Many(mut v) => v
            .pop()
            .ok_or_else(|| crate::Error::Protocol("expected a single object, got []".into())),
    }
}

pub(crate) fn segment(id: impl Into<ResourceId>) -> String {
    id.into().as_segment()
}
