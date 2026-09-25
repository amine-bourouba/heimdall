# M2-09 · App shell: split view, sidebar with live counts, toolbar, overlays

> Milestone M2 · Workstream: Vue shell and design system · Scope: MVP

## Goal

The app shell reproduces Poirot's split view with a sidebar of live counts, back/forward and contextual toolbars, an in-app menu for Windows and Linux, and mount points for toasts, search, onboarding and shortcut help.

## Inputs

- `$POIROT/Poirot/Sources/App/ContentView.swift:82-157`
- `$POIROT/Poirot/Sources/Views/Components/SidebarView.swift`
- `docs/port-plan.md` — decision P-4 (window chrome)

## Outputs

- `src/App.vue` — modify
- `src/layout/AppShell.vue` — create
- `src/layout/Sidebar.vue` — create
- `src/layout/Toolbar.vue` — create
- `src/layout/AppMenu.vue` — create
- `src/layout/Sidebar.spec.ts` — create
- `src/features/search/SearchOverlay.vue` — create: placeholder
- `src/features/onboarding/Onboarding.vue` — create: placeholder
- `src/features/help/ShortcutHelp.vue` — create: placeholder

## Acceptance criteria

- [ ] Sidebar width 220–300 px (ideal 260); items and order from `src/router/routes.ts`; counts from `get_sidebar_counts` refresh on `data-changed`.
- [ ] The toolbar's back/forward buttons are bound to `useNavigation` and disabled when unavailable.
- [ ] On Windows and Linux an in-app menu offers About, Check for Updates, Settings, Help and Keyboard Shortcuts through `useAppActions`; on macOS the layout honours `--titlebar-inset`.
- [ ] Landmarks and roles: navigation, main, toolbar; the sidebar is keyboard navigable with `useKeyboardNav`.

## Dependencies

- M2-06 — Router, navigation history and page placeholders
- M2-07 — Shared components
- M2-08 — Keyboard navigation, global shortcuts and app actions
- M2-12 — Per-OS window chrome and navigation guard
- M2-13 — Native macOS app menu emitting menu-action events

## Owning agent

vue-expert

## Reviewer agent

architect-reviewer, accessibility-tester

## Size

M

## Context

Stub rule (port-plan §5.4): this task creates placeholder files that later tasks own. A placeholder holds only a minimal component that renders nothing or an empty state. The later task replaces it completely.

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
