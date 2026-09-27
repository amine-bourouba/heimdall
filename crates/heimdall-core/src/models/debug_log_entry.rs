//! One entry of a `~/.claude/debug/<sessionId>.txt` log (Models/DebugLogEntry.swift).

use chrono::{DateTime, Utc};
use serde::Serialize;
use specta::Type;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DebugLogEntry {
    /// Position in the file (Identifiable uses the index).
    pub id: u32,
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
}

/// Written as `DEBUG`, `WARN`, `ERROR` in the log; sent as `debug`, `warn`, `error` over IPC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum LogLevel {
    Debug,
    Warn,
    Error,
}

impl LogLevel {
    pub const ALL: [LogLevel; 3] = [LogLevel::Debug, LogLevel::Warn, LogLevel::Error];

    pub const fn raw_value(self) -> &'static str {
        match self {
            Self::Debug => "DEBUG",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
        }
    }

    pub const fn label(self) -> &'static str {
        self.raw_value()
    }

    pub fn from_raw(raw: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|l| l.raw_value() == raw)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: u32, message: &str) -> DebugLogEntry {
        DebugLogEntry {
            id,
            timestamp: DateTime::UNIX_EPOCH,
            level: LogLevel::Debug,
            message: message.into(),
        }
    }

    // Ported from PoirotTests/Models/DebugLogEntryTests.swift.
    #[test]
    fn level_raw_values() {
        assert_eq!(LogLevel::ALL.map(LogLevel::raw_value), ["DEBUG", "WARN", "ERROR"]);
        assert_eq!(LogLevel::from_raw("WARN"), Some(LogLevel::Warn));
    }
    #[test]
    fn level_labels() {
        assert_eq!(LogLevel::ALL.map(LogLevel::label), ["DEBUG", "WARN", "ERROR"]);
    }
    #[test]
    fn all_cases_contains_three_levels() {
        assert_eq!(LogLevel::ALL.len(), 3);
    }
    #[test]
    fn identifiable_uses_index() {
        assert_eq!(entry(42, "Test").id, 42);
    }
    #[test]
    fn hashable_same_id_are_equal() {
        assert_eq!(entry(1, "A"), entry(1, "A"));
    }
    #[test]
    fn hashable_different_id_are_not_equal() {
        assert_ne!(entry(1, "A"), entry(2, "A"));
    }

    #[test]
    fn level_serialises_lower_case_for_ipc() {
        assert_eq!(serde_json::to_value(LogLevel::Error).unwrap(), "error");
    }
}
