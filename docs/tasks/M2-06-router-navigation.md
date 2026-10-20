# M2-06 · Router, navigation history and page placeholders

> Milestone M2 · Workstream: Vue shell and design system · Scope: MVP

## Goal

Every screen is a Vue Router route with native back/forward history reproducing the Swift reference app's navigation semantics, and every feature has a placeholder page and an availability flag.

## Inputs

- `$SWIFT_REF/Poirot/Sources/App/AppState.swift:82-190`
- `$SWIFT_REF/Poirot/Sources/Models/Provider/NavigationItem.swift`
- `$SWIFT_REF/PoirotTests/Providers/NavigationItemTests.swift`
- `$SWIFT_REF/PoirotTests/App/AppStateTests.swift (navigation cases)`
- `docs/port-plan.md` — decision P-3

## Outputs

- `src/router/index.ts` — create
- `src/router/routes.ts` — create
- `src/composables/useNavigation.ts` — create
- `src/composables/useNavigation.spec.ts` — create
- `src/main.ts` — modify: install Pinia and the router
- `src/features/session/SessionsPage.vue` — create: placeholder
- `src/features/todos/TodosPage.vue` — create: placeholder
- `src/features/history/HistoryPage.vue` — create: placeholder
- `src/features/plans/PlansPage.vue` — create: placeholder
- `src/features/memory/MemoryPage.vue` — create: placeholder
- `src/features/analytics/AnalyticsPage.vue` — create: placeholder
- `src/features/settings/SettingsPage.vue` — create: placeholder
- `src/features/help/HelpPage.vue` — create: placeholder
- `src/features/about/AboutDialog.vue` — create: placeholder
- `src/features/config/commands/CommandsPage.vue` — create: placeholder
- `src/features/config/skills/SkillsPage.vue` — create: placeholder
- `src/features/config/output-styles/OutputStylesPage.vue` — create: placeholder
- `src/features/config/hooks/HooksPage.vue` — create: placeholder
- `src/features/config/mcp/McpServersPage.vue` — create: placeholder
- `src/features/config/plugins/PluginsPage.vue` — create: placeholder
- `src/features/config/models/ModelsPage.vue` — create: placeholder
- `src/features/config/subagents/SubAgentsPage.vue` — create: placeholder
- `src/features/session/index.ts` — create: placeholder
- `src/features/todos/index.ts` — create: placeholder
- `src/features/history/index.ts` — create: placeholder
- `src/features/plans/index.ts` — create: placeholder
- `src/features/memory/index.ts` — create: placeholder
- `src/features/analytics/index.ts` — create: placeholder
- `src/features/settings/index.ts` — create: placeholder
- `src/features/help/index.ts` — create: placeholder
- `src/features/about/index.ts` — create: placeholder
- `src/features/config/commands/index.ts` — create: placeholder
- `src/features/config/skills/index.ts` — create: placeholder
- `src/features/config/output-styles/index.ts` — create: placeholder
- `src/features/config/hooks/index.ts` — create: placeholder
- `src/features/config/mcp/index.ts` — create: placeholder
- `src/features/config/plugins/index.ts` — create: placeholder
- `src/features/config/models/index.ts` — create: placeholder
- `src/features/config/subagents/index.ts` — create: placeholder
- `src/features/tray/index.ts` — create: placeholder

## Acceptance criteria

- [ ] Hash history; session route `/sessions/:projectId/:sessionId?`; config routes `/config/:kind` and `/config/:kind/:scope/:slug`.
- [ ] `useNavigation` exposes canGoBack/canGoForward from `history.state`; cache hydration uses `router.replace`, so the carried-over 'skip re-push' and 'trim forward history' cases pass.
- [ ] Carried-over NavigationItemTests pass (ids, order, labels).
- [ ] The sidebar list reads each feature's `index.ts` `available` flag, so later features stay hidden until their task flips it.

## Dependencies

- M2-03 — Typed IPC client wrapper and test helpers
- M2-05 — Prefs, theme, toasts and platform stores

## Owning agent

vue-expert

## Reviewer agent

architect-reviewer

## Size

M

## Context

Stub rule (port-plan §5.4): this task creates placeholder files that later tasks own. A placeholder holds only a minimal component that renders nothing or an empty state. The later task replaces it completely.

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
