# M3-11 · Shared config list, detail, toolbar, picker and skeleton

> Milestone M3 · Workstream: Feature screens · Scope: MVP

## Goal

Shared config building blocks give every config screen the same list/grid layouts, filter field, project picker, detail view with copy, badges, empty state and skeleton.

## Inputs

- `$SWIFT_REF/Poirot/Sources/Views/Config/ConfigBadge.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Config/ConfigDetailToolbar.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Config/ConfigEmptyState.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Config/ConfigFilterField.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Config/ConfigItemDetailView.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Config/ConfigLayoutToolbar.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Config/ConfigProjectPicker.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Config/ConfigScreenHeader.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Config/ConfigSkeletonView.swift`
- `docs/spec/config-screens.md`
- `$SWIFT_REF/PoirotTests/App/AppStateTests.swift` — effectiveConfigProjectPath and configProjectName cases
- `docs/spec/test-carryover.md`

## Outputs

- `src/features/config/store.ts` — create
- `src/features/config/store.spec.ts` — create
- `src/features/config/shared/{ConfigListLayout,ConfigItemDetail,ConfigDetailToolbar,ConfigBadge,ConfigEmptyState,ConfigLayoutToolbar,ConfigProjectPicker,ConfigScreenHeader,ConfigSkeleton}.vue` — create

## Acceptance criteria

- [ ] The project picker offers known projects plus a folder chosen via `pick_config_project`; the choice persists in `configProjectPath`.
- [ ] Per-screen layout persists in `configLayouts`.
- [ ] The detail toolbar copies name and content; editor and reveal buttons come later (M3-55).

## Dependencies

- M0-09 — Behaviour spec for the configuration screens, forms and MCP wizard
- M1-44 — Config read commands
- M2-07 — Shared components
- M2-09 — App shell: split view, sidebar with live counts, toolbar, overlays
- M2-10 — renderMarkdown, Shiki highlighting and the single v-html component
- M0-06 — Map every PoirotTests case to its Heimdall destination
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
