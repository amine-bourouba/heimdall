//! The parts of `settings.json` Heimdall reads as typed data (Models/Config/ClaudeSettings.swift).
//! Hooks are read separately, as raw JSON, so unknown handler keys survive (config/read/hooks.rs).

use std::collections::BTreeMap;

use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ClaudeSettings {
    pub permissions: Option<Permissions>,
    pub enabled_plugins: Option<BTreeMap<String, bool>>,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Permissions {
    pub allow: Option<Vec<String>>,
    pub deny: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_known_keys_and_ignores_the_rest() {
        let s: ClaudeSettings = serde_json::from_str(
            r#"{"model": "opus", "enabledPlugins": {"a@b": true}, "permissions": {"allow": ["mcp__gh__*"]}, "hooks": {}, "other": 1}"#,
        ).unwrap();
        assert_eq!(s.model.as_deref(), Some("opus"));
        assert!(s.enabled_plugins.unwrap()["a@b"]);
        assert_eq!(s.permissions.unwrap().allow.unwrap(), ["mcp__gh__*"]);
    }
}
