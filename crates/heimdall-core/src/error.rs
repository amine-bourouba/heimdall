//! The one error type every command returns, plus the loader and write wrappers
//! (docs/ipc-contract.md §3, ADR 0005).

use serde::Serialize;
use specta::Type;

/// Tagged union serialised as `{"kind": "...", "message": "...", "path": "..." | null}`.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum AppError {
    #[error("not found: {message}")]
    NotFound { message: String, path: Option<String> },
    #[error("i/o error: {message}")]
    Io { message: String, path: Option<String> },
    #[error("parse error: {message}")]
    Parse { message: String, path: Option<String> },
    #[error("permission denied: {message}")]
    Permission { message: String, path: Option<String> },
    /// A file stayed locked by another process after bounded retries (Windows sharing violation).
    #[error("busy: {message}")]
    Busy { message: String, path: Option<String> },
    #[error("invalid input: {message}")]
    InvalidInput { message: String, path: Option<String> },
}

pub type Result<T> = std::result::Result<T, AppError>;

impl AppError {
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound {
            message: message.into(),
            path: None,
        }
    }
    pub fn io(message: impl Into<String>) -> Self {
        Self::Io {
            message: message.into(),
            path: None,
        }
    }
    pub fn parse(message: impl Into<String>) -> Self {
        Self::Parse {
            message: message.into(),
            path: None,
        }
    }
    pub fn permission(message: impl Into<String>) -> Self {
        Self::Permission {
            message: message.into(),
            path: None,
        }
    }
    pub fn busy(message: impl Into<String>) -> Self {
        Self::Busy {
            message: message.into(),
            path: None,
        }
    }
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput {
            message: message.into(),
            path: None,
        }
    }

    /// A command whose owning task has not landed yet (M1-37 scaffolding); every real task
    /// replaces the body that returns this before it ships.
    pub fn not_implemented(command: &str) -> Self {
        Self::invalid_input(format!("not implemented: {command}"))
    }

    /// Attach the path the error is about. Paths are shown to the user; they are never secrets.
    pub fn with_path(mut self, p: impl AsRef<std::path::Path>) -> Self {
        let value = Some(p.as_ref().display().to_string());
        match &mut self {
            Self::NotFound { path, .. }
            | Self::Io { path, .. }
            | Self::Parse { path, .. }
            | Self::Permission { path, .. }
            | Self::Busy { path, .. }
            | Self::InvalidInput { path, .. } => *path = value,
        }
        self
    }

    /// Map an I/O error for `path`: missing → notFound, denied → permission,
    /// Windows sharing and lock violations (os errors 32 and 33) → busy, anything else → io.
    pub fn from_io(err: &std::io::Error, path: impl AsRef<std::path::Path>) -> Self {
        let message = err.to_string();
        let base = match (err.kind(), err.raw_os_error()) {
            (std::io::ErrorKind::NotFound, _) => Self::NotFound { message, path: None },
            (std::io::ErrorKind::PermissionDenied, _) => Self::Permission { message, path: None },
            (_, Some(32 | 33)) if cfg!(windows) => Self::Busy { message, path: None },
            _ => Self::Io { message, path: None },
        };
        base.with_path(path)
    }
}

/// A loader result: the value plus how many lines or files were skipped because they did not parse.
#[derive(Debug, Clone, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Loaded<T> {
    pub value: T,
    pub skipped: u32,
}

impl<T> Loaded<T> {
    pub fn new(value: T, skipped: u32) -> Self {
        Self { value, skipped }
    }
}

/// Every mutating command returns this. `cancelled` means the user declined a native dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum WriteOutcome {
    Written,
    Cancelled,
    Unchanged,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn every_variant_serialises_to_the_contract_shape() {
        let cases = [
            (AppError::not_found("m"), "notFound"),
            (AppError::io("m"), "io"),
            (AppError::parse("m"), "parse"),
            (AppError::permission("m"), "permission"),
            (AppError::busy("m"), "busy"),
            (AppError::invalid_input("m"), "invalidInput"),
        ];
        for (err, kind) in cases {
            assert_eq!(
                serde_json::to_value(&err).unwrap(),
                json!({"kind": kind, "message": "m", "path": null})
            );
        }
    }

    #[test]
    fn with_path_sets_the_path_field() {
        let err = AppError::parse("bad json").with_path("/Users/fixture/.claude/settings.json");
        assert_eq!(
            serde_json::to_value(&err).unwrap(),
            json!({"kind": "parse", "message": "bad json", "path": "/Users/fixture/.claude/settings.json"})
        );
    }

    #[test]
    fn from_io_maps_error_kinds() {
        let p = "/x";
        let nf = std::io::Error::new(std::io::ErrorKind::NotFound, "gone");
        assert!(matches!(AppError::from_io(&nf, p), AppError::NotFound { .. }));
        let pd = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "no");
        assert!(matches!(AppError::from_io(&pd, p), AppError::Permission { .. }));
        let sharing = std::io::Error::from_raw_os_error(32);
        let expected_busy = cfg!(windows);
        assert_eq!(
            matches!(AppError::from_io(&sharing, p), AppError::Busy { .. }),
            expected_busy
        );
    }

    #[test]
    fn loaded_and_write_outcome_shapes() {
        assert_eq!(
            serde_json::to_value(Loaded::new(vec![1, 2], 3)).unwrap(),
            json!({"value": [1, 2], "skipped": 3})
        );
        assert_eq!(serde_json::to_value(WriteOutcome::Written).unwrap(), json!("written"));
        assert_eq!(
            serde_json::to_value(WriteOutcome::Cancelled).unwrap(),
            json!("cancelled")
        );
        assert_eq!(
            serde_json::to_value(WriteOutcome::Unchanged).unwrap(),
            json!("unchanged")
        );
    }
}
