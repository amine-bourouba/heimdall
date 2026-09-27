//! The Models screen (Views/Config/ModelsListView.swift): the curated catalog plus models found in
//! transcripts, with the default model from the global `settings.json` and the project's model.

use serde::Serialize;
use specta::Type;

use crate::models::catalog::claude_models::{
    ClaudeModelInfo, DEFAULT_MODEL_NAME, curated_names, friendly_name, models,
};

#[derive(Debug, Clone, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ModelsView {
    pub models: Vec<ClaudeModelInfo>,
    /// Display name of the global default: the settings `model`, or the catalog default when unset.
    pub default_model: String,
    /// Display name of the project's `model`, if it sets one.
    pub project_model: Option<String>,
}

/// `discovered_ids` are session `model` values; `settings_model` and `project_model` are the raw
/// `model` values of the global and project settings (ids such as `claude-opus-4-6`, aliases such
/// as `opus`, or display names Poirot wrote).
pub fn models_view(discovered_ids: &[String], settings_model: Option<&str>, project_model: Option<&str>) -> ModelsView {
    ModelsView {
        models: models(discovered_ids),
        default_model: settings_model.map_or_else(|| DEFAULT_MODEL_NAME.to_string(), display_name),
        project_model: project_model.map(display_name),
    }
}

/// A catalog display name as it is (any case); anything else through [`friendly_name`].
fn display_name(raw: &str) -> String {
    curated_names()
        .into_iter()
        .find(|n| n.eq_ignore_ascii_case(raw))
        .unwrap_or_else(|| friendly_name(raw))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::catalog::claude_models::curated;

    #[test]
    fn catalog_plus_discovered_with_default_and_project_models() {
        let v = models_view(
            &["claude-3-5-sonnet-20241022".into()],
            Some("claude-opus-4-8-20260514"),
            Some("Sonnet 5"),
        );
        assert_eq!(v.models.len(), curated().len() + 1);
        assert_eq!(
            (v.default_model.as_str(), v.project_model.as_deref()),
            ("Opus 4.8", Some("Sonnet 5"))
        );
    }

    #[test]
    fn the_catalog_default_applies_when_settings_set_none() {
        let v = models_view(&[], None, None);
        assert_eq!((v.default_model.as_str(), v.project_model), ("Opus 5", None));
        assert_eq!(
            models_view(&[], Some("opus 5"), Some("claude-haiku-4-5-20251001"))
                .project_model
                .as_deref(),
            Some("Haiku 4.5")
        );
    }
}
