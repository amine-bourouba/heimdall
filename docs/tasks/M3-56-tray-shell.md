# M3-56 · Tray icon and tray window

> Milestone M3 · Workstream: Tauri shell / OS integration · Scope: Later

## Goal

A tray icon (hidden when `showMenuBarIcon` is false) opens a small tray window on Windows and macOS, and a menu with recent sessions on Linux.

## Inputs

- `$POIROT/Poirot/Sources/App/PoirotApp.swift:124-128`
- `$POIROT/Poirot/Sources/Views/MenuBar/MenuBarView.swift`
- `docs/adr/0003-platform-abstraction.md` — Tray
- `docs/adr/0010-webview-trust-boundary.md` — Capabilities

## Outputs

- `src-tauri/src/tray.rs` — modify

## Acceptance criteria

- [ ] The tray window is created in Rust with label `tray`, no decorations, skip-taskbar, hidden on blur, positioned from the tray icon rectangle.
- [ ] Linux uses a native menu: Open Heimdall, up to 10 recent sessions, Quit; GNOME needs the AppIndicator extension (documented in docs/install.md later).
- [ ] Toggling `showMenuBarIcon` shows or hides the icon live.

## Dependencies

- M1-48 — Per-window capability files
- M3-30 — Analytics and tray commands
- M3-57 — Separate frontend entry for the tray window

## Owning agent

rust-engineer

## Reviewer agent

security-engineer, security-auditor, deployment-engineer

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
