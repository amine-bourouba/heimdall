//! ADR 0010 control 8: every MCP `env`/`headers` value is a secret and must never reach a
//! serialised DTO or a `Debug`/log line. This test drives the fixture at
//! `fixtures/security/secrets-claude.json` — seeded with `SENTINEL-SECRET-*` values across every
//! disk source (`~/.claude.json` user scope, its `projects.<path>` local scope, and a project's
//! own `.mcp.json`) — through the real loader and checks both output shapes it could leak from.

use std::path::Path;

use heimdall_core::config::read::mcp;
use heimdall_core::models::config::mcp::{ClaudeConfig, McpProjectConfig};
use heimdall_core::paths::{ClaudeHome, TargetOs};

const SENTINEL: &str = "SENTINEL-SECRET";

fn fixture_path() -> &'static Path {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/security/secrets-claude.json"
    ))
}

/// A Claude home seeded from the fixture, plus a project-scoped `.mcp.json` (the one disk source
/// the single-file fixture cannot represent on its own) carrying its own sentinel.
fn world() -> (tempfile::TempDir, ClaudeHome, String) {
    let dir = tempfile::tempdir().unwrap();
    let config_file = dir.path().join(".claude.json");
    let fixture = std::fs::read_to_string(fixture_path()).unwrap();
    // The fixture's project key is a placeholder path; point it at this run's real temp project.
    let project = dir.path().join("project");
    std::fs::create_dir_all(&project).unwrap();
    let project_str = project.display().to_string();
    std::fs::write(
        &config_file,
        fixture.replace("/fixture/project", &project_str.replace('\\', "\\\\")),
    )
    .unwrap();
    std::fs::write(
        project.join(".mcp.json"),
        serde_json::json!({"mcpServers": {"project-scoped": {
            "type": "stdio", "command": "node",
            "env": {"PROJECT_TOKEN": "SENTINEL-SECRET-PROJECT-ENV-1"}
        }}})
        .to_string(),
    )
    .unwrap();
    let root = dir.path().join(".claude");
    (dir, ClaudeHome::new(root, config_file), project_str)
}

#[test]
fn fixture_actually_contains_sentinels() {
    // Guards against a silently-empty or misnamed fixture making every other assertion vacuous.
    let raw = std::fs::read_to_string(fixture_path()).unwrap();
    assert!(
        raw.contains(SENTINEL),
        "fixture at {} carries no sentinel",
        fixture_path().display()
    );
}

#[test]
fn loaded_servers_never_carry_secrets_in_json_or_debug() {
    let (_dir, home, project) = world();
    let loaded = mcp::load(&home, Some(&project), TargetOs::current());
    let servers = loaded.value;

    // Sanity: all three disk-based sources (user, local override, project) were actually
    // exercised, not skipped as unparsed.
    assert_eq!(loaded.skipped, 0);
    let sources: Vec<_> = servers.iter().map(|s| s.raw_name.as_str()).collect();
    assert!(sources.contains(&"user-stdio"), "{sources:?}");
    assert!(sources.contains(&"local-db"), "{sources:?}");
    assert!(sources.contains(&"project-scoped"), "{sources:?}");

    let json = serde_json::to_string(&servers).unwrap();
    assert!(!json.contains(SENTINEL), "leaked into DTO JSON: {json}");

    let debug = format!("{servers:?}");
    assert!(!debug.contains(SENTINEL), "leaked into Debug output: {debug}");
    // The redaction marker must actually appear, proving the fields were visited, not omitted.
    assert!(debug.contains("[redacted]"), "{debug}");
}

#[test]
fn raw_disk_models_redact_on_debug_before_any_dto_conversion() {
    // Even the on-disk model (never `Serialize`, so it cannot reach JSON) must not print a
    // secret through `Debug`, since that is the path a stray `log::debug!("{cfg:?}")` would take.
    let (_dir, home, project) = world();
    let bytes = std::fs::read(home.config_file()).unwrap();
    let config: ClaudeConfig = serde_json::from_slice(&bytes).unwrap();
    let debug = format!("{config:?}");
    assert!(!debug.contains(SENTINEL), "leaked into ClaudeConfig Debug: {debug}");
    assert!(debug.contains("[redacted]"), "{debug}");

    let project_config: McpProjectConfig =
        serde_json::from_str(&std::fs::read_to_string(Path::new(&project).join(".mcp.json")).unwrap()).unwrap();
    let debug = format!("{project_config:?}");
    assert!(!debug.contains(SENTINEL), "leaked into McpProjectConfig Debug: {debug}");
}
