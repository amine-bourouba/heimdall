//! `settings.json` reader (ClaudeConfigLoader.loadSettings and loadProjectModel): permissions,
//! enabled plugins and model. Hooks are read by [`super::hooks`], which keeps raw handler fields.

use std::io::ErrorKind;
use std::path::Path;

use crate::error::{AppError, Result};
use crate::models::config::settings::ClaudeSettings;

/// `None` when the file does not exist; a file that does not parse is an error naming it, so the
/// UI can say so instead of showing empty settings.
pub fn load(settings_file: &Path) -> Result<Option<ClaudeSettings>> {
    let bytes = match std::fs::read(settings_file) {
        Ok(b) => b,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(AppError::from_io(&e, settings_file)),
    };
    serde_json::from_slice(&bytes)
        .map(Some)
        .map_err(|e| AppError::parse(e.to_string()).with_path(settings_file))
}

/// The `model` of `<project>/.claude/settings.json`; `None` when there is none or the file is unreadable.
pub fn project_model(project_root: &Path) -> Option<String> {
    load(&project_root.join(".claude").join("settings.json"))
        .ok()
        .flatten()?
        .model
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::read::test_support::{home, repo, synthetic};

    #[test]
    fn reads_permissions_plugins_and_model() {
        let s = load(&home().join("settings.json")).unwrap().unwrap();
        assert_eq!(s.model.as_deref(), Some("claude-opus-4-6"));
        let p = s.permissions.unwrap();
        assert_eq!(
            (p.allow.unwrap().len(), p.deny.unwrap()),
            (4, vec!["mcp__filesystem__delete_file".to_string()])
        );
        assert_eq!(s.enabled_plugins.unwrap().get("legacy-tools@acme"), Some(&false));
    }

    #[test]
    fn missing_is_none_and_corrupt_is_a_parse_error() {
        assert!(load(Path::new("/nonexistent/2c7e/settings.json")).unwrap().is_none());
        assert!(matches!(
            load(&synthetic().join("corrupt").join("settings.json")).unwrap_err(),
            AppError::Parse { path: Some(_), .. }
        ));
    }

    #[test]
    fn project_model_matches_load_project_model() {
        assert_eq!(project_model(&repo()).as_deref(), Some("claude-sonnet-4-6"));
        assert_eq!(project_model(Path::new("/nonexistent/7a1f")), None);
    }
}
