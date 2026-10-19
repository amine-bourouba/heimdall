//! Exports to Markdown, patches and CSV.

use serde::Deserialize;
use specta::Type;

use crate::error::{AppError, Result};

pub mod analytics_csv;
pub mod patch;
pub mod session_markdown;

/// The 8-byte signature every real PNG file starts with (`\x89PNG\r\n\x1a\n`) -- `save_png`'s own
/// AC ("rejects bytes without the PNG signature"): the webview supplies these bytes itself (a
/// canvas's own `toBlob`/`toDataURL` output), so this is the one place they are validated before
/// ever reaching a save dialog and the filesystem, the same "never trust webview-supplied content
/// blindly" principle ADR 0010 applies to paths.
const PNG_SIGNATURE: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

pub fn validate_png(bytes: &[u8]) -> Result<()> {
    if bytes.starts_with(&PNG_SIGNATURE) {
        Ok(())
    } else {
        Err(AppError::invalid_input("not a PNG file (missing signature)"))
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_png_accepts_the_real_signature_plus_arbitrary_data() {
        let mut bytes = PNG_SIGNATURE.to_vec();
        bytes.extend_from_slice(b"whatever comes after is not this function's job to check");
        assert!(validate_png(&bytes).is_ok());
    }

    #[test]
    fn validate_png_rejects_anything_else() {
        for bad in [b"".as_slice(), b"\x89PNG", b"GIF89a", b"not a png at all"] {
            assert!(matches!(validate_png(bad).unwrap_err(), AppError::InvalidInput { .. }));
        }
    }
}
