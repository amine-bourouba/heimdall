//! Output styles (Models/Config/OutputStyle.swift).

use serde::Serialize;
use specta::Type;

use super::scope::ConfigScope;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct OutputStyle {
    pub id: String,
    pub name: String,
    pub description: String,
    pub filename: String,
    pub body: String,
    /// Display only; never sent back by the webview (ADR 0010).
    pub file_path: String,
    pub scope: ConfigScope,
}
