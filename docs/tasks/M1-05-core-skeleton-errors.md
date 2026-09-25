# M1-05 · Declare every heimdall-core module and implement AppError

> Milestone M1 · Workstream: Rust data core · Scope: MVP

## Goal

`heimdall-core` declares every module in ADR 0002 and implements the serialisable `AppError` and `Loaded<T>` types exactly as docs/ipc-contract.md §3 defines them.

## Inputs

- `docs/adr/0002-layer-boundaries.md`
- `docs/adr/0005-error-model.md`
- `docs/ipc-contract.md` — §3 Errors and wrappers

## Outputs

- `crates/heimdall-core/src/lib.rs` — modify: declare every module
- `crates/heimdall-core/src/error.rs` — create
- `crates/heimdall-core/src/paths.rs` — create: placeholder
- `crates/heimdall-core/src/jsonl.rs` — create: placeholder
- `crates/heimdall-core/src/classify.rs` — create: placeholder
- `crates/heimdall-core/src/diff.rs` — create: placeholder
- `crates/heimdall-core/src/mcp_status.rs` — create: placeholder
- `crates/heimdall-core/src/models/mod.rs` — create: placeholder
- `crates/heimdall-core/src/models/message.rs` — create: placeholder
- `crates/heimdall-core/src/models/session.rs` — create: placeholder
- `crates/heimdall-core/src/models/project.rs` — create: placeholder
- `crates/heimdall-core/src/models/token_usage.rs` — create: placeholder
- `crates/heimdall-core/src/models/context_usage.rs` — create: placeholder
- `crates/heimdall-core/src/models/session_group.rs` — create: placeholder
- `crates/heimdall-core/src/models/stats_cache.rs` — create: placeholder
- `crates/heimdall-core/src/models/history_entry.rs` — create: placeholder
- `crates/heimdall-core/src/models/session_todo.rs` — create: placeholder
- `crates/heimdall-core/src/models/session_facets.rs` — create: placeholder
- `crates/heimdall-core/src/models/debug_log_entry.rs` — create: placeholder
- `crates/heimdall-core/src/models/file_version.rs` — create: placeholder
- `crates/heimdall-core/src/models/prefs.rs` — create: placeholder
- `crates/heimdall-core/src/models/secret.rs` — create: placeholder
- `crates/heimdall-core/src/models/config/mod.rs` — create: placeholder
- `crates/heimdall-core/src/models/config/scope.rs` — create: placeholder
- `crates/heimdall-core/src/models/config/mcp.rs` — create: placeholder
- `crates/heimdall-core/src/models/config/hook.rs` — create: placeholder
- `crates/heimdall-core/src/models/config/sub_agent.rs` — create: placeholder
- `crates/heimdall-core/src/models/config/command.rs` — create: placeholder
- `crates/heimdall-core/src/models/config/skill.rs` — create: placeholder
- `crates/heimdall-core/src/models/config/output_style.rs` — create: placeholder
- `crates/heimdall-core/src/models/config/plan.rs` — create: placeholder
- `crates/heimdall-core/src/models/config/memory_file.rs` — create: placeholder
- `crates/heimdall-core/src/models/config/plugin.rs` — create: placeholder
- `crates/heimdall-core/src/models/config/settings.rs` — create: placeholder
- `crates/heimdall-core/src/models/catalog/mod.rs` — create: placeholder
- `crates/heimdall-core/src/models/catalog/claude_models.rs` — create: placeholder
- `crates/heimdall-core/src/models/catalog/mcp_catalog.rs` — create: placeholder
- `crates/heimdall-core/src/transcript/mod.rs` — create: placeholder
- `crates/heimdall-core/src/transcript/full.rs` — create: placeholder
- `crates/heimdall-core/src/transcript/blocks.rs` — create: placeholder
- `crates/heimdall-core/src/transcript/searchable.rs` — create: placeholder
- `crates/heimdall-core/src/transcript/system_content.rs` — create: placeholder
- `crates/heimdall-core/src/transcript/tool_input.rs` — create: placeholder
- `crates/heimdall-core/src/transcript/header.rs` — create: placeholder
- `crates/heimdall-core/src/transcript/summary.rs` — create: placeholder
- `crates/heimdall-core/src/projects/mod.rs` — create: placeholder
- `crates/heimdall-core/src/projects/discovery.rs` — create: placeholder
- `crates/heimdall-core/src/projects/sessions_index.rs` — create: placeholder
- `crates/heimdall-core/src/projects/subagents.rs` — create: placeholder
- `crates/heimdall-core/src/projects/project_path.rs` — create: placeholder
- `crates/heimdall-core/src/projects/delete.rs` — create: placeholder
- `crates/heimdall-core/src/data/mod.rs` — create: placeholder
- `crates/heimdall-core/src/data/history.rs` — create: placeholder
- `crates/heimdall-core/src/data/history_write.rs` — create: placeholder
- `crates/heimdall-core/src/data/todos.rs` — create: placeholder
- `crates/heimdall-core/src/data/debug_log.rs` — create: placeholder
- `crates/heimdall-core/src/data/facets.rs` — create: placeholder
- `crates/heimdall-core/src/data/file_history.rs` — create: placeholder
- `crates/heimdall-core/src/data/stats_cache.rs` — create: placeholder
- `crates/heimdall-core/src/data/stats.rs` — create: placeholder
- `crates/heimdall-core/src/data/tray.rs` — create: placeholder
- `crates/heimdall-core/src/config/mod.rs` — create: placeholder
- `crates/heimdall-core/src/config/read/mod.rs` — create: placeholder
- `crates/heimdall-core/src/config/read/frontmatter.rs` — create: placeholder
- `crates/heimdall-core/src/config/read/commands.rs` — create: placeholder
- `crates/heimdall-core/src/config/read/skills.rs` — create: placeholder
- `crates/heimdall-core/src/config/read/plans.rs` — create: placeholder
- `crates/heimdall-core/src/config/read/output_styles.rs` — create: placeholder
- `crates/heimdall-core/src/config/read/agents.rs` — create: placeholder
- `crates/heimdall-core/src/config/read/settings.rs` — create: placeholder
- `crates/heimdall-core/src/config/read/hooks.rs` — create: placeholder
- `crates/heimdall-core/src/config/read/mcp.rs` — create: placeholder
- `crates/heimdall-core/src/config/read/plugins.rs` — create: placeholder
- `crates/heimdall-core/src/config/read/memory.rs` — create: placeholder
- `crates/heimdall-core/src/config/read/models.rs` — create: placeholder
- `crates/heimdall-core/src/config/write/mod.rs` — create: placeholder
- `crates/heimdall-core/src/config/write/json_doc.rs` — create: placeholder
- `crates/heimdall-core/src/config/write/atomic.rs` — create: placeholder
- `crates/heimdall-core/src/config/write/settings.rs` — create: placeholder
- `crates/heimdall-core/src/config/write/mcp.rs` — create: placeholder
- `crates/heimdall-core/src/config/write/files.rs` — create: placeholder
- `crates/heimdall-core/src/search/mod.rs` — create: placeholder
- `crates/heimdall-core/src/search/matcher.rs` — create: placeholder
- `crates/heimdall-core/src/search/providers.rs` — create: placeholder
- `crates/heimdall-core/src/os_integration/mod.rs` — create: placeholder
- `crates/heimdall-core/src/os_integration/cli_locate.rs` — create: placeholder
- `crates/heimdall-core/src/os_integration/launch.rs` — create: placeholder
- `crates/heimdall-core/src/os_integration/links.rs` — create: placeholder
- `crates/heimdall-core/src/export/mod.rs` — create: placeholder
- `crates/heimdall-core/src/export/session_markdown.rs` — create: placeholder
- `crates/heimdall-core/src/export/patch.rs` — create: placeholder
- `crates/heimdall-core/src/export/analytics_csv.rs` — create: placeholder

## Acceptance criteria

- [ ] `cargo test -p heimdall-core error::` passes, with a JSON snapshot test per AppError variant, for example `{"kind":"notFound","message":"…","path":null}`.
- [ ] `AppError` derives `thiserror::Error`, `serde::Serialize` and `specta::Type`; `Loaded<T>` carries `value` and `skipped`.
- [ ] `#![deny(clippy::print_stdout, clippy::print_stderr)]` is set in lib.rs (ADR 0005 logging rule).
- [ ] `cargo doc -p heimdall-core --no-deps` has no warnings.

## Dependencies

- M1-01 — Create the Cargo workspace, crate manifests and supply-chain policy

## Owning agent

rust-engineer

## Reviewer agent

architect-reviewer

## Size

M

## Context

Stub rule (port-plan §5.4): a skeleton task may create placeholder files owned by later tasks. A placeholder holds only a module doc comment and the minimal signatures the wiring needs. The later task owns the file outright.

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
