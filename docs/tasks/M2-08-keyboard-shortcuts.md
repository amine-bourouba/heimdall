# M2-08 · Keyboard navigation, global shortcuts and app actions

> Milestone M2 · Workstream: Vue shell and design system · Scope: MVP

## Goal

One composable reproduces KeyboardNavigationModifier (arrow navigation, Enter, layered and input-aware Escape, focus trap), and a shortcut registry binds every A.7 shortcut with platform-aware labels.

## Inputs

- `$POIROT/Poirot/Sources/Views/Components/KeyboardNavigationModifier.swift`
- `$POIROT/Poirot/Sources/App/ContentView.swift:36-50, 440-460`
- `$POIROT/Poirot/Sources/App/PoirotApp.swift:47-104`
- `docs/parity-matrix.md` — section J

## Outputs

- `src/composables/useKeyboardNav.ts` — create
- `src/composables/useShortcuts.ts` — create
- `src/composables/useAppActions.ts` — create
- `src/components/ShortcutLabel.vue` — create
- `src/composables/useKeyboardNav.spec.ts` — create
- `src/composables/useShortcuts.spec.ts` — create

## Acceptance criteria

- [ ] Escape order matches KeyboardNavigationModifier.swift:52-72: session search, tool filter, config detail, session selection; a focused input is blurred first.
- [ ] Mod+K, Mod+[, Mod+], Mod+T, Mod+= / Mod+- / Mod+0, Mod+? and Mod+, dispatch named app actions; Mod is ⌘ on macOS and Ctrl elsewhere.
- [ ] `useAppActions` is the single dispatcher shared by shortcuts, the macOS menu (`menu-action` events) and the in-app menu.

## Dependencies

- M2-05 — Prefs, theme, toasts and platform stores
- M2-06 — Router, navigation history and page placeholders

## Owning agent

vue-expert

## Reviewer agent

architect-reviewer, accessibility-tester

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
