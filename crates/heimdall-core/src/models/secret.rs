//! A redacting wrapper for MCP `env` and `headers` values (ADR 0010, control 8).
//!
//! `Secret` deserialises from a JSON string but has no `Serialize` impl, so a secret can never be
//! placed in an IPC response by accident; `Debug` and `Display` print `[redacted]`. Code that must
//! write the real value back to disk (config writes) calls [`Secret::expose`].

use std::fmt;

use serde::{Deserialize, Deserializer};

#[derive(Clone, PartialEq, Eq, Default)]
pub struct Secret(String);

impl Secret {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// The real value. Only config writers may call this.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[redacted]")
    }
}

impl fmt::Display for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[redacted]")
    }
}

impl<'de> Deserialize<'de> for Secret {
    /// Strings are kept as they are; other JSON values (numbers, booleans) are kept as their JSON text.
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(match serde_json::Value::deserialize(deserializer)? {
            serde_json::Value::String(s) => Self(s),
            other => Self(other.to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn never_prints_its_value() {
        let s = Secret::new("sk-123");
        assert_eq!(format!("{s:?} {s}"), "[redacted] [redacted]");
        assert_eq!(s.expose(), "sk-123");
    }

    #[test]
    fn deserialises_strings_and_other_json_values() {
        let v: std::collections::BTreeMap<String, Secret> = serde_json::from_str(r#"{"A": "x", "B": 3}"#).unwrap();
        assert_eq!((v["A"].expose(), v["B"].expose()), ("x", "3"));
    }
}
