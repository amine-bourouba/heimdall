# M1-36 · Session Markdown and patch export

> Milestone M1 · Workstream: Rust data core · Scope: MVP

## Goal

Session export produces the same Markdown as SessionExporter (with its four toggles) and unified-diff patches for Edit blocks.

## Inputs

- `$POIROT/Poirot/Sources/Services/Export/SessionExporter.swift:1-280`
- `$POIROT/Poirot/Sources/Views/Session/ExportOptionsView.swift`
- `docs/spec/session-views.md`

## Outputs

- `crates/heimdall-core/src/export/session_markdown.rs` — fill the placeholder created by M1-05
- `crates/heimdall-core/src/export/patch.rs` — fill the placeholder created by M1-05

## Acceptance criteria

- [ ] Tests cover each toggle (timestamps, tool results, thinking, token usage) on fixtures/screenshot.
- [ ] Patch output applies cleanly with `git apply --check` in a test.

## Dependencies

- M0-08 — Behaviour spec for the session browser and conversation view
- M1-12 — Transcript parser: full-parse tier, blocks, system content, tool input
- M1-35 — Line diff port

## Owning agent

rust-engineer

## Reviewer agent

architect-reviewer

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
