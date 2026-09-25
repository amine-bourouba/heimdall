# fixtures/synthetic

Task M0-11. Standalone fixtures extracted from `$POIROT/PoirotTests` (commit `a29dd6e`), plus one integrated macOS-shaped `~/.claude` tree covering every format in `docs/spec/claude-home-inventory.md` section 1. Rust unit/integration tests and Vitest both read these files directly; nothing here is generated at test time.

Home directory used throughout: `/Users/fixture`. No other absolute home path appears under this tree.

## Layout

- `cases/<SwiftTestFileStem>/<testFunctionName>/` -- one folder per Swift test in `PoirotTests/Services/*.swift` (and the three `PoirotTests/Models/*.swift` files that `JSONDecoder` a JSON string: `SessionFacetsTests`, `SessionTodoTests`, `StatsCacheTests`) that builds a file or a JSON/text literal inline. The folder reproduces the file(s) the test writes, laid out relative to the root the test passes to the loader (or, for tests that decode a literal without touching disk, a single `input.json`/`input.txt`/`input.md` holding that literal's content). Tests that only construct Swift values in code (no file/JSON/text built) are **not** given a folder; the mapping table below lists them as `inline`. See the per-destination tables for exact counts.
- `claude-home/` -- one integrated, realistic `~/.claude` tree (home `/Users/fixture`). Every format in the inventory appears at least once; see the coverage table below.
- `claude.json` -- stands in for `~/.claude.json`: top-level `mcpServers` (a stdio `npx` server with `env` secrets using sentinel values, an `http` server with `url`/`headers`) and `projects["/Users/fixture/src/app"].mcpServers`.
- `project-repo/` -- a project folder outside `~/.claude`: `.mcp.json`, `.claude/settings.json` (model + hooks), `.claude/commands/*.md`, `.claude/skills/<name>/SKILL.md`, `.claude/output-styles/*.md`.
- `corrupt/` -- intentionally unparsable files (truncated `settings.json`, a truncated `claude.json` stand-in, a JSONL with a malformed middle line and a truncated last line, an invalid `stats-cache.json`); see `corrupt/README.md` for what each one exercises.

## Deterministic substitutes for random values

Several Swift tests generate a random `UUID().uuidString` for a session id, message id or record uuid that the test's own assertions never check (only structural fields like counts or grouped content matter). Where the Swift source pins an explicit literal (for example `"a1b2c3d4-e5f6-7890-abcd-ef1234567890"`), the fixture uses that same literal. Where the Swift source calls `UUID().uuidString` with no fixed value, the fixture substitutes a deterministic UUID derived from `uuid5(NAMESPACE_URL, "poirot-test:<file>/<test>/<tag>")`, noted per-case below. Swift's multiline JSON string literals are reproduced as pretty-printed JSON (2-space indent) for readability; JSONL lines use compact JSON with no trailing newline, matching `JSONSerialization.data(withJSONObject:)` and the `.joined(separator: "\n")` the Swift helpers use. Content and field values are byte-for-byte what the Swift test builds; only insignificant JSON whitespace was normalised.

## Intentionally invalid fixtures inside `cases/`

A handful of Services tests assert that a loader **skips** a bad file/line rather than crashing (`..._invalidJSON_returnsNil`, `..._skipsInvalidFiles`, `..._skipsInvalidLines`, `..._skipsInvalidJSON`, `parse_malformedLines_skipsGracefully`). Their fixtures necessarily contain invalid JSON/JSONL content, matching what the Swift test writes. These are excluded from the "every `.json`/`.jsonl` outside `corrupt/` parses" acceptance check by explicit path, not by folder convention:

- `cases/FacetsLoaderTests/loadAllFacets_skipsInvalidFiles/bad-session.json`
- `cases/FacetsLoaderTests/loadFacets_invalidJSON_returnsNil/bad-session.json`
- `cases/HistoryLoaderTests/loadAll_skipsInvalidLines/history.jsonl`
- `cases/TodoLoaderTests/loadTodos_skipsInvalidJSON/abc-1234-agent-bad.json`
- `cases/TranscriptParserTests/parse_malformedLines_skipsGracefully/test-session.jsonl`

## Coverage: `claude-home/` vs. `docs/spec/claude-home-inventory.md` section 1

| Inventory row | Format | Path in `claude-home/` |
|---|---|---|
| R1 | projects/ directory listing | `projects/-Users-fixture-src-app/` |
| R2 | transcript JSONL (text, thinking, tool_use incl. Edit old_string/new_string, tool_result incl. is_error, `<synthetic>` message, sidechain record, summary/system/file-history-snapshot/file-history-delta records, `cwd` on every record) | `projects/-Users-fixture-src-app/11111111-1111-4111-8111-111111111111.jsonl` |
| R3/R4 | sessions-index.json (`entries[].projectPath` and top-level `originalPath`) | `projects/-Users-fixture-src-app/sessions-index.json` |
| R5/R6 | sub-agent JSONL + `.meta.json` | `projects/-Users-fixture-src-app/11111111.../subagents/agent-a9b1c2d3.jsonl` (+ `.meta.json`) |
| R8 | project memory files (`MEMORY.md` + another) | `projects/-Users-fixture-src-app/memory/MEMORY.md`, `memory/architecture.md` |
| R10 | history.jsonl (with pastedContents) | `history.jsonl` |
| R11/R12 | todos (plain and `-agent-` suffixed) | `todos/11111111....json`, `todos/11111111...-agent-a9b1c2d3.json` |
| R13 | debug log (DEBUG/WARN/ERROR, one multi-line entry) | `debug/11111111-1111-4111-8111-111111111111.txt` |
| R14/R15 | file-history backups matching transcript snapshot/delta records | `file-history/11111111.../3f2504e5f2a14f9f@v1` and `@v2` |
| R17 | usage-data/facets/<sessionId>.json | `usage-data/facets/11111111....json` |
| R18 | stats-cache.json | `stats-cache.json` |
| R20 | commands/*.md | `commands/review.md` |
| R21 | plans/*.md | `plans/refactor-auth-module.md` |
| R22 | skills/<name>/SKILL.md | `skills/pdf-tools/SKILL.md` |
| R25 | plugins/installed_plugins.json | `plugins/installed_plugins.json` |
| R26 | mcp-needs-auth-cache.json (with a `claude.ai ...` key) | `mcp-needs-auth-cache.json` |
| R27 | output-styles/*.md, excluding README.md | `output-styles/concise.md` (+ `README.md`, present to prove loaders must exclude it) |
| R28 | settings.json (permissions with `mcp__` entries, enabledPlugins, model, hooks with a command handler, an http handler, and an unknown handler type) | `settings.json` |
| R30 | agents/*.md | `agents/reviewer.md` |

`claude.json` (deliverable 3, not under `claude-home/`) covers R23/R31. `project-repo/.mcp.json` covers R24; `project-repo/.claude/settings.json` covers R29; `project-repo/.claude/commands`, `.claude/skills`, `.claude/output-styles` cover the project-level halves of R20, R22, R27.

## Test-carryover mapping

Every `@Test` in `PoirotTests/Services/*.swift`, plus the three `PoirotTests/Models/*.swift` files that `JSONDecoder` a JSON literal (`SessionFacetsTests`, `SessionTodoTests`, `StatsCacheTests`), mapped to its fixture path. `UpdateCheckerTests.swift` is dropped entirely per `docs/spec/test-carryover.md` (replaced by the Tauri updater plugin, port-plan P-14) and has no fixtures. `inline` means the test only builds Swift values in code (no file, JSON or text literal), so there is nothing to extract.

### `ClaudeConfigLoaderMCPTests.swift`

| Test | Destination | Fixture |
|---|---|---|
| `decodeClaudeConfig_userScopeServers` | `crates/heimdall-core/src/config/read/mcp.rs (M1-26)` | `cases/ClaudeConfigLoaderMCPTests/decodeClaudeConfig_userScopeServers/` |
| `decodeClaudeConfig_localScopeServers` | `crates/heimdall-core/src/config/read/mcp.rs (M1-26)` | `cases/ClaudeConfigLoaderMCPTests/decodeClaudeConfig_localScopeServers/` |
| `decodeMCPProjectConfig` | `crates/heimdall-core/src/config/read/mcp.rs (M1-26)` | `cases/ClaudeConfigLoaderMCPTests/decodeMCPProjectConfig/` |
| `decodeClaudeConfig_emptyObject` | `crates/heimdall-core/src/config/read/mcp.rs (M1-26)` | `cases/ClaudeConfigLoaderMCPTests/decodeClaudeConfig_emptyObject/` |
| `decodeMCPServerDefinition_minimalFields` | `crates/heimdall-core/src/config/read/mcp.rs (M1-26)` | `cases/ClaudeConfigLoaderMCPTests/decodeMCPServerDefinition_minimalFields/` |
| `decodeMCPServerDefinition_allFields` | `crates/heimdall-core/src/config/read/mcp.rs (M1-26)` | `cases/ClaudeConfigLoaderMCPTests/decodeMCPServerDefinition_allFields/` |
| `mcpServerModel_newFields` | `crates/heimdall-core/src/config/read/mcp.rs (M1-26)` | `inline` (pure MCPServer init, no JSON) |
| `mcpServerModel_httpType` | `crates/heimdall-core/src/config/read/mcp.rs (M1-26)` | `inline` (pure MCPServer init, no JSON) |

### `DebugLogLoaderTests.swift`

| Test | Destination | Fixture |
|---|---|---|
| `parse_emptyString_returnsEmpty` | `crates/heimdall-core/src/data/debug_log.rs (M1-17)` | `inline` (empty string parsed directly, no file) |
| `parse_singleDebugLine` | `crates/heimdall-core/src/data/debug_log.rs (M1-17)` | `cases/DebugLogLoaderTests/parse_singleDebugLine/` |
| `parse_errorLevel` | `crates/heimdall-core/src/data/debug_log.rs (M1-17)` | `cases/DebugLogLoaderTests/parse_errorLevel/` |
| `parse_warnLevel` | `crates/heimdall-core/src/data/debug_log.rs (M1-17)` | `cases/DebugLogLoaderTests/parse_warnLevel/` |
| `parse_multipleLines` | `crates/heimdall-core/src/data/debug_log.rs (M1-17)` | `cases/DebugLogLoaderTests/parse_multipleLines/` |
| `parse_skipsEmptyLines` | `crates/heimdall-core/src/data/debug_log.rs (M1-17)` | `cases/DebugLogLoaderTests/parse_skipsEmptyLines/` |
| `parse_skipsInvalidLines` | `crates/heimdall-core/src/data/debug_log.rs (M1-17)` | `cases/DebugLogLoaderTests/parse_skipsInvalidLines/` |
| `parse_messageWithBrackets` | `crates/heimdall-core/src/data/debug_log.rs (M1-17)` | `cases/DebugLogLoaderTests/parse_messageWithBrackets/` |
| `parse_timestamp_isParsedCorrectly` | `crates/heimdall-core/src/data/debug_log.rs (M1-17)` | `cases/DebugLogLoaderTests/parse_timestamp_isParsedCorrectly/` |
| `parse_unknownLevel_defaultsToDebug` | `crates/heimdall-core/src/data/debug_log.rs (M1-17)` | `cases/DebugLogLoaderTests/parse_unknownLevel_defaultsToDebug/` |
| `loadEntries_nonExistentSession_returnsEmpty` | `crates/heimdall-core/src/data/debug_log.rs (M1-17)` | `inline` (nonexistent path) |
| `loadEntries_validFile_returnsEntries` | `crates/heimdall-core/src/data/debug_log.rs (M1-17)` | `cases/DebugLogLoaderTests/loadEntries_validFile_returnsEntries/` |
| `hasLog_noFile_returnsFalse` | `crates/heimdall-core/src/data/debug_log.rs (M1-17)` | `inline` (nonexistent path) |
| `hasLog_fileExists_returnsTrue` | `crates/heimdall-core/src/data/debug_log.rs (M1-17)` | `cases/DebugLogLoaderTests/hasLog_fileExists_returnsTrue/` |
| `allSessionIds_nonExistentPath_returnsEmpty` | `crates/heimdall-core/src/data/debug_log.rs (M1-17)` | `inline` (nonexistent path) |
| `allSessionIds_returnsFilenames` | `crates/heimdall-core/src/data/debug_log.rs (M1-17)` | `cases/DebugLogLoaderTests/allSessionIds_returnsFilenames/` |
| `loadEntries_paginated_firstPage` | `crates/heimdall-core/src/data/debug_log.rs (M1-17)` | `cases/DebugLogLoaderTests/loadEntries_paginated_firstPage/` |
| `loadEntries_paginated_secondPage` | `crates/heimdall-core/src/data/debug_log.rs (M1-17)` | `cases/DebugLogLoaderTests/loadEntries_paginated_secondPage/` |
| `loadEntries_paginated_lastPage_partialResults` | `crates/heimdall-core/src/data/debug_log.rs (M1-17)` | `cases/DebugLogLoaderTests/loadEntries_paginated_lastPage_partialResults/` |
| `loadEntries_paginated_offsetBeyondEnd_returnsEmpty` | `crates/heimdall-core/src/data/debug_log.rs (M1-17)` | `cases/DebugLogLoaderTests/loadEntries_paginated_offsetBeyondEnd_returnsEmpty/` |
| `loadEntries_paginated_nonExistentSession` | `crates/heimdall-core/src/data/debug_log.rs (M1-17)` | `inline` (nonexistent path) |
| `parsePaged_returnsCorrectTotalCount` | `crates/heimdall-core/src/data/debug_log.rs (M1-17)` | `cases/DebugLogLoaderTests/parsePaged_returnsCorrectTotalCount/` |

### `FacetsLoaderTests.swift`

| Test | Destination | Fixture |
|---|---|---|
| `loadFacets_nonExistentPath_returnsNil` | `crates/heimdall-core/src/data/facets.rs (M1-18)` | `inline` (nonexistent path) |
| `loadFacets_existingFile_returnsFacets` | `crates/heimdall-core/src/data/facets.rs (M1-18)` | `cases/FacetsLoaderTests/loadFacets_existingFile_returnsFacets/` |
| `loadFacets_missingFile_returnsNil` | `crates/heimdall-core/src/data/facets.rs (M1-18)` | `cases/FacetsLoaderTests/loadFacets_missingFile_returnsNil/` (loader looks up session id 'nonexistent-session', which is absent) |
| `loadFacets_invalidJSON_returnsNil` | `crates/heimdall-core/src/data/facets.rs (M1-18)` | `cases/FacetsLoaderTests/loadFacets_invalidJSON_returnsNil/` (intentionally invalid JSON; tests that the loader returns nil) |
| `loadAllFacets_nonExistentPath_returnsEmpty` | `crates/heimdall-core/src/data/facets.rs (M1-18)` | `inline` (nonexistent path) |
| `loadAllFacets_emptyDirectory_returnsEmpty` | `crates/heimdall-core/src/data/facets.rs (M1-18)` | `cases/FacetsLoaderTests/loadAllFacets_emptyDirectory_returnsEmpty/` (existing empty directory) |
| `loadAllFacets_multipleFiles_returnsAll` | `crates/heimdall-core/src/data/facets.rs (M1-18)` | `cases/FacetsLoaderTests/loadAllFacets_multipleFiles_returnsAll/` |
| `loadAllFacets_skipsInvalidFiles` | `crates/heimdall-core/src/data/facets.rs (M1-18)` | `cases/FacetsLoaderTests/loadAllFacets_skipsInvalidFiles/` (bad-session.json is intentionally invalid; tests skip-on-error behaviour) |
| `loadAllFacets_skipsNonJSONFiles` | `crates/heimdall-core/src/data/facets.rs (M1-18)` | `cases/FacetsLoaderTests/loadAllFacets_skipsNonJSONFiles/` |
| `loadAllFacets_keyedBySessionId` | `crates/heimdall-core/src/data/facets.rs (M1-18)` | `cases/FacetsLoaderTests/loadAllFacets_keyedBySessionId/` |

### `FileHistoryLoaderTests.swift`

| Test | Destination | Fixture |
|---|---|---|
| `load_includesDeltaVersionsMissingFromSnapshots` | `crates/heimdall-core/src/data/file_history.rs (M1-19)` | `cases/FileHistoryLoaderTests/load_includesDeltaVersionsMissingFromSnapshots/` (sessionId b01a22e5-cfc1-5e0e-8c76-0cc2a97e14e9 (deterministic substitute for the Swift test's random UUID); file-history/ root has no backup blobs for this case) |
| `load_deltaOnlySession_stillReturnsFiles` | `crates/heimdall-core/src/data/file_history.rs (M1-19)` | `cases/FileHistoryLoaderTests/load_deltaOnlySession_stillReturnsFiles/` (sessionId fb8f8959-6681-5e08-ba4a-7547018d919b (deterministic substitute)) |
| `sameFile_treatsRelativeAsSuffixOfAbsolute` | `crates/heimdall-core/src/data/file_history.rs (M1-19)` | `inline` (pure static method, no files) |
| `load_mergesRelativeAndAbsolutePathsForSameFile` | `crates/heimdall-core/src/data/file_history.rs (M1-19)` | `cases/FileHistoryLoaderTests/load_mergesRelativeAndAbsolutePathsForSameFile/` (sessionId 7be6bd4e-5e18-5c4c-8287-81c3e0e10217 (deterministic substitute)) |
| `loadFileContent_readsBackup` | `crates/heimdall-core/src/data/file_history.rs (M1-19)` | `cases/FileHistoryLoaderTests/loadFileContent_readsBackup/` (sessionId 4333913a-24ab-5adb-859c-b5c0575ef51b (deterministic substitute)) |

### `FrontmatterParserTests.swift`

| Test | Destination | Fixture |
|---|---|---|
| `parse_noFrontmatter_returnsOriginalBody` | `crates/heimdall-core/src/config/read/frontmatter.rs (M1-23)` | `cases/FrontmatterParserTests/parse_noFrontmatter_returnsOriginalBody/` |
| `parse_unclosedFrontmatter_returnsOriginalBody` | `crates/heimdall-core/src/config/read/frontmatter.rs (M1-23)` | `cases/FrontmatterParserTests/parse_unclosedFrontmatter_returnsOriginalBody/` |
| `parse_wellFormed_extractsMetadata` | `crates/heimdall-core/src/config/read/frontmatter.rs (M1-23)` | `cases/FrontmatterParserTests/parse_wellFormed_extractsMetadata/` |
| `parse_colonInValue_preservesFullValue` | `crates/heimdall-core/src/config/read/frontmatter.rs (M1-23)` | `cases/FrontmatterParserTests/parse_colonInValue_preservesFullValue/` |
| `parse_emptyLinesInYAML_skipped` | `crates/heimdall-core/src/config/read/frontmatter.rs (M1-23)` | `cases/FrontmatterParserTests/parse_emptyLinesInYAML_skipped/` |
| `parse_bodyTrimmed_whitespaceRemoved` | `crates/heimdall-core/src/config/read/frontmatter.rs (M1-23)` | `cases/FrontmatterParserTests/parse_bodyTrimmed_whitespaceRemoved/` |
| `parse_multipleKeys_allExtracted` | `crates/heimdall-core/src/config/read/frontmatter.rs (M1-23)` | `cases/FrontmatterParserTests/parse_multipleKeys_allExtracted/` |
| `parse_emptyBody_afterFrontmatter` | `crates/heimdall-core/src/config/read/frontmatter.rs (M1-23)` | `cases/FrontmatterParserTests/parse_emptyBody_afterFrontmatter/` |
| `parse_foldedBlockScalar_joinsWithSpaces` | `crates/heimdall-core/src/config/read/frontmatter.rs (M1-23)` | `cases/FrontmatterParserTests/parse_foldedBlockScalar_joinsWithSpaces/` |
| `parse_literalBlockScalar_joinsWithNewlines` | `crates/heimdall-core/src/config/read/frontmatter.rs (M1-23)` | `cases/FrontmatterParserTests/parse_literalBlockScalar_joinsWithNewlines/` |
| `parse_blockScalarFollowedByKey_bothExtracted` | `crates/heimdall-core/src/config/read/frontmatter.rs (M1-23)` | `cases/FrontmatterParserTests/parse_blockScalarFollowedByKey_bothExtracted/` |
| `parse_blockScalarAtEnd_extracted` | `crates/heimdall-core/src/config/read/frontmatter.rs (M1-23)` | `cases/FrontmatterParserTests/parse_blockScalarAtEnd_extracted/` |

### `HistoryLoaderTests.swift`

| Test | Destination | Fixture |
|---|---|---|
| `loadAll_nonExistentFile_returnsEmpty` | `crates/heimdall-core/src/data/history.rs (M1-15)` | `inline` (nonexistent path) |
| `loadAll_emptyFile_returnsEmpty` | `crates/heimdall-core/src/data/history.rs (M1-15)` | `cases/HistoryLoaderTests/loadAll_emptyFile_returnsEmpty/` |
| `loadAll_singleEntry_parsesCorrectly` | `crates/heimdall-core/src/data/history.rs (M1-15)` | `cases/HistoryLoaderTests/loadAll_singleEntry_parsesCorrectly/` |
| `loadAll_multipleEntries_sortedByTimestampDescending` | `crates/heimdall-core/src/data/history.rs (M1-15)` | `cases/HistoryLoaderTests/loadAll_multipleEntries_sortedByTimestampDescending/` |
| `loadAll_skipsInvalidLines` | `crates/heimdall-core/src/data/history.rs (M1-15)` | `cases/HistoryLoaderTests/loadAll_skipsInvalidLines/` (middle line is intentionally invalid JSON; tests that the loader skips it) |
| `loadAll_handlesPastedContents` | `crates/heimdall-core/src/data/history.rs (M1-15)` | `cases/HistoryLoaderTests/loadAll_handlesPastedContents/` |
| `loadAll_handlesNullPastedContents` | `crates/heimdall-core/src/data/history.rs (M1-15)` | `cases/HistoryLoaderTests/loadAll_handlesNullPastedContents/` |
| `loadAll_timestampConvertedFromMilliseconds` | `crates/heimdall-core/src/data/history.rs (M1-15)` | `cases/HistoryLoaderTests/loadAll_timestampConvertedFromMilliseconds/` |
| `entryCount_nonExistentFile_returnsZero` | `crates/heimdall-core/src/data/history.rs (M1-15)` | `inline` (nonexistent path) |
| `entryCount_matchesLineCount` | `crates/heimdall-core/src/data/history.rs (M1-15)` | `cases/HistoryLoaderTests/entryCount_matchesLineCount/` |
| `uniqueProjects_returnsDistinctPaths` | `crates/heimdall-core/src/data/history.rs (M1-15)` | `cases/HistoryLoaderTests/uniqueProjects_returnsDistinctPaths/` |
| `uniqueProjects_nonExistentFile_returnsEmpty` | `crates/heimdall-core/src/data/history.rs (M1-15)` | `inline` (nonexistent path) |

### `MCPServerStatusCheckerTests.swift`

| Test | Destination | Fixture |
|---|---|---|
| `authCacheDetection` | `crates/heimdall-core/src/mcp_status.rs (M1-28)` | `inline` (pure MCPServer struct construction, no JSON) |
| `serverNotInAuthCache` | `crates/heimdall-core/src/mcp_status.rs (M1-28)` | `inline` (pure MCPServer struct construction, no JSON) |
| `httpServerConnected` | `crates/heimdall-core/src/mcp_status.rs (M1-28)` | `inline` (pure MCPServer struct construction, no JSON) |
| `httpServerNoURL` | `crates/heimdall-core/src/mcp_status.rs (M1-28)` | `inline` (pure MCPServer struct construction, no JSON) |
| `sseServerConnected` | `crates/heimdall-core/src/mcp_status.rs (M1-28)` | `inline` (pure MCPServer struct construction, no JSON) |
| `urlInferredHTTP` | `crates/heimdall-core/src/mcp_status.rs (M1-28)` | `inline` (pure MCPServer struct construction, no JSON) |
| `stdioNoCommand` | `crates/heimdall-core/src/mcp_status.rs (M1-28)` | `inline` (pure MCPServer struct construction, no JSON) |
| `noTransportInfo` | `crates/heimdall-core/src/mcp_status.rs (M1-28)` | `inline` (pure MCPServer struct construction, no JSON) |
| `bulkResolution` | `crates/heimdall-core/src/mcp_status.rs (M1-28)` | `inline` (pure MCPServer struct construction, no JSON) |
| `authCachePrecedence` | `crates/heimdall-core/src/mcp_status.rs (M1-28)` | `inline` (pure MCPServer struct construction, no JSON) |
| `authCachePrecedenceStdio` | `crates/heimdall-core/src/mcp_status.rs (M1-28)` | `inline` (pure MCPServer struct construction, no JSON) |
| `cloudIntegrationPreservesStatus` | `crates/heimdall-core/src/mcp_status.rs (M1-28)` | `inline` (pure MCPServer struct construction, no JSON) |
| `cloudIntegrationIgnoresAuthCache` | `crates/heimdall-core/src/mcp_status.rs (M1-28)` | `inline` (pure MCPServer struct construction, no JSON) |
| `pluginServerNoProcess` | `crates/heimdall-core/src/mcp_status.rs (M1-28)` | `inline` (pure MCPServer struct construction, no JSON) |

### `MemoryLoaderTests.swift`

| Test | Destination | Fixture |
|---|---|---|
| `loadMemoryFiles_returnsEmptyForMissingDirectory` | `crates/heimdall-core/src/config/read/memory.rs (M1-27)` | `inline` (nonexistent path) |
| `loadMemoryFiles_loadsMarkdownFiles` | `crates/heimdall-core/src/config/read/memory.rs (M1-27)` | `cases/MemoryLoaderTests/loadMemoryFiles_loadsMarkdownFiles/` (root is a project dir; memory/ is its memory subfolder) |
| `loadMemoryFiles_memoryMdSortedFirst` | `crates/heimdall-core/src/config/read/memory.rs (M1-27)` | `cases/MemoryLoaderTests/loadMemoryFiles_memoryMdSortedFirst/` |
| `loadMemoryFiles_setsProjectID` | `crates/heimdall-core/src/config/read/memory.rs (M1-27)` | `cases/MemoryLoaderTests/loadMemoryFiles_setsProjectID/` |
| `loadMemoryFiles_customFolder_loadsMarkdownWithSourceLabel` | `crates/heimdall-core/src/config/read/memory.rs (M1-27)` | `cases/MemoryLoaderTests/loadMemoryFiles_customFolder_loadsMarkdownWithSourceLabel/` (root is the custom folder directly (no memory/ subpath)) |
| `customMemoryFileCount_countsOnlyMarkdown` | `crates/heimdall-core/src/config/read/memory.rs (M1-27)` | `cases/MemoryLoaderTests/customMemoryFileCount_countsOnlyMarkdown/` (root is the custom folder directly) |
| `loadMemoryFiles_customFolder_missing_returnsEmpty` | `crates/heimdall-core/src/config/read/memory.rs (M1-27)` | `inline` (nonexistent path) |

### `SessionLoaderTests.swift`

| Test | Destination | Fixture |
|---|---|---|
| `discoverProjects_nonExistentPath_returnsEmpty` | `crates/heimdall-core/src/projects/discovery.rs (M1-14)` | `inline` (nonexistent path) |
| `discoverProjects_emptyDirectory_returnsEmpty` | `crates/heimdall-core/src/projects/discovery.rs (M1-14)` | `cases/SessionLoaderTests/discoverProjects_emptyDirectory_returnsEmpty/` (existing empty directory) |
| `discoverProjects_withSubdirectories_returnsProjects` | `crates/heimdall-core/src/projects/discovery.rs (M1-14)` | `cases/SessionLoaderTests/discoverProjects_withSubdirectories_returnsProjects/` (two empty project directories: project-alpha/, project-beta/) |
| `discoverProjects_skipsFiles_onlyIncludesDirectories` | `crates/heimdall-core/src/projects/discovery.rs (M1-14)` | `cases/SessionLoaderTests/discoverProjects_skipsFiles_onlyIncludesDirectories/` |
| `discoverProjects_withJSONLFiles_parsesSessions` | `crates/heimdall-core/src/projects/discovery.rs (M1-14)` | `cases/SessionLoaderTests/discoverProjects_withJSONLFiles_parsesSessions/` |
| `discoverProjects_fallbackPath_loadsAgentFiles` | `crates/heimdall-core/src/projects/subagents.rs (M1-14)` | `cases/SessionLoaderTests/discoverProjects_fallbackPath_loadsAgentFiles/` |
| `discoverProjects_sessionsSortedDescending` | `crates/heimdall-core/src/projects/discovery.rs (M1-14)` | `cases/SessionLoaderTests/discoverProjects_sessionsSortedDescending/` |
| `discoverProjects_usesOriginalPathFromIndex` | `crates/heimdall-core/src/projects/sessions_index.rs (M1-14)` | `cases/SessionLoaderTests/discoverProjects_usesOriginalPathFromIndex/` |
| `discoverProjects_withMissingIndex_usesFallback` | `crates/heimdall-core/src/projects/sessions_index.rs (M1-14)` | `cases/SessionLoaderTests/discoverProjects_withMissingIndex_usesFallback/` |
| `discoverProjects_emptyJSONLFile_producesNoSession` | `crates/heimdall-core/src/projects/discovery.rs (M1-14)` | `cases/SessionLoaderTests/discoverProjects_emptyJSONLFile_producesNoSession/` |
| `discoverProjects_indexPath_includesSidechainEntries` | `crates/heimdall-core/src/projects/sessions_index.rs (M1-14)` | `cases/SessionLoaderTests/discoverProjects_indexPath_includesSidechainEntries/` |
| `discoverProjects_indexPath_loadsAllSessions` | `crates/heimdall-core/src/projects/sessions_index.rs (M1-14)` | `cases/SessionLoaderTests/discoverProjects_indexPath_loadsAllSessions/` |
| `discoverProjects_fallbackPath_usesHeaderParse` | `crates/heimdall-core/src/projects/discovery.rs (M1-14)` | `cases/SessionLoaderTests/discoverProjects_fallbackPath_usesHeaderParse/` |
| `discoverProjects_indexPath_passesFirstPromptToSession` | `crates/heimdall-core/src/projects/sessions_index.rs (M1-14)` | `cases/SessionLoaderTests/discoverProjects_indexPath_passesFirstPromptToSession/` |
| `discoverProjects_indexPath_nilFirstPromptWhenMissing` | `crates/heimdall-core/src/projects/sessions_index.rs (M1-14)` | `cases/SessionLoaderTests/discoverProjects_indexPath_nilFirstPromptWhenMissing/` |
| `discoverProjects_fallbackPath_firstPromptIsNil` | `crates/heimdall-core/src/projects/discovery.rs (M1-14)` | `cases/SessionLoaderTests/discoverProjects_fallbackPath_firstPromptIsNil/` |

### `SettingsWriterMCPTests.swift`

| Test | Destination | Fixture |
|---|---|---|
| `catalog_entriesAreValid` | `crates/heimdall-core/src/models/catalog/mcp_catalog.rs (M1-10)` | `inline` (pure catalog/struct assertions, no JSON) |
| `catalog_searchByCategory` | `crates/heimdall-core/src/models/catalog/mcp_catalog.rs (M1-10)` | `inline` (pure catalog/struct assertions, no JSON) |
| `envKey_defaultValues` | `crates/heimdall-core/src/models/catalog/mcp_catalog.rs (M1-10)` | `inline` (pure catalog/struct assertions, no JSON) |
| `envKey_customValues` | `crates/heimdall-core/src/models/catalog/mcp_catalog.rs (M1-10)` | `inline` (pure catalog/struct assertions, no JSON) |

### `StatsComputerTests.swift`

| Test | Destination | Fixture |
|---|---|---|
| `compute_aggregatesSessionFromTranscript` | `crates/heimdall-core/src/data/stats.rs (M1-20)` | `cases/StatsComputerTests/compute_aggregatesSessionFromTranscript/` (sessionId efa6a4eb-d601-5f88-b045-1d9ef9481b50 (deterministic substitute)) |
| `compute_noSessions_returnsNil` | `crates/heimdall-core/src/data/stats.rs (M1-20)` | `cases/StatsComputerTests/compute_noSessions_returnsNil/` (existing empty projects root, no project subdir) |
| `compute_missingPath_returnsNil` | `crates/heimdall-core/src/data/stats.rs (M1-20)` | `inline` (nonexistent path) |
| `compute_includesTokensOnLaterDays` | `crates/heimdall-core/src/data/stats.rs (M1-20)` | `cases/StatsComputerTests/compute_includesTokensOnLaterDays/` (sessionId 4bb68591-f56e-57ca-a112-a9040ae533e6 (deterministic substitute)) |
| `compute_acceptsFloatingPointTokenCounts` | `crates/heimdall-core/src/data/stats.rs (M1-20)` | `cases/StatsComputerTests/compute_acceptsFloatingPointTokenCounts/` (sessionId 3cd6c2da-237c-59d5-89e7-587f34e80257 (deterministic substitute)) |
| `intValue_readsNSNumberAndIgnoresBool` | `crates/heimdall-core/src/data/stats.rs (M1-20)` | `inline` (pure function, no files) |

### `TodoLoaderTests.swift`

| Test | Destination | Fixture |
|---|---|---|
| `loadTodos_nonExistentPath_returnsEmpty` | `crates/heimdall-core/src/data/todos.rs (M1-16)` | `inline` (nonexistent path) |
| `loadTodos_emptyDirectory_returnsEmpty` | `crates/heimdall-core/src/data/todos.rs (M1-16)` | `cases/TodoLoaderTests/loadTodos_emptyDirectory_returnsEmpty/` (existing empty directory) |
| `loadTodos_matchesAgentFile` | `crates/heimdall-core/src/data/todos.rs (M1-16)` | `cases/TodoLoaderTests/loadTodos_matchesAgentFile/` |
| `loadTodos_matchesExactSessionIdFile` | `crates/heimdall-core/src/data/todos.rs (M1-16)` | `cases/TodoLoaderTests/loadTodos_matchesExactSessionIdFile/` |
| `loadTodos_mergesMultipleAgentFiles` | `crates/heimdall-core/src/data/todos.rs (M1-16)` | `cases/TodoLoaderTests/loadTodos_mergesMultipleAgentFiles/` |
| `loadTodos_doesNotMatchUnrelatedFiles` | `crates/heimdall-core/src/data/todos.rs (M1-16)` | `cases/TodoLoaderTests/loadTodos_doesNotMatchUnrelatedFiles/` |
| `loadTodos_handlesEmptyArrayFile` | `crates/heimdall-core/src/data/todos.rs (M1-16)` | `cases/TodoLoaderTests/loadTodos_handlesEmptyArrayFile/` |
| `loadTodos_skipsInvalidJSON` | `crates/heimdall-core/src/data/todos.rs (M1-16)` | `cases/TodoLoaderTests/loadTodos_skipsInvalidJSON/` (abc-1234-agent-bad.json is intentionally invalid; tests skip-on-error behaviour) |
| `loadTodos_uuidSessionId_matchesAgentFile` | `crates/heimdall-core/src/data/todos.rs (M1-16)` | `cases/TodoLoaderTests/loadTodos_uuidSessionId_matchesAgentFile/` |
| `loadAllTodos_nonExistentPath_returnsEmpty` | `crates/heimdall-core/src/data/todos.rs (M1-16)` | `inline` (nonexistent path) |
| `loadAllTodos_groupsBySessionId` | `crates/heimdall-core/src/data/todos.rs (M1-16)` | `cases/TodoLoaderTests/loadAllTodos_groupsBySessionId/` |
| `loadAllTodos_mergesAgentFilesUnderSameSession` | `crates/heimdall-core/src/data/todos.rs (M1-16)` | `cases/TodoLoaderTests/loadAllTodos_mergesAgentFilesUnderSameSession/` |
| `loadAllTodos_skipsEmptyArrayFiles` | `crates/heimdall-core/src/data/todos.rs (M1-16)` | `cases/TodoLoaderTests/loadAllTodos_skipsEmptyArrayFiles/` |
| `deleteTodos_removesMatchingFiles` | `crates/heimdall-core/src/projects/delete.rs (M3-43)` | `cases/TodoLoaderTests/deleteTodos_removesMatchingFiles/` |
| `deleteTodos_leavesUnrelatedFiles` | `crates/heimdall-core/src/projects/delete.rs (M3-43)` | `cases/TodoLoaderTests/deleteTodos_leavesUnrelatedFiles/` |
| `deleteTodos_nonExistentPath_doesNotCrash` | `crates/heimdall-core/src/projects/delete.rs (M3-43)` | `inline` (nonexistent path) |

### `TranscriptParserTests.swift`

| Test | Destination | Fixture |
|---|---|---|
| `parse_emptyFile_returnsNil` | `crates/heimdall-core/src/transcript/full.rs (M1-12)` | `cases/TranscriptParserTests/parse_emptyFile_returnsNil/` |
| `parse_onlySkippedRecordTypes_returnsNil` | `crates/heimdall-core/src/transcript/full.rs (M1-12)` | `cases/TranscriptParserTests/parse_onlySkippedRecordTypes_returnsNil/` |
| `parse_malformedLines_skipsGracefully` | `crates/heimdall-core/src/transcript/full.rs (M1-12)` | `cases/TranscriptParserTests/parse_malformedLines_skipsGracefully/` (first two lines are intentionally malformed; tests that the parser skips them) |
| `parse_sidechainRecords_skipped` | `crates/heimdall-core/src/transcript/full.rs (M1-12)` | `cases/TranscriptParserTests/parse_sidechainRecords_skipped/` |
| `parse_userStringContent_createsTextBlock` | `crates/heimdall-core/src/transcript/full.rs (M1-12)` | `cases/TranscriptParserTests/parse_userStringContent_createsTextBlock/` |
| `parse_userToolResult_createsToolResultBlock` | `crates/heimdall-core/src/transcript/full.rs (M1-12)` | `cases/TranscriptParserTests/parse_userToolResult_createsToolResultBlock/` |
| `parse_toolResult_stringContent_normalized` | `crates/heimdall-core/src/transcript/full.rs (M1-12)` | `cases/TranscriptParserTests/parse_toolResult_stringContent_normalized/` |
| `parse_toolResult_arrayContent_extractsTextOnly` | `crates/heimdall-core/src/transcript/full.rs (M1-12)` | `cases/TranscriptParserTests/parse_toolResult_arrayContent_extractsTextOnly/` |
| `parse_toolResult_isError_flagged` | `crates/heimdall-core/src/transcript/full.rs (M1-12)` | `cases/TranscriptParserTests/parse_toolResult_isError_flagged/` |
| `parse_assistantTextBlock` | `crates/heimdall-core/src/transcript/full.rs (M1-12)` | `cases/TranscriptParserTests/parse_assistantTextBlock/` |
| `parse_assistantThinkingBlock` | `crates/heimdall-core/src/transcript/full.rs (M1-12)` | `cases/TranscriptParserTests/parse_assistantThinkingBlock/` |
| `parse_assistantToolUseBlock` | `crates/heimdall-core/src/transcript/full.rs (M1-12)` | `cases/TranscriptParserTests/parse_assistantToolUseBlock/` |
| `parse_toolUse_nonStringInputsStringified` | `crates/heimdall-core/src/transcript/full.rs (M1-12)` | `cases/TranscriptParserTests/parse_toolUse_nonStringInputsStringified/` (Edit-style tool_use with non-string input values, stringified by the parser) |
| `parse_syntheticModel_skipped` | `crates/heimdall-core/src/transcript/full.rs (M1-12)` | `cases/TranscriptParserTests/parse_syntheticModel_skipped/` (assistant record uses the <synthetic> model sentinel) |
| `parse_multipleRecordsSameId_grouped` | `crates/heimdall-core/src/transcript/full.rs (M1-12)` | `cases/TranscriptParserTests/parse_multipleRecordsSameId_grouped/` |
| `parse_usage_deduplicatedPerGroup` | `crates/heimdall-core/src/transcript/full.rs (M1-12)` | `cases/TranscriptParserTests/parse_usage_deduplicatedPerGroup/` |
| `parse_startedAt_usesEarliestTimestamp` | `crates/heimdall-core/src/transcript/full.rs (M1-12)` | `cases/TranscriptParserTests/parse_startedAt_usesEarliestTimestamp/` |
| `parse_model_fromFirstAssistantGroup` | `crates/heimdall-core/src/transcript/full.rs (M1-12)` | `cases/TranscriptParserTests/parse_model_fromFirstAssistantGroup/` |
| `parse_totalTokens_summedAcrossGroups` | `crates/heimdall-core/src/transcript/full.rs (M1-12)` | `cases/TranscriptParserTests/parse_totalTokens_summedAcrossGroups/` |
| `parse_interleaved_userAndAssistant_preservesOrder` | `crates/heimdall-core/src/transcript/full.rs (M1-12)` | `cases/TranscriptParserTests/parse_interleaved_userAndAssistant_preservesOrder/` |
| `parseSummary_emptyFile_returnsNil` | `crates/heimdall-core/src/transcript/summary.rs (M1-13)` | `cases/TranscriptParserTests/parseSummary_emptyFile_returnsNil/` |
| `parseSummary_extractsTitle` | `crates/heimdall-core/src/transcript/summary.rs (M1-13)` | `cases/TranscriptParserTests/parseSummary_extractsTitle/` |
| `parseSummary_extractsTurnCount` | `crates/heimdall-core/src/transcript/summary.rs (M1-13)` | `cases/TranscriptParserTests/parseSummary_extractsTurnCount/` |
| `parseSummary_extractsModel` | `crates/heimdall-core/src/transcript/summary.rs (M1-13)` | `cases/TranscriptParserTests/parseSummary_extractsModel/` |
| `parseSummary_sumsTokensPerGroup` | `crates/heimdall-core/src/transcript/summary.rs (M1-13)` | `cases/TranscriptParserTests/parseSummary_sumsTokensPerGroup/` (msg_1 appears twice (duplicate messageId); tokens must not double-count) |
| `parseSummary_skipsSidechainRecords` | `crates/heimdall-core/src/transcript/summary.rs (M1-13)` | `cases/TranscriptParserTests/parseSummary_skipsSidechainRecords/` |
| `parseSummary_skipsSyntheticAssistant` | `crates/heimdall-core/src/transcript/summary.rs (M1-13)` | `cases/TranscriptParserTests/parseSummary_skipsSyntheticAssistant/` |
| `parseSummary_usesEarliestTimestamp` | `crates/heimdall-core/src/transcript/summary.rs (M1-13)` | `cases/TranscriptParserTests/parseSummary_usesEarliestTimestamp/` |
| `parseSummary_searchableText_indexesConversationContent` | `crates/heimdall-core/src/transcript/summary.rs (M1-13)` | `cases/TranscriptParserTests/parseSummary_searchableText_indexesConversationContent/` |
| `parseSummary_searchableText_excludesSyntheticAndSidechain` | `crates/heimdall-core/src/transcript/summary.rs (M1-13)` | `cases/TranscriptParserTests/parseSummary_searchableText_excludesSyntheticAndSidechain/` |
| `parseSummary_searchableText_respectsCap` | `crates/heimdall-core/src/transcript/summary.rs (M1-13)` | `cases/TranscriptParserTests/parseSummary_searchableText_respectsCap/` (content is 25000 repeated 'a' characters (searchableTextCap=20000 + 5000 overflow)) |
| `parse_searchableText_indexesContent` | `crates/heimdall-core/src/transcript/searchable.rs (M1-12)` | `cases/TranscriptParserTests/parse_searchableText_indexesContent/` |

### `SessionFacetsTests.swift`

| Test | Destination | Fixture |
|---|---|---|
| `decodesFullJSON` | `crates/heimdall-core/src/models/session_facets.rs (M1-09)` | `cases/SessionFacetsTests/decodesFullJSON/` |
| `decodesEmptyDictionaries` | `crates/heimdall-core/src/models/session_facets.rs (M1-09)` | `cases/SessionFacetsTests/decodesEmptyDictionaries/` |
| `id_matchesSessionId` | `crates/heimdall-core/src/models/session_facets.rs (M1-09)` | `cases/SessionFacetsTests/id_matchesSessionId/` |
| `sortedGoalCategories_orderedByCountDescending` | `crates/heimdall-core/src/models/session_facets.rs (M1-09)` | `cases/SessionFacetsTests/sortedGoalCategories_orderedByCountDescending/` (session_id is a deterministic substitute for the Swift test's random UUID) |
| `totalFrictionCount_sumsValues` | `crates/heimdall-core/src/models/session_facets.rs (M1-09)` | `cases/SessionFacetsTests/totalFrictionCount_sumsValues/` (session_id is a deterministic substitute) |
| `sortedFrictionItems_filtersZeroValues` | `crates/heimdall-core/src/models/session_facets.rs (M1-09)` | `cases/SessionFacetsTests/sortedFrictionItems_filtersZeroValues/` (session_id is a deterministic substitute) |
| `outcomeLabel_mapsKnownValues` | `crates/heimdall-core/src/models/session_facets.rs (M1-09)` | `cases/SessionFacetsTests/outcomeLabel_mapsKnownValues/` |
| `helpfulnessLabel_mapsKnownValues` | `crates/heimdall-core/src/models/session_facets.rs (M1-09)` | `cases/SessionFacetsTests/helpfulnessLabel_mapsKnownValues/` |
| `sessionTypeLabel_formatsUnderscores` | `crates/heimdall-core/src/models/session_facets.rs (M1-09)` | `cases/SessionFacetsTests/sessionTypeLabel_formatsUnderscores/` |
| `categoryLabel_formatsUnderscores` | `crates/heimdall-core/src/models/session_facets.rs (M1-09)` | `inline` (pure static string-formatting function) |
| `frictionLabel_formatsUnderscores` | `crates/heimdall-core/src/models/session_facets.rs (M1-09)` | `inline` (pure static string-formatting function) |

### `SessionTodoTests.swift`

| Test | Destination | Fixture |
|---|---|---|
| `decodesCompletedStatus` | `crates/heimdall-core/src/models/session_todo.rs (M1-09)` | `cases/SessionTodoTests/decodesCompletedStatus/` |
| `decodesInProgressStatus` | `crates/heimdall-core/src/models/session_todo.rs (M1-09)` | `cases/SessionTodoTests/decodesInProgressStatus/` |
| `decodesPendingStatus` | `crates/heimdall-core/src/models/session_todo.rs (M1-09)` | `cases/SessionTodoTests/decodesPendingStatus/` |
| `decodesArrayOfTodos` | `crates/heimdall-core/src/models/session_todo.rs (M1-09)` | `cases/SessionTodoTests/decodesArrayOfTodos/` |
| `identifiable_idDerivedFromContent` | `crates/heimdall-core/src/models/session_todo.rs (M1-09)` | `cases/SessionTodoTests/identifiable_idDerivedFromContent/` |
| `hashable_todosWithSameContentAreEqual` | `crates/heimdall-core/src/models/session_todo.rs (M1-09)` | `cases/SessionTodoTests/hashable_todosWithSameContentAreEqual/` |
| `decodesEmptyArray` | `crates/heimdall-core/src/models/session_todo.rs (M1-09)` | `cases/SessionTodoTests/decodesEmptyArray/` |

### `StatsCacheTests.swift`

| Test | Destination | Fixture |
|---|---|---|
| `decode_fullJSON_succeeds` | `crates/heimdall-core/src/models/stats_cache.rs (M1-09)` | `cases/StatsCacheTests/decode_fullJSON_succeeds/` (shared literal (Self.validJSON in the Swift suite)) |
| `decode_dailyActivity_parsesCorrectly` | `crates/heimdall-core/src/models/stats_cache.rs (M1-09)` | `cases/StatsCacheTests/decode_dailyActivity_parsesCorrectly/` (shared literal (Self.validJSON in the Swift suite)) |
| `decode_dailyModelTokens_parsesMultiModel` | `crates/heimdall-core/src/models/stats_cache.rs (M1-09)` | `cases/StatsCacheTests/decode_dailyModelTokens_parsesMultiModel/` (shared literal (Self.validJSON in the Swift suite)) |
| `decode_modelUsage_parsesTokenCounts` | `crates/heimdall-core/src/models/stats_cache.rs (M1-09)` | `cases/StatsCacheTests/decode_modelUsage_parsesTokenCounts/` (shared literal (Self.validJSON in the Swift suite)) |
| `decode_longestSession_parsesAllFields` | `crates/heimdall-core/src/models/stats_cache.rs (M1-09)` | `cases/StatsCacheTests/decode_longestSession_parsesAllFields/` (shared literal (Self.validJSON in the Swift suite)) |
| `decode_hourCounts_parsesStringKeys` | `crates/heimdall-core/src/models/stats_cache.rs (M1-09)` | `cases/StatsCacheTests/decode_hourCounts_parsesStringKeys/` (shared literal (Self.validJSON in the Swift suite)) |
| `longestSessionFormatted_showsHoursAndMinutes` | `crates/heimdall-core/src/models/stats_cache.rs (M1-09)` | `cases/StatsCacheTests/longestSessionFormatted_showsHoursAndMinutes/` (shared literal (Self.validJSON in the Swift suite)) |
| `sortedHourCounts_sortsAscending` | `crates/heimdall-core/src/models/stats_cache.rs (M1-09)` | `cases/StatsCacheTests/sortedHourCounts_sortsAscending/` (shared literal (Self.validJSON in the Swift suite)) |
| `totalOutputTokens_sumsAcrossModels` | `crates/heimdall-core/src/models/stats_cache.rs (M1-09)` | `cases/StatsCacheTests/totalOutputTokens_sumsAcrossModels/` (shared literal (Self.validJSON in the Swift suite)) |
| `firstSessionParsedDate_parsesISO8601` | `crates/heimdall-core/src/models/stats_cache.rs (M1-09)` | `cases/StatsCacheTests/firstSessionParsedDate_parsesISO8601/` (shared literal (Self.validJSON in the Swift suite)) |
| `totalToolCalls_sumsAcrossDays` | `crates/heimdall-core/src/models/stats_cache.rs (M1-09)` | `cases/StatsCacheTests/totalToolCalls_sumsAcrossDays/` (shared literal (Self.validJSON in the Swift suite)) |
| `friendlyModelName_mapsKnownModels` | `crates/heimdall-core/src/models/stats_cache.rs (M1-09)` | `inline` (pure static string-mapping function) |
| `loader_missingFile_returnsNil` | `crates/heimdall-core/src/data/stats_cache.rs (M1-20)` | `inline` (nonexistent path) |
| `isStale_tokensLagActivity_isTrue` | `crates/heimdall-core/src/data/stats_cache.rs (M1-20)` | `inline` (StatsCache built via memberwise init, no JSON) |
| `isStale_lastComputedBeforeLatestSession_isTrue` | `crates/heimdall-core/src/data/stats_cache.rs (M1-20)` | `inline` (StatsCache built via memberwise init, no JSON) |
| `loadAnalytics_staleCache_recomputesFromTranscripts` | `crates/heimdall-core/src/data/stats_cache.rs (M1-20)` | `cases/StatsCacheTests/loadAnalytics_staleCache_recomputesFromTranscripts/` (sessionId 3218cc5d-e1cb-5a4a-8358-4df2e7fca63a (deterministic substitute); stats-cache.json is the pre-stale cache as JSONEncoder would write it) |
| `loadAnalytics_freshCache_isKept` | `crates/heimdall-core/src/data/stats_cache.rs (M1-20)` | `cases/StatsCacheTests/loadAnalytics_freshCache_isKept/` (sessionId e884e007-2f9d-5e36-bded-9ad4c439cb84 (deterministic substitute); lastComputedDate matches the transcript's latest session day (2026-08-20), so the cache is fresh and kept as-is) |

## Acceptance checks

Run from the Heimdall repository root:

```bash
python fixtures/synthetic/check_fixtures.py
```

This verifies: every `.json` outside `corrupt/` parses; every `.jsonl` line outside `corrupt/` parses (both excluding the intentionally-invalid paths listed above); and no absolute home path other than `/Users/fixture` appears anywhere under `fixtures/synthetic/`.

