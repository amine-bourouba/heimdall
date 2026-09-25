# M2-05 · Prefs, theme, toasts and platform stores

> Milestone M2 · Workstream: Vue shell and design system · Scope: MVP

## Goal

Pinia stores own preferences, theme and font scale, toasts and platform facts, and pass the carried-over app-state tests.

## Inputs

- `$POIROT/Poirot/Sources/App/AppState.swift:1-31, 72-80, 398-440`
- `$POIROT/Poirot/Sources/Views/Components/ToastOverlay.swift`
- `$POIROT/PoirotTests/App/ColorThemeTests.swift`
- `$POIROT/PoirotTests/App/AppStateTests.swift`
- `docs/spec/test-carryover.md`
- `docs/adr/0007-preferences-storage-and-migration.md`

## Outputs

- `src/stores/prefs.ts` — create
- `src/stores/theme.ts` — create
- `src/stores/toasts.ts` — create
- `src/stores/platform.ts` — create
- `src/stores/prefs.spec.ts` — create
- `src/stores/theme.spec.ts` — create
- `src/stores/toasts.spec.ts` — create

## Acceptance criteria

- [ ] Carried-over ColorThemeTests and the toast and font-scale cases of AppStateTests pass as Vitest specs.
- [ ] Font scale steps by 0.05 within 0.75–1.5 and persists through `update_prefs`.
- [ ] Toasts are a single-item queue that auto-dismisses after 4 s, or 10 s when a URL is attached; URL clicks go through `open_external_url`.
- [ ] The platform store exposes the OS and the modifier label (⌘ on macOS, Ctrl elsewhere).

## Dependencies

- M1-39 — Preferences storage and preference commands
- M1-40 — App commands: app info, sidebar counts, CLI detection
- M2-02 — Theme tokens for 20 palettes and the Tailwind v4 theme
- M2-03 — Typed IPC client wrapper and test helpers

## Owning agent

vue-expert

## Reviewer agent

architect-reviewer

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
