# M1-11 · Typed Prefs model with per-OS defaults

> Milestone M1 · Workstream: Rust data core · Scope: MVP

## Goal

A `Prefs` struct holds all 18 preference keys with per-OS defaults and a `schemaVersion`, and editor/terminal enums cover each OS.

## Inputs

- `$SWIFT_REF/Poirot/Sources/App/AppState.swift:72-80, 270-297`
- `$SWIFT_REF/Poirot/Sources/App/SettingsView.swift`
- `$SWIFT_REF/Poirot/Sources/App/MemorySourcesStore.swift`
- `$SWIFT_REF/Poirot/Sources/Models/PreferredEditor.swift`
- `$SWIFT_REF/Poirot/Sources/Models/PreferredTerminal.swift`
- `$SWIFT_REF/PoirotTests/Models/PreferredEditorTests.swift`
- `$SWIFT_REF/PoirotTests/Models/PreferredTerminalTests.swift`
- `docs/adr/0007-preferences-storage-and-migration.md`
- `docs/parity-matrix.md` — section I

## Outputs

- `crates/heimdall-core/src/models/prefs.rs` — fill the placeholder created by M1-05

## Acceptance criteria

- [ ] Every key in parity-matrix section I exists with the ADR 0007 type and default; `claudeCodePath` defaults to none (PATH search) instead of `/usr/local/bin/claude`.
- [ ] `#[serde(default)]` everywhere: unknown keys are ignored and missing keys take defaults (tested).
- [ ] `PreferredEditor` = vscode, cursor, zed, xcode (xcode offered on macOS only); `PreferredTerminal` has per-OS variant sets per ADR 0003; carried-over tests pass.
- [ ] `ColorThemeId` has the 20 theme ids from PoirotTheme.swift:169-192.

## Dependencies

- M1-05 — Declare every heimdall-core module and implement AppError

## Owning agent

rust-engineer

## Reviewer agent

architect-reviewer

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
