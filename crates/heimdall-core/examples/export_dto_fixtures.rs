//! Generates `fixtures/dto/**` from the real Rust loaders against `fixtures/screenshot` (task
//! M2-01) — Vitest's DTO fixtures are never hand-typed, so they can never silently drift from
//! what the real commands actually serialise over IPC.
//!
//! Usage: `cargo run -p heimdall-core --example export_dto_fixtures`

use std::path::{Path, PathBuf};

use heimdall_core::config::read::memory;
use heimdall_core::paths::TargetOs;
use heimdall_core::projects::discovery;

const POIROT: &str = "-Users-fixture-Dev-poirot";

fn fixtures_root() -> PathBuf {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures")).to_path_buf()
}

fn main() {
    let claude_home = fixtures_root().join("screenshot").join("claude-home");
    let projects_dir = claude_home.join("projects");
    // fixtures/screenshot was captured on macOS (fixtures_conformance.rs's own Set table) —
    // TargetOs::current() would silently misparse it when this runs on a Windows/Linux dev
    // machine, exactly the ADR 0003 mistake this whole repo is built to catch.
    let os = TargetOs::Macos;

    let loaded = discovery::discover(&projects_dir, os).expect("discover should not error");
    assert_eq!(
        loaded.skipped, 0,
        "fixture discovery skipped {} lines/files",
        loaded.skipped
    );
    let projects = loaded.value;

    let project_summaries: Vec<_> = projects
        .iter()
        .map(|p| {
            let has_memory = memory::folder_file_count(&projects_dir.join(&p.id).join("memory")) > 0;
            p.summary(has_memory)
        })
        .collect();
    write_json("projects.json", &project_summaries);

    let poirot = projects
        .iter()
        .find(|p| p.id == POIROT)
        .expect("fixtures/screenshot must have the poirot project");
    let session_summaries: Vec<_> = poirot
        .sessions
        .iter()
        .filter(|s| !s.is_sidechain) // list_sessions' own default (includeAgents: false)
        .map(|s| s.summary(POIROT))
        .collect();
    write_json("sessions-poirot.json", &session_summaries);

    println!("wrote fixtures/dto/projects.json and fixtures/dto/sessions-poirot.json");
}

fn write_json<T: serde::Serialize>(name: &str, value: &T) {
    let dir = fixtures_root().join("dto");
    std::fs::create_dir_all(&dir).unwrap();
    let json = serde_json::to_string_pretty(value).unwrap();
    std::fs::write(dir.join(name), json + "\n").unwrap();
}
