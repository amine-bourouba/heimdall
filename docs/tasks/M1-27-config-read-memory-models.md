# M1-27 · Memory and models readers

> Milestone M1 · Workstream: Rust data core · Scope: MVP

## Goal

Memory files load from project memory folders and user-chosen folders (MEMORY.md first), and the models view merges the curated catalog with transcript-discovered model ids plus default and project models.

## Inputs

- `$SWIFT_REF/Poirot/Sources/Services/Config/ClaudeConfigLoader.swift:418-533`
- `$SWIFT_REF/Poirot/Sources/Models/ClaudeModelCatalog.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Config/ModelsListView.swift`
- `$SWIFT_REF/PoirotTests/Services/MemoryLoaderTests.swift`
- `$SWIFT_REF/PoirotTests/Models/ClaudeModelCatalogTests.swift`

## Outputs

- `crates/heimdall-core/src/config/read/memory.rs` — fill the placeholder created by M1-05
- `crates/heimdall-core/src/config/read/models.rs` — fill the placeholder created by M1-05

## Acceptance criteria

- [ ] Carried-over MemoryLoaderTests and the logic part of ClaudeModelCatalogTests pass.
- [ ] Custom folders come from `Prefs.customMemoryFolders` passed in by the caller (no preference storage access in core).

## Dependencies

- M1-10 — Config models, catalogs and the redacting Secret type
- M1-11 — Typed Prefs model with per-OS defaults
- M1-14 — Project and session discovery with cwd-first project paths

## Owning agent

rust-engineer

## Reviewer agent

architect-reviewer

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
