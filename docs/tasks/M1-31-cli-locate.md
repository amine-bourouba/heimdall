# M1-31 · Cross-platform PATH search for claude and editor CLIs

> Milestone M1 · Workstream: Rust data core · Scope: MVP

## Goal

A pure PATH search finds `claude` (`claude`, `claude.cmd`, `claude.exe` via PATHEXT) and editor CLIs on each OS, replacing hard-coded macOS paths.

## Inputs

- `$SWIFT_REF/Poirot/Sources/Views/Onboarding/OnboardingView.swift:281-297`
- `$SWIFT_REF/Poirot/Sources/Utilities/EditorLauncher.swift:39-48`
- `$SWIFT_REF/Poirot/Sources/Models/PreferredEditor.swift`
- `docs/adr/0003-platform-abstraction.md`

## Outputs

- `crates/heimdall-core/src/os_integration/cli_locate.rs` — fill the placeholder created by M1-05

## Acceptance criteria

- [ ] Functions take PATH, PATHEXT, home and an `exists` function as inputs; tests cover Windows, Linux and macOS layouts on any host.
- [ ] Fallback directories per OS are listed in one table (for example `~/.local/bin`, `%APPDATA%\npm`, `/opt/homebrew/bin`).

## Dependencies

- M1-06 — Claude home resolution, path normalisation and comparison keys

## Owning agent

rust-engineer

## Reviewer agent

architect-reviewer, security-auditor

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
