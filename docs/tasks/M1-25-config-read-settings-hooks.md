# M1-25 · Settings and hooks readers

> Milestone M1 · Workstream: Rust data core · Scope: MVP

## Goal

Global and project `settings.json` load into typed views (permissions, enabledPlugins, model, hooks) without losing unknown hook-handler fields.

## Inputs

- `$POIROT/Poirot/Sources/Services/Config/ClaudeConfigLoader.swift:534-613`
- `$POIROT/Poirot/Sources/Models/Config/Hook.swift`
- `$POIROT/Poirot/Sources/Models/Config/ClaudeSettings.swift`

## Outputs

- `crates/heimdall-core/src/config/read/settings.rs` — fill the placeholder created by M1-05
- `crates/heimdall-core/src/config/read/hooks.rs` — fill the placeholder created by M1-05

## Acceptance criteria

- [ ] Tests cover global and project hooks, matcher groups, http and command handlers, and unknown handler keys retained in `extra`.
- [ ] Project model reading matches `loadProjectModel`.

## Dependencies

- M1-06 — Claude home resolution, path normalisation and comparison keys
- M1-10 — Config models, catalogs and the redacting Secret type

## Owning agent

rust-engineer

## Reviewer agent

architect-reviewer

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
