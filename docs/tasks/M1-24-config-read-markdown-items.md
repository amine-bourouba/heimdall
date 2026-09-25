# M1-24 · Loaders for commands, skills, plans, output styles and agents

> Milestone M1 · Workstream: Rust data core · Scope: MVP

## Goal

Commands, skills, plans, output styles and custom agents load from global and project scope with stable ConfigRef slugs, as ClaudeConfigLoader does.

## Inputs

- `$POIROT/Poirot/Sources/Services/Config/ClaudeConfigLoader.swift:10-110, 382-417, 614-650`
- `$POIROT/Poirot/Sources/Models/Config/SubAgent.swift:76-110`
- `docs/ipc-contract.md` — ConfigRef
- `docs/adr/0003-platform-abstraction.md`

## Outputs

- `crates/heimdall-core/src/config/read/commands.rs` — fill the placeholder created by M1-05
- `crates/heimdall-core/src/config/read/skills.rs` — fill the placeholder created by M1-05
- `crates/heimdall-core/src/config/read/plans.rs` — fill the placeholder created by M1-05
- `crates/heimdall-core/src/config/read/output_styles.rs` — fill the placeholder created by M1-05
- `crates/heimdall-core/src/config/read/agents.rs` — fill the placeholder created by M1-05

## Acceptance criteria

- [ ] Tests on fixtures/synthetic cover global and project scope per loader.
- [ ] Output styles exclude README.md; plan names are humanised from file names; built-in agents are merged with custom ones.
- [ ] Each item carries a `ConfigRef`; no loader returns a raw path meant for the webview to send back.

## Dependencies

- M1-06 — Claude home resolution, path normalisation and comparison keys
- M1-10 — Config models, catalogs and the redacting Secret type
- M1-23 — Frontmatter parser

## Owning agent

rust-engineer

## Reviewer agent

architect-reviewer

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
