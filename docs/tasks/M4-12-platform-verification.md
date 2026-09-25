# M4-12 · Install and smoke-test on target platforms

> Milestone M4 · Workstream: Testing · Scope: MVP

## Goal

Installers from the dry run are installed and smoke-tested on every target platform, with results recorded.

## Inputs

- `docs/release/dry-run.md`
- `docs/install.md`

## Outputs

- `docs/release/platform-verification.md` — create

## Acceptance criteria

- [ ] Platforms: Windows 10 22H2 and 11; Ubuntu 22.04 and 24.04 (deb and AppImage); Debian 12 (deb); current Fedora (rpm); Arch (AppImage); macOS 13+ and 15 on Apple Silicon.
- [ ] Smoke steps: launch, list sessions, open a transcript, search, switch theme, check for updates.
- [ ] Records whether the Flathub runtime ships WebKitGTK 4.1 (information for a later Flatpak).

## Dependencies

- M4-09 — Install guide for unsigned builds
- M4-11 — Dry-run a tagged pre-release

## Owning agent

test-automator

## Reviewer agent

qa-expert

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
