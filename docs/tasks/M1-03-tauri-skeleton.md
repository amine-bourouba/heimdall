# M1-03 · Scaffold the Tauri app crate with final identity, CSP and module stubs

> Milestone M1 · Workstream: Tauri shell / OS integration · Scope: Infra

## Goal

The Tauri app compiles and opens an empty main window on Windows, Linux and macOS with the final identifier, the ADR 0010 CSP, and a placeholder for every src-tauri module listed in ADR 0002.

## Inputs

- `docs/adr/0002-layer-boundaries.md` — src-tauri module list
- `docs/adr/0010-webview-trust-boundary.md` — CSP strings, freezePrototype, asset protocol
- `docs/adr/0011-app-identifier-and-release-repo.md`
- `docs/ipc-contract.md`

## Outputs

- `src-tauri/src/main.rs` — create
- `src-tauri/src/lib.rs` — create
- `src-tauri/build.rs` — create
- `src-tauri/tauri.conf.json` — create
- `src-tauri/icons/**` — create: placeholder set from `tauri icon`
- `src-tauri/src/state.rs` — create: placeholder
- `src-tauri/src/prefs.rs` — create: placeholder
- `src-tauri/src/watcher.rs` — create: placeholder
- `src-tauri/src/events.rs` — create: placeholder
- `src-tauri/src/tray.rs` — create: placeholder
- `src-tauri/src/menu.rs` — create: placeholder
- `src-tauri/src/window.rs` — create: placeholder
- `src-tauri/src/updater.rs` — create: placeholder
- `src-tauri/src/platform/mod.rs` — create: placeholder
- `src-tauri/src/platform/windows.rs` — create: placeholder
- `src-tauri/src/platform/macos.rs` — create: placeholder
- `src-tauri/src/platform/linux.rs` — create: placeholder
- `src-tauri/src/commands/mod.rs` — create: placeholder
- `src-tauri/src/commands/app.rs` — create: placeholder
- `src-tauri/src/commands/prefs.rs` — create: placeholder
- `src-tauri/src/commands/sessions.rs` — create: placeholder
- `src-tauri/src/commands/session_data.rs` — create: placeholder
- `src-tauri/src/commands/history.rs` — create: placeholder
- `src-tauri/src/commands/config_read.rs` — create: placeholder
- `src-tauri/src/commands/config_write.rs` — create: placeholder
- `src-tauri/src/commands/mcp.rs` — create: placeholder
- `src-tauri/src/commands/analytics.rs` — create: placeholder
- `src-tauri/src/commands/tray.rs` — create: placeholder
- `src-tauri/src/commands/search.rs` — create: placeholder
- `src-tauri/src/commands/export.rs` — create: placeholder
- `src-tauri/src/commands/launch.rs` — create: placeholder
- `src-tauri/src/commands/update.rs` — create: placeholder

## Acceptance criteria

- [ ] `lib.rs` registers the opener, dialog, clipboard-manager, store and autostart plugins; setup calls `watcher::init`, `tray::init`, `menu::init`, `window::init`, `updater::init` (no-ops for now).
- [ ] `tauri.conf.json` sets productName `Heimdall`, the ADR 0011 identifier, `version: "../package.json"`, a `main` window 1280×820 with minimum 900×600, `app.security.csp` and `devCsp` equal to the ADR 0010 strings, `freezePrototype: true`, asset protocol disabled, `withGlobalTauri: false`, `frontendDist: ../dist`, and devUrl on port 1420.
- [ ] `npm run tauri dev` opens a window on each OS; the progress-log entry records the three manual checks.
- [ ] `cargo clippy --workspace -- -D warnings` passes.
- [ ] The production `csp` contains neither `unsafe-eval` nor `unsafe-inline`; `devCsp` contains no `unsafe-eval`.

## Dependencies

- M1-01 — Create the Cargo workspace, crate manifests and supply-chain policy
- M1-02 — Create package.json with the full frontend toolchain
- M1-04 — Scaffold the Vite + Vue 3 + TypeScript + Tailwind v4 frontend
- M0-02 — Record the final app identifier and Linux package name

## Owning agent

rust-engineer

## Reviewer agent

security-engineer, security-auditor, deployment-engineer

## Size

M

## Context

Stub rule (port-plan §5.4): a skeleton task may create placeholder files owned by later tasks. A placeholder holds only a module doc comment and the minimal signatures the wiring needs. The later task owns the file outright.

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
