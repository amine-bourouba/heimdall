# M1-01 · Create the Cargo workspace, crate manifests and supply-chain policy

> Milestone M1 · Workstream: Release and packaging · Scope: Infra

## Goal

A Cargo workspace with `heimdall-core` and the Tauri app crate resolves, with every Rust dependency needed through M3 pinned and `cargo deny` policy configured.

## Inputs

- `docs/adr/0002-layer-boundaries.md`
- `docs/adr/0004-ipc-contract-and-type-generation.md`
- `docs/adr/0010-webview-trust-boundary.md` — Supply chain
- `docs/adr/0011-app-identifier-and-release-repo.md` — crate and binary name
- `docs/port-plan.md` — §6 target layout

## Outputs

- `Cargo.toml` — create: workspace
- `Cargo.lock` — create
- `rust-toolchain.toml` — create
- `deny.toml` — create
- `crates/heimdall-core/Cargo.toml` — create
- `crates/heimdall-core/src/lib.rs` — create: empty placeholder, owned by M1-05
- `src-tauri/Cargo.toml` — create

## Acceptance criteria

- [ ] `cargo metadata --format-version 1 > /dev/null` succeeds and `cargo check -p heimdall-core` passes.
- [ ] `cargo tree -p heimdall-core -e normal | grep -c tauri` prints 0.
- [ ] heimdall-core depends on: serde (derive), serde_json with `preserve_order`, thiserror, dirs, specta (derive), sysinfo, chrono, regex, url; dev: tempfile. Windows-only: the `windows` crate with the file-system features for ReplaceFileW.
- [ ] The app crate depends on: tauri 2 (tray-icon feature), tauri-build, tauri-plugin-{opener,autostart,clipboard-manager,dialog,store,updater}, notify, notify-debouncer-full, specta, tauri-specta (exact pre-release pin), specta-typescript, serde, serde_json, heimdall-core (path). No tauri-plugin-fs or tauri-plugin-shell.
- [ ] rust-toolchain.toml pins a stable channel at or above 1.77.2 (CVE-2024-24576).
- [ ] `cargo deny check` passes (licenses, bans, advisories, sources).
- [ ] The app crate's package and binary name follow ADR 0011.

## Dependencies

- M0-02 — Record the final app identifier and Linux package name
- M0-03 — Initialise the Heimdall repository and its ignore/attribute rules

## Owning agent

dependency-manager

## Reviewer agent

architect-reviewer, security-auditor

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
