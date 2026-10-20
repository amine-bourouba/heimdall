# M2-13 · Native macOS app menu emitting menu-action events

> Milestone M2 · Workstream: Tauri shell / OS integration · Scope: MVP

## Goal

macOS gets a native app menu (About, Check for Updates…, Settings…, Edit, font size, Help, Keyboard Shortcuts) that emits `menu-action` events; Windows and Linux get no native menu bar.

## Inputs

- `$SWIFT_REF/Poirot/Sources/App/PoirotApp.swift:47-104`
- `docs/ipc-contract.md` — §7 Events

## Outputs

- `src-tauri/src/menu.rs` — modify

## Acceptance criteria

- [ ] The Edit menu provides cut, copy, paste and select all (needed for text fields on macOS).
- [ ] Each custom item emits `menu-action` with the action name defined in the contract.

## Dependencies

- M1-03 — Scaffold the Tauri app crate with final identity, CSP and module stubs
- M1-37 — Freeze the IPC contract as compiled stubs and generated TypeScript bindings

## Owning agent

rust-engineer

## Reviewer agent

security-engineer, deployment-engineer

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
