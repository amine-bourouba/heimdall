# M4-09 · Install guide for unsigned builds

> Milestone M4 · Workstream: Docs · Scope: MVP

## Goal

Users can install and verify each unsigned build by following one page.

## Inputs

- `docs/adr/0011-app-identifier-and-release-repo.md`
- `src-tauri/tauri.linux.conf.json`

## Outputs

- `docs/install.md` — create

## Acceptance criteria

- [ ] Windows: SmartScreen 'More info → Run anyway' and the reputation note.
- [ ] macOS (Apple Silicon only): System Settings → Privacy & Security → Open Anyway, and the `xattr -dr com.apple.quarantine` alternative.
- [ ] Linux: deb, rpm and AppImage steps, and the GNOME AppIndicator extension for the tray.
- [ ] Checksum verification commands for each OS.

## Dependencies

- M4-01 — Per-OS bundle configuration

## Owning agent

technical-writer

## Reviewer agent

architect-reviewer

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
