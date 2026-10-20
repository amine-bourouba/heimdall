# M4-08 · Per-OS developer setup scripts

> Milestone M4 · Workstream: Release and packaging · Scope: MVP

## Goal

One setup script per OS installs the toolchain, the build script produces installers, and a guide records measured setup times and troubleshooting.

## Inputs

- `docs/port-plan.md` — §10 Local development

## Outputs

- `scripts/setup-macos.sh` — create
- `scripts/setup-linux.sh` — create
- `scripts/setup-windows.ps1` — create
- `package.json` — modify: build scripts
- `docs/dev-setup.md` — create

## Acceptance criteria

- [ ] setup-linux.sh supports apt, dnf and pacman and installs webkit2gtk-4.1 and libayatana-appindicator3 development packages.
- [ ] setup-windows.ps1 installs rustup and Node LTS and checks for WebView2 and MSVC build tools.
- [ ] `npm run build` runs `tauri build`; output lands under `src-tauri/target/release/bundle/`.
- [ ] dev-setup.md records the measured time per OS from a clean machine and WebView2/webkit2gtk troubleshooting.

## Dependencies

- M1-02 — Create package.json with the full frontend toolchain
- M2-31 — Add the WebDriver client for Linux real-app smoke tests

## Owning agent

build-engineer

## Reviewer agent

architect-reviewer, security-auditor

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
