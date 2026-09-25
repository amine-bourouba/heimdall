# M0-08 · Behaviour spec for the session browser and conversation view

> Milestone M0 · Workstream: Legacy parity extraction · Scope: Infra

## Goal

A spec captures every numeric threshold, state and action of the session views so Vue tasks can reach parity without reading Swift.

## Inputs

- `$POIROT/Poirot/Sources/Views/Session/SessionsNavigationView.swift`
- `$POIROT/Poirot/Sources/Views/Project/ProjectSessionsView.swift`
- `$POIROT/Poirot/Sources/Views/Session/SessionDetailView.swift`
- `$POIROT/Poirot/Sources/Views/Session/SessionToolbarContent.swift`
- `$POIROT/Poirot/Sources/Views/Session/ToolBlockView.swift`
- `$POIROT/Poirot/Sources/Views/Session/ThinkingBlockView.swift`
- `$POIROT/Poirot/Sources/Views/Session/SystemContentView.swift`
- `$POIROT/Poirot/Sources/Views/Session/EditDiffView.swift`
- `$POIROT/Poirot/Sources/Views/Session/ConversationContextCard.swift`
- `$POIROT/Poirot/Sources/Views/Session/SessionTimelineView.swift`
- `$POIROT/Poirot/Sources/Views/Session/SessionFacetsCard.swift`
- `$POIROT/Poirot/Sources/Views/Session/SessionTodosView.swift`
- `$POIROT/Poirot/Sources/Views/Session/FileHistoryView.swift`
- `$POIROT/Poirot/Sources/Views/Session/DebugLogView.swift`
- `$POIROT/Poirot/Sources/Views/Session/ExportOptionsView.swift`
- `$POIROT/Poirot/Sources/Views/Session/SessionSkeletonView.swift`

## Outputs

- `docs/spec/session-views.md` — create

## Acceptance criteria

- [ ] States every threshold with its source line: page size 20 (SessionDetailView.swift:7), tool truncation 80 lines (ToolBlockView.swift:8), thinking truncation 50 lines (ThinkingBlockView.swift:6), diff cap 5000 lines (EditDiffView.swift:22), toast 4 s / 10 s with URL (ToastOverlay.swift:22).
- [ ] Lists every toolbar and context-menu action with its effect and the preference keys it reads.
- [ ] Describes each skeleton layout well enough to build it as a bespoke layout.

## Dependencies

- M0-05 — Exhaustive inventory of every ~/.claude path Poirot reads or writes

## Owning agent

legacy-modernizer (consults swift-expert, who only reads the Swift app)

## Reviewer agent

qa-expert

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
