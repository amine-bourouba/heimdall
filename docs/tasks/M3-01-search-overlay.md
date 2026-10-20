# M3-01 · Universal search overlay

> Milestone M3 · Workstream: Feature screens · Scope: MVP

## Goal

A Ctrl/Cmd+K overlay searches all 16 categories through `search`, shows grouped and highlighted results, supports fuzzy and exact modes, and runs every SearchAction from the keyboard.

## Inputs

- `$SWIFT_REF/Poirot/Sources/Views/Search/SearchOverlayView.swift`
- `docs/spec/search-overlay.md`
- `docs/ipc-contract.md` — commands/search.rs

## Outputs

- `src/features/search/store.ts` — create
- `src/features/search/SearchOverlay.vue` — modify placeholder
- `src/features/search/SearchResultRow.vue` — create
- `src/features/search/store.spec.ts` — create
- `src/features/search/SearchOverlay.spec.ts` — create

## Acceptance criteria

- [ ] Empty query shows all available nav screens plus the 5 most recent sessions (`list_recent_sessions(5)`).
- [ ] Results group by category in Swift order, at most 5 per group, with UTF-16 highlight ranges rendered as marks.
- [ ] The fuzzy/exact toggle persists in `globalSearchMode`.
- [ ] Quick actions 'Add MCP Server' and 'Toggle Menu Bar Icon' appear only when their feature's `index.ts` reports it available.
- [ ] ↑/↓ move, Return opens, Esc closes; focus is trapped while open.

## Dependencies

- M0-07 — Behaviour spec for universal search
- M1-46 — Search command
- M2-08 — Keyboard navigation, global shortcuts and app actions
- M2-09 — App shell: split view, sidebar with live counts, toolbar, overlays
- M2-14 — Sessions store
- M2-30 — M2 exit gate review

## Owning agent

frontend-developer (with typescript-pro for typed IPC)

## Reviewer agent

code-reviewer, accessibility-tester

## Size

L

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
