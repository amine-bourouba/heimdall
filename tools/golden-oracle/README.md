# golden-oracle

`GoldenOracleTests.swift` is the golden-output harness for the Swift reference app (task M1-50).
It runs real Poirot loaders and services against Heimdall's own fixture trees and writes their
outputs as canonical JSON, so Rust tests (M1-53) can prove parity byte-for-byte.

**This file is never added to `$POIROT`.** `$POIROT` (`C:\Users\pc\Desktop\tech\poirot`, pinned at
upstream commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`, ADR 0001) stays a read-only reference
forever. This harness lives here instead, and gets *copied into* a throwaway checkout of upstream
Poirot at the same pinned SHA.

## The copy-in procedure (what M1-51's workflow does)

1. Clone `https://github.com/a7t-ai/poirot` at `a29dd6ee0e0133ef26bd067c164c66b160e13d86` into a
   throwaway directory (a fresh `actions/checkout` step or `git clone` into a temp path — never
   `$POIROT` itself, and never committed anywhere).
2. Copy `tools/golden-oracle/GoldenOracleTests.swift` from *this* repo into that checkout's test
   target (e.g. `PoirotTests/GoldenOracleTests.swift`), so it compiles as a member of Poirot's own
   test module and can call its `internal`-visibility types directly.
3. Copy `fixtures/synthetic/`, `fixtures/screenshot/`, `fixtures/windows/`, `fixtures/linux/` from
   this repo into a scratch directory — this becomes `$GOLDEN_FIXTURES`.
4. Run, on a macOS runner (a Mac is required — `SessionExporter.swift` imports `AppKit`, and the
   fixture-home-directory trick this harness relies on is Foundation-on-Darwin-specific):
   ```
   GOLDEN_FIXTURES=/path/to/scratch/fixtures GOLDEN_OUT=/path/to/scratch/golden-out \
     xcodebuild test -project Poirot.xcodeproj -scheme Poirot \
     -only-testing:PoirotTests/GoldenOracleTests
   ```
   (or the equivalent `xcodebuild test-without-building` / `swift test` invocation the workflow
   settles on — whichever Poirot's own existing CI already uses for `PoirotTests`).
5. Copy `$GOLDEN_OUT/**` into `fixtures/golden/` in *this* repo (M1-52's job) and discard the
   throwaway checkout entirely.
6. Every test method throws `XCTSkip` if `GOLDEN_FIXTURES`/`GOLDEN_OUT` are unset, so running the
   full `PoirotTests` suite without them (e.g. a Poirot contributor's own local run) never fails —
   it just skips this one file's tests.

## Why fixture trees stand in for a real home directory

Every loader this harness drives (`SessionLoader`, `TranscriptParser`, `StatsComputer`,
`StatsCacheLoader`, `HistoryLoader`, `TodoLoader`, `DebugLogLoader`, `FacetsLoader`,
`FileHistoryLoader`) takes an **explicit, overridable path** in its own `init` (e.g.
`SessionLoader(claudeProjectsPath:)`), falling back to the real `~/.claude/...` only when that
argument is `nil`. The harness always supplies the fixture path, so none of these ever touch the
machine's real home directory.

**`ClaudeConfigLoader` and `SettingsWriter` are the exception — and are skipped entirely** (AC2).
Both are namespace `enum`s with no `init` at all; every function hardcodes
`FileManager.default.homeDirectoryForCurrentUser` inline, with no override parameter anywhere in
either file (confirmed by reading both files directly, not assumed). Assumption A-9 asks whether
`CFFIXED_USER_HOME` (a Foundation-on-Darwin environment variable some sandboxed test setups use to
redirect `homeDirectoryForCurrentUser`) would work around this. **This harness does not attempt
it** — proving it needs a real macOS process to observe whether `homeDirectoryForCurrentUser`
actually moves, and this repo has no macOS or Swift toolchain available anywhere it can execute
Swift code (confirmed: this task was written on a Windows machine with no Swift installed and no
Mac available). If a future task wants config-loader parity, it should first spend five minutes on
a real Mac confirming `CFFIXED_USER_HOME` actually works before building anything on top of it —
this repo's own AC2 permission to skip is exactly for this situation.

Concretely, this means `fixtures/golden/` will never contain a config/MCP-server/settings dump —
and it doesn't need to: AC1's own list of required outputs (below) never mentions one.

## Fixture sets this harness reads

Every set `fixtures/README.md` documents with a `<set>/claude-home/**` + `<set>/claude.json` pair:
`synthetic`, `screenshot`, `windows`, `linux`. (`fixtures/golden/`, `fixtures/dto/` and
`fixtures/security/` are not fixture *homes* in this sense and are never read by this harness.) A
set missing from `$GOLDEN_FIXTURES` at run time is silently skipped, not an error — useful for a
partial local run, though the real CI workflow should always copy in all four.

## Output layout under `$GOLDEN_OUT`

```
<set>/
  projects.json                                  — [ProjectSummary], from discoverProjects()
  sessions/<projectId>/summaries.json             — that project's own [SessionSummary] (summary tier, incl. sub-agents)
  sessions/<projectId>/<sessionId>/full.json      — full parse (TranscriptParser.parse)
  sessions/<projectId>/<sessionId>/header.json    — header tier (TranscriptParser.parseHeader)
  sessions/<projectId>/<sessionId>/summary.json   — summary tier, run directly (TranscriptParser.parseSummary)
  sessions/<projectId>/<sessionId>/context.json   — ContextUsage.from(messages), full-parse tier only
  sessions/<projectId>/<sessionId>/export.md      — SessionExporter.toMarkdown, raw text (no DTO shape)
  stats.json                                      — { computed, cached, latestSessionDay, isStale, chosenSource }
  history.json                                    — [HistoryEntry]
  todos.json                                      — { [sessionId]: [TodoItem] }
  facets.json                                     — { [sessionId]: SessionFacets }
  debug/<sessionId>.json                          — { entries, offset: 0, total } — the whole log as one page
  file-history/<sessionId>.json                   — [FileHistoryEntry], only written when non-empty
  diffs/<sessionId>/<fileName>-v<N>-v<M>.json      — LineDiff.diff() over every consecutive real version pair
```

`sessions/<projectId>/<sessionId>/{full,header,summary,context}.json` only exist for **top-level**
session files (direct children of a project directory: main sessions and `agent-*.jsonl` files
written next to them) — not for the nested `<parentSession>/subagents/*.jsonl` files.
`testProjectsAndSummaries`'s own `sessions/<projectId>/summaries.json` already covers subagents
structurally (`discoverProjects()` walks `subagents/` internally via `loadSubAgentSessions`), so
nothing about sub-agent sessions is missing from the golden set as a whole — it just isn't
duplicated at the per-file granularity the other tiers use.

## Known gaps and deviations from a literal ipc-contract.md §5 port

Recorded here rather than guessed at silently, per this task's own acceptance criteria:

- **Fields with no Poirot-side equivalent are omitted, not fabricated:** `ProjectSummary.pathSource`
  (Poirot's own `projectPath` is either the sessions-index's `originalPath` or a plain dash→slash
  decode — never literally "the first transcript's own `cwd`", so none of the three ipc-contract
  values is actually what Poirot computes), `hasMemory`/`hasDebugLog`/`hasFileHistory`/`hasTodos`
  (Rust-side aggregates across *other* loaders, not something any one Poirot call returns),
  `SessionDetail.toolNames`/`skipped`, and `ContentBlock.text.system` (system-reminder-tag parsing
  — `SystemSegment` has no Poirot equivalent at all; Poirot's text blocks are plain, unparsed
  strings).
- **`SessionSummary.tokens` is only a real breakdown at the full-parse tier.** Poirot's `Session`
  model stores a single `totalTokens: Int`, never a per-field input/output/cache breakdown, at the
  header or summary tiers (`messages: []` there, so there is nothing to sum). `GoldenSession`
  exposes `totalTokens` (always accurate) and `tokens` (`null` at header/summary, a real summed
  breakdown only when `includeMessages` is true).
- **Header tier's `turnCount` is always `0`.** Poirot's own `Session.turnCount` computed property
  falls back to `messages.filter(user).count` when `cachedTurnCount` is nil, and `parseHeader`
  never sets `cachedTurnCount` — so this is Poirot's real behavior, not a harness bug.
- **`ContentBlock.edit` is inlined**, not nested: `oldString`/`newString` sit directly on the block
  instead of under an `edit: {...} | null` object. Same information, simpler to build from a Swift
  enum case.
- **`FileVersion.versionId` ← Poirot's `backupFileName`** (the real on-disk unique identifier for a
  version; Poirot has no separate concept named "version id").
- **`inputPretty`** is a sorted `key=value` line-per-entry rendering of `ToolUse.input`
  (`[String: String]`, already stringified by `TranscriptParser`) — a reasonable canonical
  "pretty" form, not a port of any specific Poirot view's own formatting (SwiftUI views format
  tool input several different ways depending on the tool; there is no single canonical source).
- **`DiffResult.truncated` is always `false`.** `LineDiff.swift` has no line cap at all; Rust's own
  5000-line truncation (per ipc-contract's comment) has no Poirot equivalent to compare against. If
  a fixture's diff ever exceeds 5000 lines, that specific comparison needs listing in
  `fixtures/golden/KNOWN_DIFFERENCES.md` (M1-53), not fixed here.
- **`DebugLogEntry.level` and `TodoItem.status` are case/format-mapped, not re-encoded blind.**
  Poirot's on-disk `Level` raw values are `"DEBUG"/"WARN"/"ERROR"` (uppercase) and `SessionTodo`'s
  disk status is `"in_progress"` (snake_case) — both lowercased/camelCased explicitly to match
  ipc-contract's `"debug"/"warn"/"error"` and `"pending"/"inProgress"/"completed"`.
- **`stats.json`'s `computed.lastComputedDate` is real wall-clock time, not fixed.**
  `StatsComputer.compute` calls `Date()` directly (see `StatsComputer.swift`'s `makeStatsCache()`).
  Two runs of this harness on the same fixtures will differ on this one field every time — M1-53's
  comparison must exclude or normalize it, the same way any "generated at build/run time" field
  would be excluded from a snapshot test elsewhere in this repo.
- **`export.md`'s `**Date:**` header line is locale- and timezone-dependent.**
  `SessionExporter.formattedDate` uses a plain `DateFormatter` with `.dateStyle = .long,
  .timeStyle = .short` and no explicit locale or timezone — it renders in whatever the process
  environment's locale/timezone are. **M1-51's workflow must pin `LANG=en_US.UTF-8` (or set
  `Locale`/`TimeZone` some other way the runner respects) for this file to compare byte-for-byte
  across CI runs**; otherwise this one line will differ between two runs on machines with
  different settings even though nothing about the underlying data changed. Every other line of
  `export.md` is deterministic (plain formatting of already-canonical data).
- **Sorted keys, camelCase, UTC ISO-8601 dates (with fractional seconds) are enforced by never
  reusing a Poirot model's own `Codable` conformance for output.** Several on-disk models
  (`SessionFacets`, `SessionTodo.Status`) have snake_case `CodingKeys` for their *disk* format —
  encoding through that conformance would leak snake_case into golden JSON. Every `Golden*` type in
  the harness is a hand-written `Encodable` wrapper with its own camelCase property names instead,
  populated from the real decoded/computed Swift values. The one exception is `StatsCache` itself
  (and its nested types), which has no custom `CodingKeys` at all — its own property names are
  already exactly camelCase, so `stats.json`'s `computed`/`cached` fields reuse it directly rather
  than duplicating ~10 fields for no reason.

## Retirement

Per ADR 0001 point 4, this file (and `tools/golden-oracle/` entirely) is deleted, along with its
CI workflow (M1-51) and `fixtures/golden/`'s enforcement in the M1/later gates, only once every MVP
parity row is At parity on Windows and Linux and every parser-related `PoirotTests` row is covered
by a passing Rust test (M4-15, M4-16) — a condition, not a date.
