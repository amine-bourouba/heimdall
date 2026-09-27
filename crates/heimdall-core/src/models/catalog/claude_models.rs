//! Curated Claude model catalog (Models/ClaudeModelCatalog.swift): curated models first, then any
//! model found in session transcripts. Fully local.

use std::collections::HashSet;

use serde::Serialize;
use specta::Type;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeModelInfo {
    pub display_name: String,
    /// Empty for models discovered only in transcripts.
    pub description: String,
    pub strengths: Vec<String>,
}

impl ClaudeModelInfo {
    fn curated(display_name: &str, description: &str, strengths: [&str; 4]) -> Self {
        Self {
            display_name: display_name.into(),
            description: description.into(),
            strengths: strengths.map(String::from).to_vec(),
        }
    }
}

/// Model assumed when none is configured.
pub const DEFAULT_MODEL_NAME: &str = "Opus 5";

/// Curated current models, most capable first.
pub fn curated() -> Vec<ClaudeModelInfo> {
    vec![
        ClaudeModelInfo::curated(
            "Fable 5",
            "Mythos-class model for ambitious, days-long coding and knowledge work.",
            [
                "Ambitious multi-day coding",
                "Large-scale migrations",
                "Vision, documents, and diagrams",
                "Long-horizon agentic work",
            ],
        ),
        ClaudeModelInfo::curated(
            "Opus 5",
            "Everyday flagship that approaches Fable 5 on coding and knowledge work at half the price.",
            [
                "Complex agentic coding",
                "Root-cause debugging",
                "Knowledge work and analysis",
                "Self-verifying iteration",
            ],
        ),
        ClaudeModelInfo::curated(
            "Opus 4.8",
            "Most capable Claude model for complex reasoning, large-codebase work, and multi-step tasks.",
            [
                "Complex multi-step reasoning",
                "Large codebase analysis",
                "Architecture design",
                "Nuanced debugging",
            ],
        ),
        ClaudeModelInfo::curated(
            "Sonnet 5",
            "Balanced speed and capability for everyday coding and conversation.",
            [
                "Fast interactive coding",
                "Code generation",
                "Refactoring",
                "Everyday tasks",
            ],
        ),
        ClaudeModelInfo::curated(
            "Haiku 4.5",
            "Fastest, lightest Claude model for quick, low-latency tasks.",
            [
                "Fastest response time",
                "Simple lookups",
                "Quick edits",
                "Low-latency workflows",
            ],
        ),
    ]
}

pub fn curated_names() -> Vec<String> {
    curated().into_iter().map(|m| m.display_name).collect()
}

/// `claude-opus-4-8-20260514` → `Opus 4.8`, `claude-3-5-sonnet-20241022` → `Sonnet 3.5`. Short numeric
/// segments are version parts; a segment of six or more digits is the date stamp. Unknown ids pass through.
pub fn friendly_name(model_id: &str) -> String {
    let lower = model_id.to_lowercase();
    let families = [
        ("opus", "Opus"),
        ("sonnet", "Sonnet"),
        ("haiku", "Haiku"),
        ("fable", "Fable"),
    ];
    let Some((_, label)) = families.iter().find(|(f, _)| lower.contains(f)) else {
        return model_id.to_string();
    };
    let stripped = lower.strip_prefix("claude-").unwrap_or(&lower);
    let version = stripped
        .split('-')
        .filter(|s| !s.is_empty() && s.len() < 6 && s.chars().all(|c| c.is_numeric()))
        .collect::<Vec<_>>()
        .join(".");
    if version.is_empty() {
        label.to_string()
    } else {
        format!("{label} {version}")
    }
}

/// Curated models, then each distinct discovered model the curated set does not cover (by friendly name).
pub fn models(discovered_ids: &[String]) -> Vec<ClaudeModelInfo> {
    let mut result = curated();
    let mut seen: HashSet<String> = result.iter().map(|m| m.display_name.clone()).collect();
    for id in discovered_ids.iter().filter(|id| !id.is_empty()) {
        let name = friendly_name(id);
        if seen.insert(name.clone()) {
            result.push(ClaudeModelInfo {
                display_name: name,
                description: String::new(),
                strengths: vec![],
            });
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn find(name: &str) -> Option<ClaudeModelInfo> {
        curated().into_iter().find(|m| m.display_name == name)
    }

    // Ported from PoirotTests/Models/ClaudeModelCatalogTests.swift.
    #[test]
    fn friendly_name_maps_current_ids() {
        assert_eq!(friendly_name("claude-opus-5"), "Opus 5");
        assert_eq!(friendly_name("claude-opus-4-8-20260514"), "Opus 4.8");
        assert_eq!(friendly_name("claude-sonnet-5-20251101"), "Sonnet 5");
        assert_eq!(friendly_name("claude-haiku-4-5-20251001"), "Haiku 4.5");
        assert_eq!(friendly_name("claude-fable-5"), "Fable 5");
    }
    #[test]
    fn friendly_name_handles_legacy_ordering() {
        assert_eq!(friendly_name("claude-3-5-sonnet-20241022"), "Sonnet 3.5");
        assert_eq!(friendly_name("claude-3-opus-20240229"), "Opus 3");
    }
    #[test]
    fn friendly_name_unknown_id_passes_through() {
        assert_eq!(friendly_name("gpt-4o"), "gpt-4o");
    }
    #[test]
    fn models_include_all_curated_even_with_no_sessions() {
        let names: Vec<_> = models(&[]).into_iter().map(|m| m.display_name).collect();
        assert!(
            ["Opus 5", "Fable 5", "Opus 4.8"]
                .iter()
                .all(|n| names.iter().any(|x| x == n))
        );
        assert_eq!(names.len(), curated().len());
    }
    #[test]
    fn opus5_has_description_and_strengths() {
        let m = find("Opus 5").unwrap();
        assert!(!m.description.is_empty());
        assert_eq!(m.strengths.len(), 4);
    }
    #[test]
    fn fable5_has_description_and_strengths() {
        let m = find("Fable 5").unwrap();
        assert!(!m.description.is_empty());
        assert_eq!(m.strengths.len(), 4);
        assert_eq!(curated()[0].display_name, "Fable 5");
    }
    #[test]
    fn models_append_discovered_without_duplicating_curated() {
        let ids = [
            "claude-opus-5",
            "claude-opus-4-8-20260514",
            "claude-3-5-sonnet-20241022",
            "claude-3-5-sonnet-20241022",
            "",
        ]
        .map(String::from);
        let names: Vec<_> = models(&ids).into_iter().map(|m| m.display_name).collect();
        let count = |n: &str| names.iter().filter(|x| *x == n).count();
        assert_eq!((count("Opus 5"), count("Opus 4.8"), count("Sonnet 3.5")), (1, 1, 1));
        assert_eq!(names.len(), curated().len() + 1);
        assert!(curated_names().contains(&DEFAULT_MODEL_NAME.to_string()));
    }
}
