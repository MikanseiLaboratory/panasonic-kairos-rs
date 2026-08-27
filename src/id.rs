//! Resource identifiers (name, index, or UUID).

use std::fmt;

/// Identifies a KAIROS object in a request path.
///
/// The vendor spec allows name, numeric index (where documented), or UUID.
/// UUID is the recommended form because scene names are user-defined and may collide.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResourceId {
    /// User-visible name (`Main`, `IP1`, `Multiviewer1`, …).
    Name(String),
    /// Numeric index (`/aux/0`, `/multiviewers/1`, `/inputs/128`, …).
    Index(u32),
    /// Object UUID.
    Uuid(String),
}

impl ResourceId {
    /// Name identifier.
    pub fn name(value: impl Into<String>) -> Self {
        Self::Name(value.into())
    }

    /// Numeric index identifier.
    pub fn index(value: u32) -> Self {
        Self::Index(value)
    }

    /// UUID identifier.
    pub fn uuid(value: impl Into<String>) -> Self {
        Self::Uuid(value.into())
    }

    /// Path segment string (unencoded).
    pub fn as_segment(&self) -> String {
        match self {
            Self::Name(v) | Self::Uuid(v) => v.clone(),
            Self::Index(v) => v.to_string(),
        }
    }
}

impl fmt::Display for ResourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.as_segment())
    }
}

impl From<u32> for ResourceId {
    fn from(value: u32) -> Self {
        Self::Index(value)
    }
}

impl From<&str> for ResourceId {
    fn from(value: &str) -> Self {
        if looks_like_uuid(value) {
            Self::Uuid(value.to_string())
        } else {
            Self::Name(value.to_string())
        }
    }
}

impl From<String> for ResourceId {
    fn from(value: String) -> Self {
        ResourceId::from(value.as_str())
    }
}

fn looks_like_uuid(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 36
        && bytes[8] == b'-'
        && bytes[13] == b'-'
        && bytes[18] == b'-'
        && bytes[23] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(i, c)| matches!(i, 8 | 13 | 18 | 23) || c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uuid_from_str() {
        let id = ResourceId::from("e53210f7-2235-5ae3-9c02-4f58d67bf8b8");
        assert!(matches!(id, ResourceId::Uuid(_)));
    }

    #[test]
    fn name_from_str() {
        let id = ResourceId::from("Main");
        assert_eq!(id, ResourceId::Name("Main".into()));
    }
}
