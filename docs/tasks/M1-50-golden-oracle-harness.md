# M1-50 · Swift golden-output harness for the reference app

> Milestone M1 · Workstream: Legacy parity extraction · Scope: MVP

## Goal

A Swift test file, run inside a throwaway checkout of upstream the Swift reference app, writes the Swift app's computed outputs for each fixture tree as JSON shaped like Heimdall's DTOs.

## Inputs

- `$SWIFT_REF/Poirot/Sources/Services/SessionLoader.swift`
- `$SWIFT_REF/Poirot/Sources/Services/TranscriptParser.swift`
- `$SWIFT_REF/Poirot/Sources/Services/StatsComputer.swift`
- `$SWIFT_REF/Poirot/Sources/Services/StatsCacheLoader.swift`
- `$SWIFT_REF/Poirot/Sources/Services/HistoryLoader.swift`
- `$SWIFT_REF/Poirot/Sources/Services/TodoLoader.swift`
- `$SWIFT_REF/Poirot/Sources/Services/DebugLogLoader.swift`
- `$SWIFT_REF/Poirot/Sources/Services/FacetsLoader.swift`
- `$SWIFT_REF/Poirot/Sources/Services/FileHistoryLoader.swift`
- `$SWIFT_REF/Poirot/Sources/Services/Export/SessionExporter.swift`
- `$SWIFT_REF/Poirot/Sources/Utilities/LineDiff.swift`
- `$SWIFT_REF/Poirot/Sources/Models/ContextUsage.swift`
- `fixtures/README.md`
- `docs/ipc-contract.md` — §5 Types
- `docs/adr/0001-repo-layout-and-swift-reference.md`

## Outputs

- `tools/golden-oracle/GoldenOracleTests.swift` — create
- `tools/golden-oracle/README.md` — create

## Acceptance criteria

- [ ] Given `GOLDEN_FIXTURES` and `GOLDEN_OUT`, the suite writes canonical JSON (sorted keys, UTC ISO dates, camelCase) for: projects and summaries, full parses, header and summary tiers, stats (computed and cache-staleness verdict), history, todos, debug-log pages, facets, file history, context usage, export Markdown, line diffs.
- [ ] Loaders that only read the real home directory (ClaudeConfigLoader, SettingsWriter) are skipped unless a `CFFIXED_USER_HOME` override is proven to work (assumption A-9); the README records which.
- [ ] The file is never added to $SWIFT_REF; README documents the copy-in procedure the workflow uses.

## Dependencies

- M0-05 — Exhaustive inventory of every ~/.claude path the Swift reference app reads or writes
- M0-06 — Map every PoirotTests case to its Heimdall destination
- M0-07 — Behaviour spec for universal search
- M0-08 — Behaviour spec for the session browser and conversation view
- M0-09 — Behaviour spec for the configuration screens, forms and MCP wizard
- M0-10 — Behaviour spec for analytics, exports and the context ring
- M0-11 — Extract PoirotTests inline fixtures into a shared synthetic ~/.claude tree
- M0-12 — Convert ScreenshotData.swift into a deterministic fixture tree
- M0-14 — Capture a scrubbed real Windows ~/.claude tree
- M0-15 — Capture a scrubbed real Linux ~/.claude tree
- M1-08 — Serde models for messages, sessions and projects
- M1-09 — Serde models for stats, history, todos, facets, debug log and file history

## Owning agent

swift-expert

## Reviewer agent

legacy-modernizer, qa-expert

## Size

M

## Context

Write task for swift-expert, allowed because the repo-layout decision is settled (separate project) and the read-only extraction tasks M0-05 to M0-10 come first.

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
