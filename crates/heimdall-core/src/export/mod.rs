//! Exports to Markdown, patches and CSV.

use serde::Deserialize;
use specta::Type;

pub mod analytics_csv;
pub mod patch;
pub mod session_markdown;

/// Where an export goes (contract `ExportDestination`): the clipboard, or a file the command opens
/// a save dialog for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ExportDestination {
    Clipboard,
    File,
}

/// Which analytics table `export_analytics_csv` writes (contract `CsvKind`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum CsvKind {
    DailyActivity,
    ModelUsage,
    DailyTokens,
    HourlyActivity,
}

/// Which chart `save_png` writes (contract `PngKind`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum PngKind {
    Analytics,
    EditDiff,
}
