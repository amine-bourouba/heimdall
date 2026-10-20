# M0-05 · Exhaustive inventory of every ~/.claude path the Swift reference app reads or writes

> Milestone M0 · Workstream: Legacy parity extraction · Scope: Infra

## Goal

One document lists every path under `~/.claude` and `~/.claude.json` that the Swift reference app reads, writes, deletes or watches, with its format, the fields consumed, the Swift file:line, and the Heimdall module that will own it.

## Inputs

- `$SWIFT_REF/Poirot/Sources/Services/SessionLoader.swift`
- `$SWIFT_REF/Poirot/Sources/Services/TranscriptParser.swift`
- `$SWIFT_REF/Poirot/Sources/Services/HistoryLoader.swift`
- `$SWIFT_REF/Poirot/Sources/Services/TodoLoader.swift`
- `$SWIFT_REF/Poirot/Sources/Services/DebugLogLoader.swift`
- `$SWIFT_REF/Poirot/Sources/Services/FacetsLoader.swift`
- `$SWIFT_REF/Poirot/Sources/Services/FileHistoryLoader.swift`
- `$SWIFT_REF/Poirot/Sources/Services/StatsCacheLoader.swift`
- `$SWIFT_REF/Poirot/Sources/Services/StatsComputer.swift`
- `$SWIFT_REF/Poirot/Sources/Services/Config/ClaudeConfigLoader.swift`
- `$SWIFT_REF/Poirot/Sources/Services/Config/SettingsWriter.swift`
- `$SWIFT_REF/Poirot/Sources/Services/Config/MCPServerStatusChecker.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Todos/TodosOverviewView.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Search/SearchOverlayView.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Memory/MemoryListView.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Onboarding/OnboardingView.swift`
- `$SWIFT_REF/Poirot/Sources/App/ContentView.swift`
- `$SWIFT_REF/Poirot/Sources/App/AppState.swift`
- `docs/parity-matrix.md` — sections E (reads), F (writes), G (watchers)
- `docs/adr/0002-layer-boundaries.md`

## Outputs

- `docs/spec/claude-home-inventory.md` — create

## Acceptance criteria

- [ ] Every row of parity-matrix sections E, F and G maps to at least one inventory row, shown in a cross-reference table.
- [ ] Every inventory row cites a file:line that exists at commit a29dd6e.
- [ ] Every hit of `grep -rn -E "homeDirectoryForCurrentUser|NSHomeDirectory" $SWIFT_REF/Poirot/Sources` is accounted for.
- [ ] An 'Open discrepancies' section lists at least: sessions-index `originalPath` (TodosOverviewView.swift:372) versus `projectPath` (SessionLoader.swift:239-240); Swift never reads the transcript `cwd` field.

## Dependencies

- None

## Owning agent

legacy-modernizer (consults swift-expert, who only reads the Swift app)

## Reviewer agent

qa-expert

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
