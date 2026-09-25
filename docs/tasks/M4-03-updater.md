# M4-03 · Updater plugin wiring

> Milestone M4 · Workstream: Tauri shell / OS integration · Scope: MVP

## Goal

The updater plugin checks `latest.json` on the fork's GitHub Releases, verifies the minisign signature, and exposes `check()` and `install()` functions for the update commands.

## Inputs

- `docs/adr/0011-app-identifier-and-release-repo.md`
- `docs/adr/0010-webview-trust-boundary.md` — Updates and signing, Links
- `docs/ipc-contract.md` — commands/update.rs
- `docs/release/updater-key.md`

## Outputs

- `src-tauri/src/updater.rs` — modify
- `src-tauri/tauri.conf.json` — modify: plugins.updater, bundle.createUpdaterArtifacts

## Acceptance criteria

- [ ] The endpoint is `https://github.com/amine-bourouba/heimdall/releases/latest/download/latest.json` and the public key comes from updater-key.md.
- [ ] The plugin is registered at runtime from `updater::init`.
- [ ] A manual check against a draft release confirms signature verification fails for a tampered artifact.

## Dependencies

- M1-48 — Per-window capability files
- M1-49 — Scope app commands with AppManifest and verify exposure
- M2-12 — Per-OS window chrome and navigation guard
- M2-13 — Native macOS app menu emitting menu-action events
- M4-02 — Generate and store the updater signing key

## Owning agent

rust-engineer

## Reviewer agent

security-engineer, security-auditor, deployment-engineer

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
