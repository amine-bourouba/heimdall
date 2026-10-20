//! Typed preferences with per-OS defaults (ADR 0007), plus the editor and terminal choices
//! (Models/PreferredEditor.swift, Models/PreferredTerminal.swift, ADR 0003).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::paths::TargetOs;

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum Layout {
    #[default]
    Grid,
    List,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum SearchMode {
    #[default]
    Fuzzy,
    Exact,
}

/// The 20 theme ids of PoirotTheme.swift:169-192.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ColorThemeId {
    #[default]
    Default,
    Solarized,
    HighContrast,
    Charcoal,
    TealNight,
    White,
    Pastel,
    GreenTheme,
    Nightshade,
    Fjord,
    FjordLight,
    Harbor,
    Mocha,
    Rosewood,
    Ember,
    Parchment,
    DeepSea,
    Daybreak,
    Citrus,
    Evergreen,
}

impl ColorThemeId {
    pub const ALL: [Self; 20] = [
        Self::Default,
        Self::Solarized,
        Self::HighContrast,
        Self::Charcoal,
        Self::TealNight,
        Self::White,
        Self::Pastel,
        Self::GreenTheme,
        Self::Nightshade,
        Self::Fjord,
        Self::FjordLight,
        Self::Harbor,
        Self::Mocha,
        Self::Rosewood,
        Self::Ember,
        Self::Parchment,
        Self::DeepSea,
        Self::Daybreak,
        Self::Citrus,
        Self::Evergreen,
    ];
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum PreferredEditor {
    #[default]
    Vscode,
    Cursor,
    Xcode,
    Zed,
}

impl PreferredEditor {
    pub const ALL: [Self; 4] = [Self::Vscode, Self::Cursor, Self::Xcode, Self::Zed];

    /// Editors offered on `os`; Xcode is macOS only.
    pub fn options(os: TargetOs) -> Vec<Self> {
        Self::ALL
            .into_iter()
            .filter(|e| *e != Self::Xcode || os == TargetOs::Macos)
            .collect()
    }

    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Vscode => "VS Code",
            Self::Cursor => "Cursor",
            Self::Xcode => "Xcode",
            Self::Zed => "Zed",
        }
    }

    pub const fn cli_command(self) -> &'static str {
        match self {
            Self::Vscode => "code",
            Self::Cursor => "cursor",
            Self::Xcode => "xed",
            Self::Zed => "zed",
        }
    }

    /// macOS bundle id, used to check the app is installed.
    pub const fn bundle_identifier(self) -> &'static str {
        match self {
            Self::Vscode => "com.microsoft.VSCode",
            Self::Cursor => "com.todesktop.230313mzl4w4u92",
            Self::Xcode => "com.apple.dt.Xcode",
            Self::Zed => "dev.zed.Zed",
        }
    }
}

/// Every terminal Heimdall can open, across OSes; [`PreferredTerminal::options`] gives each OS's set (ADR 0003).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum PreferredTerminal {
    // macOS (the Swift reference app's set)
    Terminal,
    Iterm2,
    Warp,
    Ghostty,
    Kitty,
    Alacritty,
    // Windows
    WindowsTerminal,
    Powershell,
    Cmd,
    // Linux
    GnomeTerminal,
    Konsole,
    /// `x-terminal-emulator`.
    System,
}

impl PreferredTerminal {
    pub fn options(os: TargetOs) -> &'static [Self] {
        use PreferredTerminal::*;
        match os {
            TargetOs::Macos => &[Terminal, Iterm2, Warp, Ghostty, Kitty, Alacritty],
            TargetOs::Windows => &[WindowsTerminal, Powershell, Cmd],
            TargetOs::Linux => &[GnomeTerminal, Konsole, Kitty, Alacritty, System],
        }
    }

    pub const fn default_for(os: TargetOs) -> Self {
        match os {
            TargetOs::Macos => Self::Terminal,
            TargetOs::Windows => Self::WindowsTerminal,
            TargetOs::Linux => Self::System,
        }
    }

    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Terminal => "Terminal",
            Self::Iterm2 => "iTerm2",
            Self::Warp => "Warp",
            Self::Ghostty => "Ghostty",
            Self::Kitty => "Kitty",
            Self::Alacritty => "Alacritty",
            Self::WindowsTerminal => "Windows Terminal",
            Self::Powershell => "PowerShell",
            Self::Cmd => "Command Prompt",
            Self::GnomeTerminal => "GNOME Terminal",
            Self::Konsole => "Konsole",
            Self::System => "System default",
        }
    }

    /// macOS bundle id; `None` for terminals Heimdall only launches on Windows or Linux.
    pub const fn bundle_identifier(self) -> Option<&'static str> {
        match self {
            Self::Terminal => Some("com.apple.Terminal"),
            Self::Iterm2 => Some("com.googlecode.iterm2"),
            Self::Warp => Some("dev.warp.Warp-Stable"),
            Self::Ghostty => Some("com.mitchellh.ghostty"),
            Self::Kitty => Some("net.kovidgoyal.kitty"),
            Self::Alacritty => Some("org.alacritty"),
            _ => None,
        }
    }
}

impl Default for PreferredTerminal {
    fn default() -> Self {
        Self::default_for(TargetOs::current())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", default)]
pub struct Prefs {
    pub schema_version: u32,
    pub show_menu_bar_icon: bool,
    pub has_completed_onboarding: bool,
    pub color_theme: ColorThemeId,
    /// 0.75–1.5 in 0.05 steps; `update_prefs` validates (M1-39).
    pub font_scale: f64,
    pub session_layout: Layout,
    /// Set only by `pick_config_project`.
    pub config_project_path: Option<String>,
    pub config_layouts: BTreeMap<String, Layout>,
    pub show_agent_sessions: bool,
    pub text_editor: PreferredEditor,
    pub preferred_terminal: PreferredTerminal,
    pub open_terminal_on_bash: bool,
    /// `None` means search PATH (the Swift reference app defaulted to `/usr/local/bin/claude`).
    pub claude_code_path: Option<String>,
    pub show_animations: bool,
    pub wrap_code_lines: bool,
    pub auto_expand_blocks: bool,
    pub parse_markdown_in_results: bool,
    pub global_search_mode: SearchMode,
    pub custom_memory_folders: Vec<String>,
}

/// A partial update to [`Prefs`] (contract `PrefsPatch`): every field but `schemaVersion` and the
/// two fields with their own commands (`customMemoryFolders`, `configProjectPath`), each optional
/// so a patch only touches the keys it sets.
#[derive(Debug, Clone, Default, PartialEq, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PrefsPatch {
    pub show_menu_bar_icon: Option<bool>,
    pub has_completed_onboarding: Option<bool>,
    pub color_theme: Option<ColorThemeId>,
    pub font_scale: Option<f64>,
    pub session_layout: Option<Layout>,
    pub config_layouts: Option<BTreeMap<String, Layout>>,
    pub show_agent_sessions: Option<bool>,
    pub text_editor: Option<PreferredEditor>,
    pub preferred_terminal: Option<PreferredTerminal>,
    pub open_terminal_on_bash: Option<bool>,
    pub claude_code_path: Option<String>,
    pub show_animations: Option<bool>,
    pub wrap_code_lines: Option<bool>,
    pub auto_expand_blocks: Option<bool>,
    pub parse_markdown_in_results: Option<bool>,
    pub global_search_mode: Option<SearchMode>,
}

impl Prefs {
    pub fn defaults_for(os: TargetOs) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            show_menu_bar_icon: true,
            has_completed_onboarding: false,
            color_theme: ColorThemeId::Default,
            font_scale: 1.0,
            session_layout: Layout::Grid,
            config_project_path: None,
            config_layouts: BTreeMap::new(),
            show_agent_sessions: false,
            text_editor: PreferredEditor::Vscode,
            preferred_terminal: PreferredTerminal::default_for(os),
            open_terminal_on_bash: false,
            claude_code_path: None,
            show_animations: true,
            wrap_code_lines: true,
            auto_expand_blocks: true,
            parse_markdown_in_results: true,
            global_search_mode: SearchMode::Fuzzy,
            custom_memory_folders: Vec::new(),
        }
    }

    /// Reads stored prefs key by key: a value that no longer parses (say, a theme id written by a
    /// newer version) falls back to its default instead of resetting every other preference.
    pub fn from_stored(stored: serde_json::Value, os: TargetOs) -> Self {
        let defaults = Self::defaults_for(os);
        let serde_json::Value::Object(mut merged) = serde_json::to_value(&defaults).expect("prefs serialise") else {
            unreachable!("Prefs serialises to an object")
        };
        if let serde_json::Value::Object(stored) = stored {
            for (key, value) in stored {
                let Some(previous) = merged.insert(key.clone(), value) else {
                    merged.remove(&key); // unknown key
                    continue;
                };
                if serde_json::from_value::<Self>(serde_json::Value::Object(merged.clone())).is_err() {
                    merged.insert(key, previous);
                }
            }
        }
        serde_json::from_value(serde_json::Value::Object(merged)).unwrap_or(defaults)
    }
}

impl Default for Prefs {
    fn default() -> Self {
        Self::defaults_for(TargetOs::current())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Ported from PoirotTests/Models/PreferredEditorTests.swift.
    #[test]
    fn all_cases_has_four_editors() {
        assert_eq!(PreferredEditor::ALL.len(), 4);
    }
    /// Deviation: the stored value for VS Code is `vscode` (ADR 0007), not the Swift reference app's `code`.
    #[test]
    fn editor_raw_values_match_expected() {
        let raw = PreferredEditor::ALL.map(|e| serde_json::to_value(e).unwrap());
        assert_eq!(raw, ["vscode", "cursor", "xcode", "zed"].map(serde_json::Value::from));
    }
    #[test]
    fn editor_display_names_non_empty() {
        assert!(PreferredEditor::ALL.iter().all(|e| !e.display_name().is_empty()));
    }
    #[test]
    fn cli_commands_match_expected() {
        assert_eq!(
            PreferredEditor::ALL.map(PreferredEditor::cli_command),
            ["code", "cursor", "xed", "zed"]
        );
    }
    #[test]
    fn editor_bundle_identifiers_non_empty() {
        assert!(PreferredEditor::ALL.iter().all(|e| !e.bundle_identifier().is_empty()));
    }
    #[test]
    fn editor_bundle_identifiers_match_expected() {
        assert_eq!(
            PreferredEditor::ALL.map(PreferredEditor::bundle_identifier),
            [
                "com.microsoft.VSCode",
                "com.todesktop.230313mzl4w4u92",
                "com.apple.dt.Xcode",
                "dev.zed.Zed"
            ]
        );
    }

    // Ported from PoirotTests/Models/PreferredTerminalTests.swift; the Swift reference app's six are the macOS set.
    #[test]
    fn all_cases_has_six_terminals() {
        assert_eq!(PreferredTerminal::options(TargetOs::Macos).len(), 6);
    }
    #[test]
    fn terminal_raw_values_match_expected() {
        let raw: Vec<_> = PreferredTerminal::options(TargetOs::Macos)
            .iter()
            .map(|t| serde_json::to_value(t).unwrap())
            .collect();
        assert_eq!(
            raw,
            ["terminal", "iterm2", "warp", "ghostty", "kitty", "alacritty"].map(serde_json::Value::from)
        );
    }
    #[test]
    fn terminal_display_names_non_empty() {
        let all = [TargetOs::Macos, TargetOs::Windows, TargetOs::Linux]
            .map(PreferredTerminal::options)
            .concat();
        assert!(all.iter().all(|t| !t.display_name().is_empty()));
    }
    #[test]
    fn terminal_bundle_identifiers_match_expected() {
        let ids: Vec<_> = PreferredTerminal::options(TargetOs::Macos)
            .iter()
            .map(|t| t.bundle_identifier().unwrap())
            .collect();
        assert_eq!(
            ids,
            [
                "com.apple.Terminal",
                "com.googlecode.iterm2",
                "dev.warp.Warp-Stable",
                "com.mitchellh.ghostty",
                "net.kovidgoyal.kitty",
                "org.alacritty"
            ]
        );
    }

    #[test]
    fn per_os_options_and_defaults() {
        assert!(!PreferredEditor::options(TargetOs::Windows).contains(&PreferredEditor::Xcode));
        assert!(PreferredEditor::options(TargetOs::Macos).contains(&PreferredEditor::Xcode));
        for os in [TargetOs::Macos, TargetOs::Windows, TargetOs::Linux] {
            let d = Prefs::defaults_for(os);
            assert!(PreferredTerminal::options(os).contains(&d.preferred_terminal));
        }
        assert_eq!(
            Prefs::defaults_for(TargetOs::Windows).preferred_terminal,
            PreferredTerminal::WindowsTerminal
        );
        assert_eq!(
            Prefs::defaults_for(TargetOs::Linux).preferred_terminal,
            PreferredTerminal::System
        );
        assert_eq!(
            Prefs::defaults_for(TargetOs::Macos).preferred_terminal,
            PreferredTerminal::Terminal
        );
    }

    #[test]
    fn has_twenty_theme_ids() {
        assert_eq!(ColorThemeId::ALL.len(), 20);
        assert_eq!(
            serde_json::to_value(ColorThemeId::HighContrast).unwrap(),
            "highContrast"
        );
    }

    #[test]
    fn defaults_follow_adr_0007() {
        let v = serde_json::to_value(Prefs::defaults_for(TargetOs::Windows)).unwrap();
        assert_eq!(v.as_object().unwrap().len(), 19); // schemaVersion + 18 keys
        assert_eq!(v["claudeCodePath"], serde_json::Value::Null);
        assert_eq!(
            (
                v["colorTheme"].as_str(),
                v["fontScale"].as_f64(),
                v["textEditor"].as_str()
            ),
            (Some("default"), Some(1.0), Some("vscode"))
        );
        assert_eq!(
            (
                v["showMenuBarIcon"].as_bool(),
                v["wrapCodeLines"].as_bool(),
                v["showAgentSessions"].as_bool()
            ),
            (Some(true), Some(true), Some(false))
        );
    }

    #[test]
    fn missing_keys_take_defaults_and_unknown_keys_are_ignored() {
        let p: Prefs = serde_json::from_value(json!({"fontScale": 1.25, "someFutureKey": 1})).unwrap();
        assert_eq!(p.font_scale, 1.25);
        assert_eq!(
            p,
            Prefs {
                font_scale: 1.25,
                ..Prefs::default()
            }
        );
    }

    #[test]
    fn a_bad_value_falls_back_for_that_key_only() {
        let stored =
            json!({"colorTheme": "fromTheFuture", "customMemoryFolders": ["/notes"], "wrapCodeLines": false, "x": 1});
        let p = Prefs::from_stored(stored, TargetOs::Linux);
        assert_eq!(p.color_theme, ColorThemeId::Default);
        assert_eq!(p.custom_memory_folders, ["/notes"]);
        assert!(!p.wrap_code_lines);
        assert_eq!(
            Prefs::from_stored(json!("garbage"), TargetOs::Linux),
            Prefs::defaults_for(TargetOs::Linux)
        );
    }
}
