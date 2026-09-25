# M4-01 · Per-OS bundle configuration

> Milestone M4 · Workstream: Release and packaging · Scope: MVP

## Goal

Per-OS Tauri config files produce an NSIS installer on Windows, deb, rpm and AppImage on Linux, and an ad-hoc-signed Apple Silicon app and DMG on macOS, with the Linux package and binary named per ADR 0011.

## Inputs

- `docs/adr/0011-app-identifier-and-release-repo.md`
- `docs/port-plan.md` — decisions P-9 and P-11
- `src-tauri/tauri.conf.json`

## Outputs

- `src-tauri/tauri.windows.conf.json` — create
- `src-tauri/tauri.linux.conf.json` — create
- `src-tauri/tauri.macos.conf.json` — create

## Acceptance criteria

- [ ] Windows: `bundle.targets: ["nsis"]`, per-user install mode, WebView2 download bootstrapper.
- [ ] Linux: targets deb, rpm, appimage; deb depends on libwebkit2gtk-4.1-0 and libayatana-appindicator3-1; rpm dependencies verified on Fedora; `dpkg -I` and `rpm -qip` show the ADR 0011 package name, not `heimdall`.
- [ ] macOS: targets app and dmg, `aarch64-apple-darwin` only, ad-hoc signing identity `-`, no notarisation, `minimumSystemVersion` "13.0"; `codesign -dv` reports adhoc.
- [ ] `npm run tauri build` succeeds on each OS and artifact sizes are recorded in the progress log.

## Dependencies

- M0-02 — Record the final app identifier and Linux package name
- M1-03 — Scaffold the Tauri app crate with final identity, CSP and module stubs
- M2-12 — Per-OS window chrome and navigation guard

## Owning agent

deployment-engineer

## Reviewer agent

architect-reviewer, security-auditor

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
