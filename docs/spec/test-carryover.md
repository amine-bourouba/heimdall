# Test carry-over map

Task M0-06. Every Swift test in `$SWIFT_REF/PoirotTests` (commit `a29dd6e`) with its Heimdall destination. Rust destinations are the module file that holds the test (`#[cfg(test)] mod tests` in that file). A task that ports a file must port every row whose destination is one of its outputs.

**Count.** `grep -rho "@Test" $SWIFT_REF/PoirotTests | wc -l` = 637. This map has 637 rows, one per `@Test`.

## Decisions encoded here

- **Analytics.** View-model tests go to Vitest (`src/features/analytics/store.spec.ts`), as the brief asks. `AnalyticsCSVExporterTests` goes to Rust because the CSV is generated where the file is written (port-plan P-18).
- **History date buckets and relative times** go to Vitest (`src/features/history/store.spec.ts`): they depend on the local clock and calendar, which is view logic (P-1).
- **Project sorting** (`ProjectSortingTests`, the sort cases of `AppStateTests`) goes to the sessions store, where sorting happens.
- **`SettingsWriterMCPTests`** tests the MCP catalog, not the settings writer, so it goes to `models/catalog/mcp_catalog.rs`.
- **`HighlightedTextTests`** contains only matching assertions (no AttributedString assertions exist at `a29dd6e`), so all four go to `search/matcher.rs`.
- **`UsageWindowTests.timeUntilReset`** is dropped: reset countdowns belonged to the removed Usage Limits feature. The other two cover the gauge ring.
- **Palette tests in `ColorThemeTests`** go with the token file (`src/theme/contrast.spec.ts`); theme-list and persistence tests go to the theme store.
- **Dropped files:** `ClaudeCodeProviderTests` (ADR 0008) and `UpdateCheckerTests` (P-14). Mocks: `*LoadingMock.swift` and `ProviderDescribingMock.swift` are replaced by IPC mocks (`src/ipc/testing.ts`); `ScreenshotData.swift` becomes `fixtures/screenshot/` (M0-12); `Screenshots/Helpers/ScreenshotTestHelpers.swift` becomes `e2e/support/fixtures.ts` (M2-23).
- **Stale CI entry:** `unit-tests.yml` skips `ScreenshotTests_Usage`, which no longer exists; nothing to port.
- **Gaps (no Swift test exists):** history delete, transcript header tier, session export, settings writes and file history deletes. Their Heimdall tasks define new tests (M1-13, M1-36, M3-40 to M3-43).

## Summary by destination

| Destination | Tests |
|---|---|
| DROP: provider abstraction removed (ADR 0008) | 7 |
| DROP: replaced by the Tauri updater plugin (port-plan P-14) | 10 |
| DROP: reset countdowns belonged to the removed Usage Limits feature (parity X1) | 1 |
| `crates/heimdall-core/src/config/read/frontmatter.rs` | 12 |
| `crates/heimdall-core/src/config/read/mcp.rs` | 8 |
| `crates/heimdall-core/src/config/read/memory.rs` | 7 |
| `crates/heimdall-core/src/data/debug_log.rs` | 22 |
| `crates/heimdall-core/src/data/facets.rs` | 10 |
| `crates/heimdall-core/src/data/file_history.rs` | 5 |
| `crates/heimdall-core/src/data/history.rs` | 12 |
| `crates/heimdall-core/src/data/stats.rs` | 6 |
| `crates/heimdall-core/src/data/stats_cache.rs` | 5 |
| `crates/heimdall-core/src/data/todos.rs` | 13 |
| `crates/heimdall-core/src/data/tray.rs` | 7 |
| `crates/heimdall-core/src/diff.rs` | 13 |
| `crates/heimdall-core/src/export/analytics_csv.rs` | 11 |
| `crates/heimdall-core/src/mcp_status.rs` | 14 |
| `crates/heimdall-core/src/models/catalog/claude_models.rs` | 7 |
| `crates/heimdall-core/src/models/catalog/mcp_catalog.rs` | 18 |
| `crates/heimdall-core/src/models/config/mcp.rs` | 14 |
| `crates/heimdall-core/src/models/config/memory_file.rs` | 15 |
| `crates/heimdall-core/src/models/config/plan.rs` | 12 |
| `crates/heimdall-core/src/models/config/sub_agent.rs` | 7 |
| `crates/heimdall-core/src/models/context_usage.rs` | 4 |
| `crates/heimdall-core/src/models/debug_log_entry.rs` | 6 |
| `crates/heimdall-core/src/models/history_entry.rs` | 5 |
| `crates/heimdall-core/src/models/message.rs` | 18 |
| `crates/heimdall-core/src/models/prefs.rs` | 10 |
| `crates/heimdall-core/src/models/project.rs` | 3 |
| `crates/heimdall-core/src/models/session.rs` | 32 |
| `crates/heimdall-core/src/models/session_facets.rs` | 11 |
| `crates/heimdall-core/src/models/session_group.rs` | 8 |
| `crates/heimdall-core/src/models/session_todo.rs` | 7 |
| `crates/heimdall-core/src/models/stats_cache.rs` | 11 |
| `crates/heimdall-core/src/models/token_usage.rs` | 10 |
| `crates/heimdall-core/src/projects/delete.rs` | 3 |
| `crates/heimdall-core/src/projects/discovery.rs` | 9 |
| `crates/heimdall-core/src/projects/sessions_index.rs` | 6 |
| `crates/heimdall-core/src/projects/subagents.rs` | 1 |
| `crates/heimdall-core/src/search/matcher.rs` | 4 |
| `crates/heimdall-core/src/transcript/full.rs` | 20 |
| `crates/heimdall-core/src/transcript/searchable.rs` | 1 |
| `crates/heimdall-core/src/transcript/summary.rs` | 11 |
| `crates/heimdall-core/src/transcript/system_content.rs` | 9 |
| `crates/heimdall-core/src/transcript/tool_input.rs` | 7 |
| `e2e/visual/later-screens.spec.ts` | 7 |
| `e2e/visual/mvp-screens.spec.ts` | 30 |
| `e2e/visual/session.spec.ts` | 57 |
| `src-tauri/src/prefs.rs` | 4 |
| `src-tauri/src/state.rs` | 3 |
| `src/components/GaugeRing.spec.ts` | 2 |
| `src/composables/useNavigation.spec.ts` | 19 |
| `src/features/analytics/store.spec.ts` | 16 |
| `src/features/config/store.spec.ts` | 5 |
| `src/features/history/store.spec.ts` | 5 |
| `src/features/session/store.spec.ts` | 41 |
| `src/stores/theme.spec.ts` | 12 |
| `src/theme/contrast.spec.ts` | 4 |

## Summary by Swift file

| Swift test file | Destination (task) | Tests |
|---|---|---|
| `App/AppStateCacheTests.swift` | `src-tauri/src/state.rs` (M1-38) | 3 |
| `App/AppStateSessionLoadingTests.swift` | `src/features/session/store.spec.ts` (M2-14) | 11 |
| `App/AppStateTests.swift` | `src/stores/theme.spec.ts` (M2-05) | 5 |
| `App/AppStateTests.swift` | `src/features/session/store.spec.ts` (M2-14) | 24 |
| `App/AppStateTests.swift` | `src/composables/useNavigation.spec.ts` (M2-06) | 9 |
| `App/AppStateTests.swift` | `src/features/config/store.spec.ts` (M3-11) | 5 |
| `App/ColorThemeTests.swift` | `src/stores/theme.spec.ts` (M2-05) | 7 |
| `App/ColorThemeTests.swift` | `src/theme/contrast.spec.ts` (M2-02) | 4 |
| `App/MemorySourcesStoreTests.swift` | `src-tauri/src/prefs.rs` (M1-39) | 4 |
| `Models/ClaudeModelCatalogTests.swift` | `crates/heimdall-core/src/models/catalog/claude_models.rs` (M1-10) | 7 |
| `Models/ContextUsageTests.swift` | `crates/heimdall-core/src/models/context_usage.rs` (M1-08) | 4 |
| `Models/DebugLogEntryTests.swift` | `crates/heimdall-core/src/models/debug_log_entry.rs` (M1-09) | 6 |
| `Models/HistoryEntryTests.swift` | `crates/heimdall-core/src/models/history_entry.rs` (M1-09) | 5 |
| `Models/HistoryEntryTests.swift` | `src/features/history/store.spec.ts` (M3-02) | 5 |
| `Models/MCPServerCatalogTests.swift` | `crates/heimdall-core/src/models/catalog/mcp_catalog.rs` (M1-10) | 14 |
| `Models/MCPServerStatusTests.swift` | `crates/heimdall-core/src/models/config/mcp.rs` (M1-10) | 14 |
| `Models/MemoryFileTests.swift` | `crates/heimdall-core/src/models/config/memory_file.rs` (M1-10) | 15 |
| `Models/MessageTests.swift` | `crates/heimdall-core/src/models/message.rs` (M1-08) | 18 |
| `Models/PlanTests.swift` | `crates/heimdall-core/src/models/config/plan.rs` (M1-10) | 12 |
| `Models/PreferredEditorTests.swift` | `crates/heimdall-core/src/models/prefs.rs` (M1-11) | 6 |
| `Models/PreferredTerminalTests.swift` | `crates/heimdall-core/src/models/prefs.rs` (M1-11) | 4 |
| `Models/ProjectSortingTests.swift` | `src/features/session/store.spec.ts` (M2-14) | 6 |
| `Models/ProjectTests.swift` | `crates/heimdall-core/src/models/project.rs` (M1-08) | 3 |
| `Models/SessionFacetsTests.swift` | `crates/heimdall-core/src/models/session_facets.rs` (M1-09) | 11 |
| `Models/SessionGroupTests.swift` | `crates/heimdall-core/src/models/session_group.rs` (M1-08) | 8 |
| `Models/SessionTests.swift` | `crates/heimdall-core/src/models/session.rs` (M1-08) | 16 |
| `Models/SessionTodoTests.swift` | `crates/heimdall-core/src/models/session_todo.rs` (M1-09) | 7 |
| `Models/SessionXMLStrippingTests.swift` | `crates/heimdall-core/src/models/session.rs` (M1-08) | 16 |
| `Models/StatsCacheTests.swift` | `crates/heimdall-core/src/models/stats_cache.rs` (M1-09) | 11 |
| `Models/StatsCacheTests.swift` | `src/features/analytics/store.spec.ts` (M3-31) | 1 |
| `Models/StatsCacheTests.swift` | `crates/heimdall-core/src/data/stats_cache.rs` (M1-20) | 5 |
| `Models/SubAgentTests.swift` | `crates/heimdall-core/src/models/config/sub_agent.rs` (M1-10) | 7 |
| `Models/TokenUsageTests.swift` | `crates/heimdall-core/src/models/token_usage.rs` (M1-08) | 10 |
| `Models/UsageWindowTests.swift` | `src/components/GaugeRing.spec.ts` (M2-07) | 2 |
| `Models/UsageWindowTests.swift` | DROP: reset countdowns belonged to the removed Usage Limits feature (parity X1) (—) | 1 |
| `Providers/ClaudeCodeProviderTests.swift` | DROP: provider abstraction removed (ADR 0008) (—) | 7 |
| `Providers/NavigationItemTests.swift` | `src/composables/useNavigation.spec.ts` (M2-06) | 10 |
| `Screenshots/ScreenshotTests_Analytics.swift` | `e2e/visual/later-screens.spec.ts` (M3-61) | 7 |
| `Screenshots/ScreenshotTests_ConfigScreens.swift` | `e2e/visual/mvp-screens.spec.ts` (M3-18) | 14 |
| `Screenshots/ScreenshotTests_DebugLog.swift` | `e2e/visual/mvp-screens.spec.ts` (M3-18) | 1 |
| `Screenshots/ScreenshotTests_EditDiff.swift` | `e2e/visual/session.spec.ts` (M2-24) | 4 |
| `Screenshots/ScreenshotTests_Facets.swift` | `e2e/visual/mvp-screens.spec.ts` (M3-18) | 3 |
| `Screenshots/ScreenshotTests_FileHistory.swift` | `e2e/visual/mvp-screens.spec.ts` (M3-18) | 1 |
| `Screenshots/ScreenshotTests_FullViews.swift` | `e2e/visual/session.spec.ts` (M2-24) | 9 |
| `Screenshots/ScreenshotTests_History.swift` | `e2e/visual/mvp-screens.spec.ts` (M3-18) | 3 |
| `Screenshots/ScreenshotTests_Memory.swift` | `e2e/visual/mvp-screens.spec.ts` (M3-18) | 2 |
| `Screenshots/ScreenshotTests_Plans.swift` | `e2e/visual/mvp-screens.spec.ts` (M3-18) | 2 |
| `Screenshots/ScreenshotTests_SessionDetail.swift` | `e2e/visual/session.spec.ts` (M2-24) | 8 |
| `Screenshots/ScreenshotTests_Sidebar.swift` | `e2e/visual/session.spec.ts` (M2-24) | 4 |
| `Screenshots/ScreenshotTests_Standalone.swift` | `e2e/visual/session.spec.ts` (M2-24) | 10 |
| `Screenshots/ScreenshotTests_ThinkingBlocks.swift` | `e2e/visual/session.spec.ts` (M2-24) | 4 |
| `Screenshots/ScreenshotTests_Todos.swift` | `e2e/visual/mvp-screens.spec.ts` (M3-18) | 4 |
| `Screenshots/ScreenshotTests_ToolBlocks.swift` | `e2e/visual/session.spec.ts` (M2-24) | 15 |
| `Screenshots/ScreenshotTests_Windows.swift` | `e2e/visual/session.spec.ts` (M2-24) | 3 |
| `Services/ClaudeConfigLoaderMCPTests.swift` | `crates/heimdall-core/src/config/read/mcp.rs` (M1-26) | 8 |
| `Services/DebugLogLoaderTests.swift` | `crates/heimdall-core/src/data/debug_log.rs` (M1-17) | 22 |
| `Services/FacetsLoaderTests.swift` | `crates/heimdall-core/src/data/facets.rs` (M1-18) | 10 |
| `Services/FileHistoryLoaderTests.swift` | `crates/heimdall-core/src/data/file_history.rs` (M1-19) | 5 |
| `Services/FrontmatterParserTests.swift` | `crates/heimdall-core/src/config/read/frontmatter.rs` (M1-23) | 12 |
| `Services/HistoryLoaderTests.swift` | `crates/heimdall-core/src/data/history.rs` (M1-15) | 12 |
| `Services/MCPServerStatusCheckerTests.swift` | `crates/heimdall-core/src/mcp_status.rs` (M1-28) | 14 |
| `Services/MemoryLoaderTests.swift` | `crates/heimdall-core/src/config/read/memory.rs` (M1-27) | 7 |
| `Services/SessionLoaderTests.swift` | `crates/heimdall-core/src/projects/discovery.rs` (M1-14) | 9 |
| `Services/SessionLoaderTests.swift` | `crates/heimdall-core/src/projects/subagents.rs` (M1-14) | 1 |
| `Services/SessionLoaderTests.swift` | `crates/heimdall-core/src/projects/sessions_index.rs` (M1-14) | 6 |
| `Services/SettingsWriterMCPTests.swift` | `crates/heimdall-core/src/models/catalog/mcp_catalog.rs` (M1-10) | 4 |
| `Services/StatsComputerTests.swift` | `crates/heimdall-core/src/data/stats.rs` (M1-20) | 6 |
| `Services/TodoLoaderTests.swift` | `crates/heimdall-core/src/data/todos.rs` (M1-16) | 13 |
| `Services/TodoLoaderTests.swift` | `crates/heimdall-core/src/projects/delete.rs` (M3-43) | 3 |
| `Services/TranscriptParserTests.swift` | `crates/heimdall-core/src/transcript/full.rs` (M1-12) | 20 |
| `Services/TranscriptParserTests.swift` | `crates/heimdall-core/src/transcript/summary.rs` (M1-13) | 11 |
| `Services/TranscriptParserTests.swift` | `crates/heimdall-core/src/transcript/searchable.rs` (M1-12) | 1 |
| `Services/UpdateCheckerTests.swift` | DROP: replaced by the Tauri updater plugin (port-plan P-14) (—) | 10 |
| `Utilities/HighlightedTextTests.swift` | `crates/heimdall-core/src/search/matcher.rs` (M1-29) | 4 |
| `Utilities/JSONBeautifierTests.swift` | `crates/heimdall-core/src/transcript/tool_input.rs` (M1-12) | 7 |
| `Utilities/LineDiffTests.swift` | `crates/heimdall-core/src/diff.rs` (M1-35) | 13 |
| `Utilities/SystemContentParserTests.swift` | `crates/heimdall-core/src/transcript/system_content.rs` (M1-12) | 9 |
| `Views/AnalyticsCSVExporterTests.swift` | `crates/heimdall-core/src/export/analytics_csv.rs` (M3-36) | 11 |
| `Views/AnalyticsViewModelTests.swift` | `src/features/analytics/store.spec.ts` (M3-31) | 15 |
| `Views/MenuBarStateTests.swift` | `crates/heimdall-core/src/data/tray.rs` (M1-21) | 7 |

## Every test

| # | Swift file:line | Test | Destination | Task |
|---|---|---|---|---|
| 1 | `App/AppStateCacheTests.swift:27` | `cacheSession_storesAndRetrieves` | `src-tauri/src/state.rs` | M1-38 |
| 2 | `App/AppStateCacheTests.swift:39` | `cachedSession_returnsNilForUnknownId` | `src-tauri/src/state.rs` | M1-38 |
| 3 | `App/AppStateCacheTests.swift:48` | `clearCache_removesAllEntries` | `src-tauri/src/state.rs` | M1-38 |
| 4 | `App/AppStateSessionLoadingTests.swift:51` | `selectingCachedSession_setsLoadingFalse` | `src/features/session/store.spec.ts` | M2-14 |
| 5 | `App/AppStateSessionLoadingTests.swift:71` | `selectingNilSession_resetsLoadingFlag` | `src/features/session/store.spec.ts` | M2-14 |
| 6 | `App/AppStateSessionLoadingTests.swift:84` | `selectingFullyLoadedSession_resetsLoadingFlag` | `src/features/session/store.spec.ts` | M2-14 |
| 7 | `App/AppStateSessionLoadingTests.swift:100` | `selectingSessionWithNoFileURL_resetsLoadingFlag` | `src/features/session/store.spec.ts` | M2-14 |
| 8 | `App/AppStateSessionLoadingTests.swift:126` | `deferBlock_resetsFlag_whenSessionStillSelected` | `src/features/session/store.spec.ts` | M2-14 |
| 9 | `App/AppStateSessionLoadingTests.swift:142` | `deferBlock_skipsReset_whenDifferentSessionSelected` | `src/features/session/store.spec.ts` | M2-14 |
| 10 | `App/AppStateSessionLoadingTests.swift:158` | `deferBlock_resetsFlag_afterCancellation` | `src/features/session/store.spec.ts` | M2-14 |
| 11 | `App/AppStateSessionLoadingTests.swift:177` | `rapidSelection_cacheHitAfterStub_resetsLoadingCorrectly` | `src/features/session/store.spec.ts` | M2-14 |
| 12 | `App/AppStateSessionLoadingTests.swift:197` | `rapidSelection_switchBackToCached_resetsCorrectly` | `src/features/session/store.spec.ts` | M2-14 |
| 13 | `App/AppStateSessionLoadingTests.swift:227` | `successfulLoad_cachesAndUpdatesSession` | `src/features/session/store.spec.ts` | M2-14 |
| 14 | `App/AppStateSessionLoadingTests.swift:249` | `successfulLoad_nilParseResult_stillResetsLoading` | `src/features/session/store.spec.ts` | M2-14 |
| 15 | `App/AppStateTests.swift:40` | `increaseFontScale_incrementsByPointZeroFive` | `src/stores/theme.spec.ts` | M2-05 |
| 16 | `App/AppStateTests.swift:50` | `increaseFontScale_clampsAtMaximum` | `src/stores/theme.spec.ts` | M2-05 |
| 17 | `App/AppStateTests.swift:60` | `decreaseFontScale_decrementsBy005` | `src/stores/theme.spec.ts` | M2-05 |
| 18 | `App/AppStateTests.swift:70` | `decreaseFontScale_clampsAtMinimum` | `src/stores/theme.spec.ts` | M2-05 |
| 19 | `App/AppStateTests.swift:80` | `resetFontScale_setsToOne` | `src/stores/theme.spec.ts` | M2-05 |
| 20 | `App/AppStateTests.swift:92` | `currentProject_withMatchingId_returnsProject` | `src/features/session/store.spec.ts` | M2-14 |
| 21 | `App/AppStateTests.swift:102` | `currentProject_withNilSelection_returnsNil` | `src/features/session/store.spec.ts` | M2-14 |
| 22 | `App/AppStateTests.swift:111` | `currentProject_withNonMatchingId_returnsNil` | `src/features/session/store.spec.ts` | M2-14 |
| 23 | `App/AppStateTests.swift:122` | `filteredSortedProjects_excludesEmptyProjects` | `src/features/session/store.spec.ts` | M2-14 |
| 24 | `App/AppStateTests.swift:138` | `filteredSortedProjects_searchFiltersByProjectName` | `src/features/session/store.spec.ts` | M2-14 |
| 25 | `App/AppStateTests.swift:152` | `filteredSortedProjects_searchFiltersBySessionTitle` | `src/features/session/store.spec.ts` | M2-14 |
| 26 | `App/AppStateTests.swift:181` | `filteredSortedProjects_searchIsCaseInsensitive` | `src/features/session/store.spec.ts` | M2-14 |
| 27 | `App/AppStateTests.swift:192` | `filteredSortedProjects_emptySearchReturnsAll` | `src/features/session/store.spec.ts` | M2-14 |
| 28 | `App/AppStateTests.swift:204` | `filteredSortedProjects_sortByRecentActivity` | `src/features/session/store.spec.ts` | M2-14 |
| 29 | `App/AppStateTests.swift:226` | `filteredSortedProjects_sortByName` | `src/features/session/store.spec.ts` | M2-14 |
| 30 | `App/AppStateTests.swift:240` | `filteredSortedProjects_sortBySessionCount` | `src/features/session/store.spec.ts` | M2-14 |
| 31 | `App/AppStateTests.swift:264` | `deleteSession_removesFromProject` | `src/features/session/store.spec.ts` | M2-14 |
| 32 | `App/AppStateTests.swift:276` | `deleteSession_removesEmptyProject` | `src/features/session/store.spec.ts` | M2-14 |
| 33 | `App/AppStateTests.swift:287` | `deleteSession_clearsSelectionIfSelected` | `src/features/session/store.spec.ts` | M2-14 |
| 34 | `App/AppStateTests.swift:299` | `deleteSession_preservesSelectionIfDifferent` | `src/features/session/store.spec.ts` | M2-14 |
| 35 | `App/AppStateTests.swift:312` | `deleteSession_removesFromCache` | `src/features/session/store.spec.ts` | M2-14 |
| 36 | `App/AppStateTests.swift:324` | `deleteSession_leavesOtherProjectsUntouched` | `src/features/session/store.spec.ts` | M2-14 |
| 37 | `App/AppStateTests.swift:341` | `deleteProject_removesFromProjects` | `src/features/session/store.spec.ts` | M2-14 |
| 38 | `App/AppStateTests.swift:352` | `deleteProject_clearsSelectionIfSessionBelongsToProject` | `src/features/session/store.spec.ts` | M2-14 |
| 39 | `App/AppStateTests.swift:365` | `deleteProject_preservesSelectionIfDifferentProject` | `src/features/session/store.spec.ts` | M2-14 |
| 40 | `App/AppStateTests.swift:380` | `deleteProject_removesCachedSessions` | `src/features/session/store.spec.ts` | M2-14 |
| 41 | `App/AppStateTests.swift:398` | `isLoadingSession_defaultsFalse` | `src/features/session/store.spec.ts` | M2-14 |
| 42 | `App/AppStateTests.swift:406` | `initialState_hasCorrectDefaults` | `src/features/session/store.spec.ts` | M2-14 |
| 43 | `App/AppStateTests.swift:425` | `navigationHistory_selectingSession_pushesToHistory` | `src/composables/useNavigation.spec.ts` | M2-06 |
| 44 | `App/AppStateTests.swift:438` | `navigationHistory_canNavigateBack` | `src/composables/useNavigation.spec.ts` | M2-06 |
| 45 | `App/AppStateTests.swift:453` | `navigationHistory_canNavigateForward` | `src/composables/useNavigation.spec.ts` | M2-06 |
| 46 | `App/AppStateTests.swift:469` | `navigationHistory_backDoesNothingAtStart` | `src/composables/useNavigation.spec.ts` | M2-06 |
| 47 | `App/AppStateTests.swift:481` | `navigationHistory_forwardDoesNothingAtEnd` | `src/composables/useNavigation.spec.ts` | M2-06 |
| 48 | `App/AppStateTests.swift:493` | `navigationHistory_newSelectionAfterBack_trimsForwardHistory` | `src/composables/useNavigation.spec.ts` | M2-06 |
| 49 | `App/AppStateTests.swift:512` | `navigationHistory_duplicateSelection_doesNotPush` | `src/composables/useNavigation.spec.ts` | M2-06 |
| 50 | `App/AppStateTests.swift:522` | `navigationHistory_navigateBackDoesNotPush` | `src/composables/useNavigation.spec.ts` | M2-06 |
| 51 | `App/AppStateTests.swift:535` | `navigationHistory_reSettingSameSessionPreservesForward` | `src/composables/useNavigation.spec.ts` | M2-06 |
| 52 | `App/AppStateTests.swift:555` | `effectiveConfigProjectPath_prefersExplicitPicker` | `src/features/config/store.spec.ts` | M3-11 |
| 53 | `App/AppStateTests.swift:566` | `effectiveConfigProjectPath_fallsBackToCurrentProject` | `src/features/config/store.spec.ts` | M3-11 |
| 54 | `App/AppStateTests.swift:577` | `effectiveConfigProjectPath_nilWhenNeitherSet` | `src/features/config/store.spec.ts` | M3-11 |
| 55 | `App/AppStateTests.swift:586` | `configProjectName_extractsLastPathComponent` | `src/features/config/store.spec.ts` | M3-11 |
| 56 | `App/AppStateTests.swift:594` | `configProjectName_nilWhenNoPath` | `src/features/config/store.spec.ts` | M3-11 |
| 57 | `App/AppStateTests.swift:604` | `cacheSession_overwritesExistingEntry` | `src/features/session/store.spec.ts` | M2-14 |
| 58 | `App/ColorThemeTests.swift:8` | `allCases_leadWithOriginalsThenPortedThemes` | `src/stores/theme.spec.ts` | M2-05 |
| 59 | `App/ColorThemeTests.swift:19` | `isDark_marksFixedThemes_andNilForAdaptiveOriginals` | `src/stores/theme.spec.ts` | M2-05 |
| 60 | `App/ColorThemeTests.swift:31` | `rawValues_matchExpected` | `src/stores/theme.spec.ts` | M2-05 |
| 61 | `App/ColorThemeTests.swift:38` | `rawValues_initFromString` | `src/stores/theme.spec.ts` | M2-05 |
| 62 | `App/ColorThemeTests.swift:48` | `labels_matchExpected` | `src/stores/theme.spec.ts` | M2-05 |
| 63 | `App/ColorThemeTests.swift:57` | `defaultPalette_matchesOriginalDesignTokens` | `src/theme/contrast.spec.ts` | M2-02 |
| 64 | `App/ColorThemeTests.swift:70` | `solarizedPalette_usesCanonicalColors` | `src/theme/contrast.spec.ts` | M2-02 |
| 65 | `App/ColorThemeTests.swift:85` | `highContrastPalette_usesMaxContrast` | `src/theme/contrast.spec.ts` | M2-02 |
| 66 | `App/ColorThemeTests.swift:98` | `allPalettes_haveNonZeroHexValues` | `src/theme/contrast.spec.ts` | M2-02 |
| 67 | `App/ColorThemeTests.swift:111` | `storage_defaultIsDefault` | `src/stores/theme.spec.ts` | M2-05 |
| 68 | `App/ColorThemeTests.swift:120` | `storage_persistsSelection` | `src/stores/theme.spec.ts` | M2-05 |
| 69 | `App/MemorySourcesStoreTests.swift:13` | `add_appendsAndPersists` | `src-tauri/src/prefs.rs` | M1-39 |
| 70 | `App/MemorySourcesStoreTests.swift:26` | `add_ignoresDuplicates` | `src-tauri/src/prefs.rs` | M1-39 |
| 71 | `App/MemorySourcesStoreTests.swift:37` | `remove_dropsFolder` | `src-tauri/src/prefs.rs` | M1-39 |
| 72 | `App/MemorySourcesStoreTests.swift:50` | `loadFolders_readsPersistedPaths` | `src-tauri/src/prefs.rs` | M1-39 |
| 73 | `Models/ClaudeModelCatalogTests.swift:6` | `friendlyName_mapsCurrentIds` | `crates/heimdall-core/src/models/catalog/claude_models.rs` | M1-10 |
| 74 | `Models/ClaudeModelCatalogTests.swift:15` | `friendlyName_handlesLegacyOrdering` | `crates/heimdall-core/src/models/catalog/claude_models.rs` | M1-10 |
| 75 | `Models/ClaudeModelCatalogTests.swift:22` | `friendlyName_unknownId_passesThrough` | `crates/heimdall-core/src/models/catalog/claude_models.rs` | M1-10 |
| 76 | `Models/ClaudeModelCatalogTests.swift:27` | `models_includeAllCurated_evenWithNoSessions` | `crates/heimdall-core/src/models/catalog/claude_models.rs` | M1-10 |
| 77 | `Models/ClaudeModelCatalogTests.swift:36` | `opus5_hasDescriptionAndStrengths` | `crates/heimdall-core/src/models/catalog/claude_models.rs` | M1-10 |
| 78 | `Models/ClaudeModelCatalogTests.swift:44` | `fable5_hasDescriptionAndStrengths` | `crates/heimdall-core/src/models/catalog/claude_models.rs` | M1-10 |
| 79 | `Models/ClaudeModelCatalogTests.swift:53` | `models_appendDiscovered_withoutDuplicatingCurated` | `crates/heimdall-core/src/models/catalog/claude_models.rs` | M1-10 |
| 80 | `Models/ContextUsageTests.swift:18` | `from_computesCurrentPeakAndSeries` | `crates/heimdall-core/src/models/context_usage.rs` | M1-08 |
| 81 | `Models/ContextUsageTests.swift:35` | `from_infersExtendedWindowAbove200k` | `crates/heimdall-core/src/models/context_usage.rs` | M1-08 |
| 82 | `Models/ContextUsageTests.swift:46` | `cacheHitRate_aggregatesAcrossTurns` | `crates/heimdall-core/src/models/context_usage.rs` | M1-08 |
| 83 | `Models/ContextUsageTests.swift:59` | `from_noUsage_isEmpty` | `crates/heimdall-core/src/models/context_usage.rs` | M1-08 |
| 84 | `Models/DebugLogEntryTests.swift:7` | `levelRawValues` | `crates/heimdall-core/src/models/debug_log_entry.rs` | M1-09 |
| 85 | `Models/DebugLogEntryTests.swift:14` | `levelLabels` | `crates/heimdall-core/src/models/debug_log_entry.rs` | M1-09 |
| 86 | `Models/DebugLogEntryTests.swift:21` | `allCases_containsThreeLevels` | `crates/heimdall-core/src/models/debug_log_entry.rs` | M1-09 |
| 87 | `Models/DebugLogEntryTests.swift:26` | `identifiable_usesIndex` | `crates/heimdall-core/src/models/debug_log_entry.rs` | M1-09 |
| 88 | `Models/DebugLogEntryTests.swift:37` | `hashable_sameIdAreEqual` | `crates/heimdall-core/src/models/debug_log_entry.rs` | M1-09 |
| 89 | `Models/DebugLogEntryTests.swift:55` | `hashable_differentIdAreNotEqual` | `crates/heimdall-core/src/models/debug_log_entry.rs` | M1-09 |
| 90 | `Models/HistoryEntryTests.swift:7` | `projectName_extractsLastPathComponent` | `crates/heimdall-core/src/models/history_entry.rs` | M1-09 |
| 91 | `Models/HistoryEntryTests.swift:19` | `snippet_truncatesMultipleLines` | `crates/heimdall-core/src/models/history_entry.rs` | M1-09 |
| 92 | `Models/HistoryEntryTests.swift:31` | `snippet_skipsEmptyLines` | `crates/heimdall-core/src/models/history_entry.rs` | M1-09 |
| 93 | `Models/HistoryEntryTests.swift:43` | `timeAgo_returnsRelativeString` | `src/features/history/store.spec.ts` | M3-02 |
| 94 | `Models/HistoryEntryTests.swift:56` | `equality_basedOnId` | `crates/heimdall-core/src/models/history_entry.rs` | M1-09 |
| 95 | `Models/HistoryEntryTests.swift:63` | `inequality_differentIds` | `crates/heimdall-core/src/models/history_entry.rs` | M1-09 |
| 96 | `Models/HistoryEntryTests.swift:73` | `today_groupsCorrectly` | `src/features/history/store.spec.ts` | M3-02 |
| 97 | `Models/HistoryEntryTests.swift:79` | `yesterday_groupsCorrectly` | `src/features/history/store.spec.ts` | M3-02 |
| 98 | `Models/HistoryEntryTests.swift:86` | `older_groupsDistantPast` | `src/features/history/store.spec.ts` | M3-02 |
| 99 | `Models/HistoryEntryTests.swift:93` | `allCases_haveNonEmptyTitles` | `src/features/history/store.spec.ts` | M3-02 |
| 100 | `Models/MCPServerCatalogTests.swift:7` | `catalog_hasEntries` | `crates/heimdall-core/src/models/catalog/mcp_catalog.rs` | M1-10 |
| 101 | `Models/MCPServerCatalogTests.swift:14` | `catalog_allEntriesHaveUniqueIds` | `crates/heimdall-core/src/models/catalog/mcp_catalog.rs` | M1-10 |
| 102 | `Models/MCPServerCatalogTests.swift:23` | `catalog_allEntriesHaveRequiredFields` | `crates/heimdall-core/src/models/catalog/mcp_catalog.rs` | M1-10 |
| 103 | `Models/MCPServerCatalogTests.swift:36` | `search_emptyQuery_returnsAll` | `crates/heimdall-core/src/models/catalog/mcp_catalog.rs` | M1-10 |
| 104 | `Models/MCPServerCatalogTests.swift:42` | `search_byName_findsMatch` | `crates/heimdall-core/src/models/catalog/mcp_catalog.rs` | M1-10 |
| 105 | `Models/MCPServerCatalogTests.swift:48` | `search_byDescription_findsMatch` | `crates/heimdall-core/src/models/catalog/mcp_catalog.rs` | M1-10 |
| 106 | `Models/MCPServerCatalogTests.swift:54` | `search_noMatch_returnsEmpty` | `crates/heimdall-core/src/models/catalog/mcp_catalog.rs` | M1-10 |
| 107 | `Models/MCPServerCatalogTests.swift:60` | `search_caseInsensitive` | `crates/heimdall-core/src/models/catalog/mcp_catalog.rs` | M1-10 |
| 108 | `Models/MCPServerCatalogTests.swift:68` | `grouped_coversAllEntries` | `crates/heimdall-core/src/models/catalog/mcp_catalog.rs` | M1-10 |
| 109 | `Models/MCPServerCatalogTests.swift:75` | `grouped_noCategoryIsEmpty` | `crates/heimdall-core/src/models/catalog/mcp_catalog.rs` | M1-10 |
| 110 | `Models/MCPServerCatalogTests.swift:85` | `categories_allHaveIcons` | `crates/heimdall-core/src/models/catalog/mcp_catalog.rs` | M1-10 |
| 111 | `Models/MCPServerCatalogTests.swift:94` | `envKeys_githubRequiresToken` | `crates/heimdall-core/src/models/catalog/mcp_catalog.rs` | M1-10 |
| 112 | `Models/MCPServerCatalogTests.swift:103` | `envKeys_playwrightRequiresNone` | `crates/heimdall-core/src/models/catalog/mcp_catalog.rs` | M1-10 |
| 113 | `Models/MCPServerCatalogTests.swift:110` | `envKeys_allHaveUniqueIdsPerEntry` | `crates/heimdall-core/src/models/catalog/mcp_catalog.rs` | M1-10 |
| 114 | `Models/MCPServerStatusTests.swift:9` | `allCasesHaveSymbolName` — All status cases have a symbol name | `crates/heimdall-core/src/models/config/mcp.rs` | M1-10 |
| 115 | `Models/MCPServerStatusTests.swift:16` | `allCasesHaveLabel` — All status cases have a label | `crates/heimdall-core/src/models/config/mcp.rs` | M1-10 |
| 116 | `Models/MCPServerStatusTests.swift:23` | `allCasesHaveAccessibilityLabel` — All status cases have an accessibility label | `crates/heimdall-core/src/models/config/mcp.rs` | M1-10 |
| 117 | `Models/MCPServerStatusTests.swift:30` | `connectedSymbol` — Connected status uses circle.fill symbol | `crates/heimdall-core/src/models/config/mcp.rs` | M1-10 |
| 118 | `Models/MCPServerStatusTests.swift:35` | `needsAuthSymbol` — NeedsAuth status uses triangle warning symbol | `crates/heimdall-core/src/models/config/mcp.rs` | M1-10 |
| 119 | `Models/MCPServerStatusTests.swift:42` | `failedSymbol` — Failed status uses xmark circle symbol | `crates/heimdall-core/src/models/config/mcp.rs` | M1-10 |
| 120 | `Models/MCPServerStatusTests.swift:47` | `unreachableSymbol` — Unreachable status uses bolt horizontal symbol | `crates/heimdall-core/src/models/config/mcp.rs` | M1-10 |
| 121 | `Models/MCPServerStatusTests.swift:54` | `startingSymbol` — Starting status uses circle.fill symbol | `crates/heimdall-core/src/models/config/mcp.rs` | M1-10 |
| 122 | `Models/MCPServerStatusTests.swift:59` | `unknownSymbol` — Unknown status uses questionmark symbol | `crates/heimdall-core/src/models/config/mcp.rs` | M1-10 |
| 123 | `Models/MCPServerStatusTests.swift:66` | `statusLabels` — Status labels are human-readable | `crates/heimdall-core/src/models/config/mcp.rs` | M1-10 |
| 124 | `Models/MCPServerStatusTests.swift:76` | `statusRawValues` — Status raw values match expected strings | `crates/heimdall-core/src/models/config/mcp.rs` | M1-10 |
| 125 | `Models/MCPServerStatusTests.swift:88` | `serverDefaultStatus` — MCPServer defaults to unknown status | `crates/heimdall-core/src/models/config/mcp.rs` | M1-10 |
| 126 | `Models/MCPServerStatusTests.swift:107` | `serverExplicitStatus` — MCPServer status can be set explicitly | `crates/heimdall-core/src/models/config/mcp.rs` | M1-10 |
| 127 | `Models/MCPServerStatusTests.swift:127` | `serverMutableStatus` — MCPServer status is mutable | `crates/heimdall-core/src/models/config/mcp.rs` | M1-10 |
| 128 | `Models/MemoryFileTests.swift:9` | `displayName_memoryMdReturnsMemory` | `crates/heimdall-core/src/models/config/memory_file.rs` | M1-10 |
| 129 | `Models/MemoryFileTests.swift:15` | `displayName_lowercaseMemoryMdReturnsMemory` | `crates/heimdall-core/src/models/config/memory_file.rs` | M1-10 |
| 130 | `Models/MemoryFileTests.swift:21` | `displayName_simpleFilename` | `crates/heimdall-core/src/models/config/memory_file.rs` | M1-10 |
| 131 | `Models/MemoryFileTests.swift:27` | `displayName_hyphenatedFilename` | `crates/heimdall-core/src/models/config/memory_file.rs` | M1-10 |
| 132 | `Models/MemoryFileTests.swift:33` | `displayName_underscoredFilename` | `crates/heimdall-core/src/models/config/memory_file.rs` | M1-10 |
| 133 | `Models/MemoryFileTests.swift:39` | `displayName_mixedSeparators` | `crates/heimdall-core/src/models/config/memory_file.rs` | M1-10 |
| 134 | `Models/MemoryFileTests.swift:47` | `isMain_trueForMemoryMd` | `crates/heimdall-core/src/models/config/memory_file.rs` | M1-10 |
| 135 | `Models/MemoryFileTests.swift:60` | `isMain_trueForLowercaseMemoryMd` | `crates/heimdall-core/src/models/config/memory_file.rs` | M1-10 |
| 136 | `Models/MemoryFileTests.swift:73` | `isMain_falseForOtherFiles` | `crates/heimdall-core/src/models/config/memory_file.rs` | M1-10 |
| 137 | `Models/MemoryFileTests.swift:88` | `identifiable_usesIdProperty` | `crates/heimdall-core/src/models/config/memory_file.rs` | M1-10 |
| 138 | `Models/MemoryFileTests.swift:101` | `hashable_equalFilesHaveSameHash` | `crates/heimdall-core/src/models/config/memory_file.rs` | M1-10 |
| 139 | `Models/MemoryFileTests.swift:110` | `hashable_differentFilesAreDifferent` | `crates/heimdall-core/src/models/config/memory_file.rs` | M1-10 |
| 140 | `Models/MemoryFileTests.swift:125` | `fuzzyMatch_matchesMemoryName` | `crates/heimdall-core/src/models/config/memory_file.rs` | M1-10 |
| 141 | `Models/MemoryFileTests.swift:132` | `fuzzyMatch_noMatchReturnsNil` | `crates/heimdall-core/src/models/config/memory_file.rs` | M1-10 |
| 142 | `Models/MemoryFileTests.swift:138` | `fuzzyMatch_matchesMemoryContent` | `crates/heimdall-core/src/models/config/memory_file.rs` | M1-10 |
| 143 | `Models/MessageTests.swift:6` | `textContent_withTextBlocks_joinsWithNewline` | `crates/heimdall-core/src/models/message.rs` | M1-08 |
| 144 | `Models/MessageTests.swift:16` | `textContent_withNoTextBlocks_returnsEmpty` | `crates/heimdall-core/src/models/message.rs` | M1-08 |
| 145 | `Models/MessageTests.swift:28` | `textContent_withMixedBlocks_extractsTextOnly` | `crates/heimdall-core/src/models/message.rs` | M1-08 |
| 146 | `Models/MessageTests.swift:42` | `toolBlocks_extractsToolUseOnly` | `crates/heimdall-core/src/models/message.rs` | M1-08 |
| 147 | `Models/MessageTests.swift:61` | `toolBlocks_withNoTools_returnsEmpty` | `crates/heimdall-core/src/models/message.rs` | M1-08 |
| 148 | `Models/MessageTests.swift:73` | `segments_textOnly_producesSingleTextSegment` | `crates/heimdall-core/src/models/message.rs` | M1-08 |
| 149 | `Models/MessageTests.swift:84` | `segments_toolsOnly_producesSingleToolsSegment` | `crates/heimdall-core/src/models/message.rs` | M1-08 |
| 150 | `Models/MessageTests.swift:97` | `segments_textThenTools_producesTwoSegments` | `crates/heimdall-core/src/models/message.rs` | M1-08 |
| 151 | `Models/MessageTests.swift:109` | `segments_thinkingProducesThinkingSegment` | `crates/heimdall-core/src/models/message.rs` | M1-08 |
| 152 | `Models/MessageTests.swift:120` | `segments_thinkingFlushesAccumulatedText` | `crates/heimdall-core/src/models/message.rs` | M1-08 |
| 153 | `Models/MessageTests.swift:135` | `segments_thinkingFlushesAccumulatedTools` | `crates/heimdall-core/src/models/message.rs` | M1-08 |
| 154 | `Models/MessageTests.swift:151` | `segments_multipleThinkingBlocksInterleave` | `crates/heimdall-core/src/models/message.rs` | M1-08 |
| 155 | `Models/MessageTests.swift:173` | `segments_toolResultsAreIgnored` | `crates/heimdall-core/src/models/message.rs` | M1-08 |
| 156 | `Models/MessageTests.swift:191` | `isEdit_trueForEditTool` | `crates/heimdall-core/src/models/message.rs` | M1-08 |
| 157 | `Models/MessageTests.swift:197` | `isEdit_falseForOtherTools` | `crates/heimdall-core/src/models/message.rs` | M1-08 |
| 158 | `Models/MessageTests.swift:205` | `hasDiffData_trueWhenBothStringsPresent` | `crates/heimdall-core/src/models/message.rs` | M1-08 |
| 159 | `Models/MessageTests.swift:216` | `hasDiffData_falseWhenMissingStrings` | `crates/heimdall-core/src/models/message.rs` | M1-08 |
| 160 | `Models/MessageTests.swift:226` | `hasDiffData_falseForNonEditTool` | `crates/heimdall-core/src/models/message.rs` | M1-08 |
| 161 | `Models/PlanTests.swift:7` | `humanize_convertsDashSlugs` | `crates/heimdall-core/src/models/config/plan.rs` | M1-10 |
| 162 | `Models/PlanTests.swift:13` | `humanize_convertsUnderscoreSlugs` | `crates/heimdall-core/src/models/config/plan.rs` | M1-10 |
| 163 | `Models/PlanTests.swift:19` | `humanize_handlesSingleWord` | `crates/heimdall-core/src/models/config/plan.rs` | M1-10 |
| 164 | `Models/PlanTests.swift:25` | `humanize_handlesMixedSeparators` | `crates/heimdall-core/src/models/config/plan.rs` | M1-10 |
| 165 | `Models/PlanTests.swift:31` | `humanize_handlesEmptyString` | `crates/heimdall-core/src/models/config/plan.rs` | M1-10 |
| 166 | `Models/PlanTests.swift:37` | `identifiable_usesIdProperty` | `crates/heimdall-core/src/models/config/plan.rs` | M1-10 |
| 167 | `Models/PlanTests.swift:48` | `hashable_equalPlansHaveSameHash` | `crates/heimdall-core/src/models/config/plan.rs` | M1-10 |
| 168 | `Models/PlanTests.swift:57` | `hashable_differentPlansAreDifferent` | `crates/heimdall-core/src/models/config/plan.rs` | M1-10 |
| 169 | `Models/PlanTests.swift:76` | `delete_removesFileFromDisk` | `crates/heimdall-core/src/models/config/plan.rs` | M1-10 |
| 170 | `Models/PlanTests.swift:95` | `fuzzyMatch_matchesPlanName` | `crates/heimdall-core/src/models/config/plan.rs` | M1-10 |
| 171 | `Models/PlanTests.swift:102` | `fuzzyMatch_noMatchReturnNil` | `crates/heimdall-core/src/models/config/plan.rs` | M1-10 |
| 172 | `Models/PlanTests.swift:108` | `fuzzyMatch_matchesPlanContent` | `crates/heimdall-core/src/models/config/plan.rs` | M1-10 |
| 173 | `Models/PreferredEditorTests.swift:8` | `allCases_hasFourEditors` | `crates/heimdall-core/src/models/prefs.rs` | M1-11 |
| 174 | `Models/PreferredEditorTests.swift:15` | `rawValues_matchExpected` | `crates/heimdall-core/src/models/prefs.rs` | M1-11 |
| 175 | `Models/PreferredEditorTests.swift:25` | `displayNames_nonEmpty` | `crates/heimdall-core/src/models/prefs.rs` | M1-11 |
| 176 | `Models/PreferredEditorTests.swift:34` | `cliCommands_matchExpected` | `crates/heimdall-core/src/models/prefs.rs` | M1-11 |
| 177 | `Models/PreferredEditorTests.swift:44` | `bundleIdentifiers_nonEmpty` | `crates/heimdall-core/src/models/prefs.rs` | M1-11 |
| 178 | `Models/PreferredEditorTests.swift:51` | `bundleIdentifiers_matchExpected` | `crates/heimdall-core/src/models/prefs.rs` | M1-11 |
| 179 | `Models/PreferredTerminalTests.swift:8` | `allCases_hasSixTerminals` | `crates/heimdall-core/src/models/prefs.rs` | M1-11 |
| 180 | `Models/PreferredTerminalTests.swift:15` | `rawValues_matchExpected` | `crates/heimdall-core/src/models/prefs.rs` | M1-11 |
| 181 | `Models/PreferredTerminalTests.swift:27` | `displayNames_nonEmpty` | `crates/heimdall-core/src/models/prefs.rs` | M1-11 |
| 182 | `Models/PreferredTerminalTests.swift:36` | `bundleIdentifiers_matchExpected` | `crates/heimdall-core/src/models/prefs.rs` | M1-11 |
| 183 | `Models/ProjectSortingTests.swift:7` | `allCases_hasThreeOptions` | `src/features/session/store.spec.ts` | M2-14 |
| 184 | `Models/ProjectSortingTests.swift:12` | `rawValues_areCorrect` | `src/features/session/store.spec.ts` | M2-14 |
| 185 | `Models/ProjectSortingTests.swift:19` | `label_returnsLocalizedStrings` | `src/features/session/store.spec.ts` | M2-14 |
| 186 | `Models/ProjectSortingTests.swift:30` | `rawValues_areCorrect` | `src/features/session/store.spec.ts` | M2-14 |
| 187 | `Models/ProjectSortingTests.swift:36` | `initFromRawValue_worksForValidValues` | `src/features/session/store.spec.ts` | M2-14 |
| 188 | `Models/ProjectSortingTests.swift:42` | `initFromRawValue_returnsNilForInvalid` | `src/features/session/store.spec.ts` | M2-14 |
| 189 | `Models/ProjectTests.swift:7` | `recentSession_returnsMostRecent` | `crates/heimdall-core/src/models/project.rs` | M1-08 |
| 190 | `Models/ProjectTests.swift:23` | `recentSession_withNoSessions_returnsNil` | `crates/heimdall-core/src/models/project.rs` | M1-08 |
| 191 | `Models/ProjectTests.swift:29` | `recentSession_withSingleSession_returnsThatSession` | `crates/heimdall-core/src/models/project.rs` | M1-08 |
| 192 | `Models/SessionFacetsTests.swift:9` | `decodesFullJSON` | `crates/heimdall-core/src/models/session_facets.rs` | M1-09 |
| 193 | `Models/SessionFacetsTests.swift:42` | `decodesEmptyDictionaries` | `crates/heimdall-core/src/models/session_facets.rs` | M1-09 |
| 194 | `Models/SessionFacetsTests.swift:69` | `id_matchesSessionId` | `crates/heimdall-core/src/models/session_facets.rs` | M1-09 |
| 195 | `Models/SessionFacetsTests.swift:94` | `sortedGoalCategories_orderedByCountDescending` | `crates/heimdall-core/src/models/session_facets.rs` | M1-09 |
| 196 | `Models/SessionFacetsTests.swift:106` | `totalFrictionCount_sumsValues` | `crates/heimdall-core/src/models/session_facets.rs` | M1-09 |
| 197 | `Models/SessionFacetsTests.swift:112` | `sortedFrictionItems_filtersZeroValues` | `crates/heimdall-core/src/models/session_facets.rs` | M1-09 |
| 198 | `Models/SessionFacetsTests.swift:122` | `outcomeLabel_mapsKnownValues` | `crates/heimdall-core/src/models/session_facets.rs` | M1-09 |
| 199 | `Models/SessionFacetsTests.swift:131` | `helpfulnessLabel_mapsKnownValues` | `crates/heimdall-core/src/models/session_facets.rs` | M1-09 |
| 200 | `Models/SessionFacetsTests.swift:139` | `sessionTypeLabel_formatsUnderscores` | `crates/heimdall-core/src/models/session_facets.rs` | M1-09 |
| 201 | `Models/SessionFacetsTests.swift:145` | `categoryLabel_formatsUnderscores` | `crates/heimdall-core/src/models/session_facets.rs` | M1-09 |
| 202 | `Models/SessionFacetsTests.swift:151` | `frictionLabel_formatsUnderscores` | `crates/heimdall-core/src/models/session_facets.rs` | M1-09 |
| 203 | `Models/SessionGroupTests.swift:31` | `group_parentWithAgents_groupsTogether` | `crates/heimdall-core/src/models/session_group.rs` | M1-08 |
| 204 | `Models/SessionGroupTests.swift:45` | `group_agentsSortedByStartTime` | `crates/heimdall-core/src/models/session_group.rs` | M1-08 |
| 205 | `Models/SessionGroupTests.swift:57` | `group_multipleParents_createsSeparateGroups` | `crates/heimdall-core/src/models/session_group.rs` | M1-08 |
| 206 | `Models/SessionGroupTests.swift:73` | `group_parentWithNoAgents_createsGroupWithEmptyAgents` | `crates/heimdall-core/src/models/session_group.rs` | M1-08 |
| 207 | `Models/SessionGroupTests.swift:84` | `group_orphanSidechain_becomesStandaloneGroup` | `crates/heimdall-core/src/models/session_group.rs` | M1-08 |
| 208 | `Models/SessionGroupTests.swift:95` | `group_emptySessions_returnsEmpty` | `crates/heimdall-core/src/models/session_group.rs` | M1-08 |
| 209 | `Models/SessionGroupTests.swift:101` | `group_sortedDescendingByParentStartTime` | `crates/heimdall-core/src/models/session_group.rs` | M1-08 |
| 210 | `Models/SessionGroupTests.swift:116` | `allSessions_includesParentAndAgents` | `crates/heimdall-core/src/models/session_group.rs` | M1-08 |
| 211 | `Models/SessionTests.swift:7` | `title_withUserMessage_returnsFirstUserMessageText` | `crates/heimdall-core/src/models/session.rs` | M1-08 |
| 212 | `Models/SessionTests.swift:26` | `title_withNoUserMessage_returnsUntitled` | `crates/heimdall-core/src/models/session.rs` | M1-08 |
| 213 | `Models/SessionTests.swift:45` | `title_withEmptyMessages_returnsUntitled` | `crates/heimdall-core/src/models/session.rs` | M1-08 |
| 214 | `Models/SessionTests.swift:58` | `projectName_returnsLastPathComponent` | `crates/heimdall-core/src/models/session.rs` | M1-08 |
| 215 | `Models/SessionTests.swift:71` | `turnCount_countsOnlyUserMessages` | `crates/heimdall-core/src/models/session.rs` | M1-08 |
| 216 | `Models/SessionTests.swift:96` | `equality_basedOnId` | `crates/heimdall-core/src/models/session.rs` | M1-08 |
| 217 | `Models/SessionTests.swift:112` | `title_withCachedTitle_prefersCached` | `crates/heimdall-core/src/models/session.rs` | M1-08 |
| 218 | `Models/SessionTests.swift:135` | `title_withNilCachedTitle_fallsBackToMessages` | `crates/heimdall-core/src/models/session.rs` | M1-08 |
| 219 | `Models/SessionTests.swift:158` | `turnCount_withCachedTurnCount_prefersCached` | `crates/heimdall-core/src/models/session.rs` | M1-08 |
| 220 | `Models/SessionTests.swift:175` | `turnCount_withNilCachedTurnCount_fallsBackToMessages` | `crates/heimdall-core/src/models/session.rs` | M1-08 |
| 221 | `Models/SessionTests.swift:200` | `fileURL_defaultsToNil` | `crates/heimdall-core/src/models/session.rs` | M1-08 |
| 222 | `Models/SessionTests.swift:206` | `fileURL_preservesValue` | `crates/heimdall-core/src/models/session.rs` | M1-08 |
| 223 | `Models/SessionTests.swift:223` | `firstPrompt_defaultsToNil` | `crates/heimdall-core/src/models/session.rs` | M1-08 |
| 224 | `Models/SessionTests.swift:236` | `firstPrompt_preservesValue` | `crates/heimdall-core/src/models/session.rs` | M1-08 |
| 225 | `Models/SessionTests.swift:250` | `firstPrompt_preservesNilExplicitly` | `crates/heimdall-core/src/models/session.rs` | M1-08 |
| 226 | `Models/SessionTests.swift:264` | `firstPrompt_preservesEmptyString` | `crates/heimdall-core/src/models/session.rs` | M1-08 |
| 227 | `Models/SessionTodoTests.swift:7` | `decodesCompletedStatus` | `crates/heimdall-core/src/models/session_todo.rs` | M1-09 |
| 228 | `Models/SessionTodoTests.swift:18` | `decodesInProgressStatus` | `crates/heimdall-core/src/models/session_todo.rs` | M1-09 |
| 229 | `Models/SessionTodoTests.swift:27` | `decodesPendingStatus` | `crates/heimdall-core/src/models/session_todo.rs` | M1-09 |
| 230 | `Models/SessionTodoTests.swift:36` | `decodesArrayOfTodos` | `crates/heimdall-core/src/models/session_todo.rs` | M1-09 |
| 231 | `Models/SessionTodoTests.swift:52` | `identifiable_idDerivedFromContent` | `crates/heimdall-core/src/models/session_todo.rs` | M1-09 |
| 232 | `Models/SessionTodoTests.swift:61` | `hashable_todosWithSameContentAreEqual` | `crates/heimdall-core/src/models/session_todo.rs` | M1-09 |
| 233 | `Models/SessionTodoTests.swift:75` | `decodesEmptyArray` | `crates/heimdall-core/src/models/session_todo.rs` | M1-09 |
| 234 | `Models/SessionXMLStrippingTests.swift:58` | `title_stripsTagMarkers_preservesInnerText` | `crates/heimdall-core/src/models/session.rs` | M1-08 |
| 235 | `Models/SessionXMLStrippingTests.swift:65` | `title_stripsMultipleTagMarkers` | `crates/heimdall-core/src/models/session.rs` | M1-08 |
| 236 | `Models/SessionXMLStrippingTests.swift:71` | `title_stripsNestedTagMarkers` | `crates/heimdall-core/src/models/session.rs` | M1-08 |
| 237 | `Models/SessionXMLStrippingTests.swift:77` | `title_preservesPlainText` | `crates/heimdall-core/src/models/session.rs` | M1-08 |
| 238 | `Models/SessionXMLStrippingTests.swift:83` | `title_stripsTagMarkersFromCachedTitle` | `crates/heimdall-core/src/models/session.rs` | M1-08 |
| 239 | `Models/SessionXMLStrippingTests.swift:89` | `title_onlyTagMarkers_innerTextPreserved` | `crates/heimdall-core/src/models/session.rs` | M1-08 |
| 240 | `Models/SessionXMLStrippingTests.swift:96` | `title_onlyEmptyTags_fallsBackToOriginal` | `crates/heimdall-core/src/models/session.rs` | M1-08 |
| 241 | `Models/SessionXMLStrippingTests.swift:103` | `title_trimsWhitespaceAfterStripping` | `crates/heimdall-core/src/models/session.rs` | M1-08 |
| 242 | `Models/SessionXMLStrippingTests.swift:109` | `title_selfClosingTags_stripped` | `crates/heimdall-core/src/models/session.rs` | M1-08 |
| 243 | `Models/SessionXMLStrippingTests.swift:115` | `title_angledBracketsMatchedAsTag` | `crates/heimdall-core/src/models/session.rs` | M1-08 |
| 244 | `Models/SessionXMLStrippingTests.swift:124` | `preview_stripsTagMarkers_preservesContent` | `crates/heimdall-core/src/models/session.rs` | M1-08 |
| 245 | `Models/SessionXMLStrippingTests.swift:130` | `preview_plainTextUnchanged` | `crates/heimdall-core/src/models/session.rs` | M1-08 |
| 246 | `Models/SessionXMLStrippingTests.swift:136` | `preview_nilWhenNoAssistantMessage` | `crates/heimdall-core/src/models/session.rs` | M1-08 |
| 247 | `Models/SessionXMLStrippingTests.swift:155` | `preview_cachedPreviewTakesPriority` | `crates/heimdall-core/src/models/session.rs` | M1-08 |
| 248 | `Models/SessionXMLStrippingTests.swift:175` | `preview_cachedPreviewAlsoStripsTagMarkers` | `crates/heimdall-core/src/models/session.rs` | M1-08 |
| 249 | `Models/SessionXMLStrippingTests.swift:189` | `preview_nilForEmptyMessages` | `crates/heimdall-core/src/models/session.rs` | M1-08 |
| 250 | `Models/StatsCacheTests.swift:84` | `decode_fullJSON_succeeds` | `crates/heimdall-core/src/models/stats_cache.rs` | M1-09 |
| 251 | `Models/StatsCacheTests.swift:96` | `decode_dailyActivity_parsesCorrectly` | `crates/heimdall-core/src/models/stats_cache.rs` | M1-09 |
| 252 | `Models/StatsCacheTests.swift:108` | `decode_dailyModelTokens_parsesMultiModel` | `crates/heimdall-core/src/models/stats_cache.rs` | M1-09 |
| 253 | `Models/StatsCacheTests.swift:120` | `decode_modelUsage_parsesTokenCounts` | `crates/heimdall-core/src/models/stats_cache.rs` | M1-09 |
| 254 | `Models/StatsCacheTests.swift:133` | `decode_longestSession_parsesAllFields` | `crates/heimdall-core/src/models/stats_cache.rs` | M1-09 |
| 255 | `Models/StatsCacheTests.swift:143` | `decode_hourCounts_parsesStringKeys` | `crates/heimdall-core/src/models/stats_cache.rs` | M1-09 |
| 256 | `Models/StatsCacheTests.swift:155` | `longestSessionFormatted_showsHoursAndMinutes` | `src/features/analytics/store.spec.ts` | M3-31 |
| 257 | `Models/StatsCacheTests.swift:164` | `sortedHourCounts_sortsAscending` | `crates/heimdall-core/src/models/stats_cache.rs` | M1-09 |
| 258 | `Models/StatsCacheTests.swift:176` | `totalOutputTokens_sumsAcrossModels` | `crates/heimdall-core/src/models/stats_cache.rs` | M1-09 |
| 259 | `Models/StatsCacheTests.swift:184` | `friendlyModelName_mapsKnownModels` | `crates/heimdall-core/src/models/stats_cache.rs` | M1-09 |
| 260 | `Models/StatsCacheTests.swift:192` | `firstSessionParsedDate_parsesISO8601` | `crates/heimdall-core/src/models/stats_cache.rs` | M1-09 |
| 261 | `Models/StatsCacheTests.swift:205` | `totalToolCalls_sumsAcrossDays` | `crates/heimdall-core/src/models/stats_cache.rs` | M1-09 |
| 262 | `Models/StatsCacheTests.swift:215` | `loader_missingFile_returnsNil` | `crates/heimdall-core/src/data/stats_cache.rs` | M1-20 |
| 263 | `Models/StatsCacheTests.swift:223` | `isStale_tokensLagActivity_isTrue` | `crates/heimdall-core/src/data/stats_cache.rs` | M1-20 |
| 264 | `Models/StatsCacheTests.swift:239` | `isStale_lastComputedBeforeLatestSession_isTrue` | `crates/heimdall-core/src/data/stats_cache.rs` | M1-20 |
| 265 | `Models/StatsCacheTests.swift:255` | `loadAnalytics_staleCache_recomputesFromTranscripts` | `crates/heimdall-core/src/data/stats_cache.rs` | M1-20 |
| 266 | `Models/StatsCacheTests.swift:298` | `loadAnalytics_freshCache_isKept` | `crates/heimdall-core/src/data/stats_cache.rs` | M1-20 |
| 267 | `Models/SubAgentTests.swift:8` | `builtIn_hasFourAgents` | `crates/heimdall-core/src/models/config/sub_agent.rs` | M1-10 |
| 268 | `Models/SubAgentTests.swift:15` | `builtIn_hasExpectedIds` | `crates/heimdall-core/src/models/config/sub_agent.rs` | M1-10 |
| 269 | `Models/SubAgentTests.swift:26` | `builtIn_allHaveNonEmptyFields` | `crates/heimdall-core/src/models/config/sub_agent.rs` | M1-10 |
| 270 | `Models/SubAgentTests.swift:38` | `builtIn_exploreAgent_hasExpectedTools` | `crates/heimdall-core/src/models/config/sub_agent.rs` | M1-10 |
| 271 | `Models/SubAgentTests.swift:49` | `builtIn_bashAgent_hasOnlyBashTool` | `crates/heimdall-core/src/models/config/sub_agent.rs` | M1-10 |
| 272 | `Models/SubAgentTests.swift:57` | `builtIn_allAreBuiltInScope` | `crates/heimdall-core/src/models/config/sub_agent.rs` | M1-10 |
| 273 | `Models/SubAgentTests.swift:68` | `knownTools_isNonEmpty` | `crates/heimdall-core/src/models/config/sub_agent.rs` | M1-10 |
| 274 | `Models/TokenUsageTests.swift:8` | `total_sumsInputAndOutput` | `crates/heimdall-core/src/models/token_usage.rs` | M1-08 |
| 275 | `Models/TokenUsageTests.swift:16` | `formatted_below1000_returnsPlainInt` | `crates/heimdall-core/src/models/token_usage.rs` | M1-08 |
| 276 | `Models/TokenUsageTests.swift:22` | `formatted_exactly1000_returns1_0k` | `crates/heimdall-core/src/models/token_usage.rs` | M1-08 |
| 277 | `Models/TokenUsageTests.swift:28` | `formatted_above1000_returnsDecimalK` | `crates/heimdall-core/src/models/token_usage.rs` | M1-08 |
| 278 | `Models/TokenUsageTests.swift:34` | `formatted_zeroTokens_returnsZero` | `crates/heimdall-core/src/models/token_usage.rs` | M1-08 |
| 279 | `Models/TokenUsageTests.swift:40` | `formatted_largeValue_correctFormat` | `crates/heimdall-core/src/models/token_usage.rs` | M1-08 |
| 280 | `Models/TokenUsageTests.swift:48` | `contextTokens_sumsPromptSide` | `crates/heimdall-core/src/models/token_usage.rs` | M1-08 |
| 281 | `Models/TokenUsageTests.swift:55` | `cacheHitRate_fractionOfPromptFromCache` | `crates/heimdall-core/src/models/token_usage.rs` | M1-08 |
| 282 | `Models/TokenUsageTests.swift:62` | `cacheHitRate_noPrompt_isZero` | `crates/heimdall-core/src/models/token_usage.rs` | M1-08 |
| 283 | `Models/TokenUsageTests.swift:67` | `legacyInit_defaultsCacheToZero` | `crates/heimdall-core/src/models/token_usage.rs` | M1-08 |
| 284 | `Models/UsageWindowTests.swift:7` | `fractionAndPercent` | `src/components/GaugeRing.spec.ts` | M2-07 |
| 285 | `Models/UsageWindowTests.swift:15` | `severityThresholds` | `src/components/GaugeRing.spec.ts` | M2-07 |
| 286 | `Models/UsageWindowTests.swift:25` | `timeUntilReset` | DROP: reset countdowns belonged to the removed Usage Limits feature (parity X1) | — |
| 287 | `Providers/ClaudeCodeProviderTests.swift:8` | `name_isClaudeCode` | DROP: provider abstraction removed (ADR 0008) | — |
| 288 | `Providers/ClaudeCodeProviderTests.swift:13` | `supportsAllCapabilities` | DROP: provider abstraction removed (ADR 0008) | — |
| 289 | `Providers/ClaudeCodeProviderTests.swift:20` | `navigationItems_includesAllFourteen` | DROP: provider abstraction removed (ADR 0008) | — |
| 290 | `Providers/ClaudeCodeProviderTests.swift:25` | `toolDefinitions_coversAllClaudeTools` | DROP: provider abstraction removed (ADR 0008) | — |
| 291 | `Providers/ClaudeCodeProviderTests.swift:33` | `toolIcon_unknownTool_returnsFallback` | DROP: provider abstraction removed (ADR 0008) | — |
| 292 | `Providers/ClaudeCodeProviderTests.swift:38` | `configurationItems_hasTenItems` | DROP: provider abstraction removed (ADR 0008) | — |
| 293 | `Providers/ClaudeCodeProviderTests.swift:43` | `projectsPath_endsWithClaudeProjects` | DROP: provider abstraction removed (ADR 0008) | — |
| 294 | `Providers/NavigationItemTests.swift:6` | `allItems_hasFourteenItems` | `src/composables/useNavigation.spec.ts` | M2-06 |
| 295 | `Providers/NavigationItemTests.swift:11` | `analytics_hasNoRequiredCapability` | `src/composables/useNavigation.spec.ts` | M2-06 |
| 296 | `Providers/NavigationItemTests.swift:16` | `history_hasNoRequiredCapability` | `src/composables/useNavigation.spec.ts` | M2-06 |
| 297 | `Providers/NavigationItemTests.swift:21` | `history_hasCorrectSystemImage` | `src/composables/useNavigation.spec.ts` | M2-06 |
| 298 | `Providers/NavigationItemTests.swift:26` | `analytics_hasCorrectSystemImage` | `src/composables/useNavigation.spec.ts` | M2-06 |
| 299 | `Providers/NavigationItemTests.swift:31` | `sessions_hasNoRequiredCapability` | `src/composables/useNavigation.spec.ts` | M2-06 |
| 300 | `Providers/NavigationItemTests.swift:36` | `skills_requiresSkillsCapability` | `src/composables/useNavigation.spec.ts` | M2-06 |
| 301 | `Providers/NavigationItemTests.swift:41` | `plans_requiresPlansCapability` | `src/composables/useNavigation.spec.ts` | M2-06 |
| 302 | `Providers/NavigationItemTests.swift:46` | `memory_requiresMemoryCapability` | `src/composables/useNavigation.spec.ts` | M2-06 |
| 303 | `Providers/NavigationItemTests.swift:51` | `memory_hasCorrectSystemImage` | `src/composables/useNavigation.spec.ts` | M2-06 |
| 304 | `Screenshots/ScreenshotTests_Analytics.swift:16` | `testAnalyticsDashboard` | `e2e/visual/later-screens.spec.ts` | M3-61 |
| 305 | `Screenshots/ScreenshotTests_Analytics.swift:21` | `testAnalyticsDashboardLight` | `e2e/visual/later-screens.spec.ts` | M3-61 |
| 306 | `Screenshots/ScreenshotTests_Analytics.swift:28` | `testAnalyticsDashboardMid` | `e2e/visual/later-screens.spec.ts` | M3-61 |
| 307 | `Screenshots/ScreenshotTests_Analytics.swift:33` | `testAnalyticsDashboardMidLight` | `e2e/visual/later-screens.spec.ts` | M3-61 |
| 308 | `Screenshots/ScreenshotTests_Analytics.swift:47` | `testAnalyticsDashboardBottom` | `e2e/visual/later-screens.spec.ts` | M3-61 |
| 309 | `Screenshots/ScreenshotTests_Analytics.swift:56` | `testAnalyticsDashboardBottomLight` | `e2e/visual/later-screens.spec.ts` | M3-61 |
| 310 | `Screenshots/ScreenshotTests_Analytics.swift:68` | `testAnalyticsDashboardContent` | `e2e/visual/later-screens.spec.ts` | M3-61 |
| 311 | `Screenshots/ScreenshotTests_ConfigScreens.swift:18` | `testCommandsList` | `e2e/visual/mvp-screens.spec.ts` | M3-18 |
| 312 | `Screenshots/ScreenshotTests_ConfigScreens.swift:33` | `testSkillsList` | `e2e/visual/mvp-screens.spec.ts` | M3-18 |
| 313 | `Screenshots/ScreenshotTests_ConfigScreens.swift:47` | `testMCPServersList` | `e2e/visual/mvp-screens.spec.ts` | M3-18 |
| 314 | `Screenshots/ScreenshotTests_ConfigScreens.swift:62` | `testModelsList` | `e2e/visual/mvp-screens.spec.ts` | M3-18 |
| 315 | `Screenshots/ScreenshotTests_ConfigScreens.swift:76` | `testSubAgentsList` | `e2e/visual/mvp-screens.spec.ts` | M3-18 |
| 316 | `Screenshots/ScreenshotTests_ConfigScreens.swift:91` | `testPluginsList` | `e2e/visual/mvp-screens.spec.ts` | M3-18 |
| 317 | `Screenshots/ScreenshotTests_ConfigScreens.swift:105` | `testHooksList` | `e2e/visual/mvp-screens.spec.ts` | M3-18 |
| 318 | `Screenshots/ScreenshotTests_ConfigScreens.swift:121` | `testOutputStylesList` | `e2e/visual/mvp-screens.spec.ts` | M3-18 |
| 319 | `Screenshots/ScreenshotTests_ConfigScreens.swift:138` | `testCommandsListWithProject` | `e2e/visual/mvp-screens.spec.ts` | M3-18 |
| 320 | `Screenshots/ScreenshotTests_ConfigScreens.swift:154` | `testSkillsListWithProject` | `e2e/visual/mvp-screens.spec.ts` | M3-18 |
| 321 | `Screenshots/ScreenshotTests_ConfigScreens.swift:170` | `testMCPServersListWithProject` | `e2e/visual/mvp-screens.spec.ts` | M3-18 |
| 322 | `Screenshots/ScreenshotTests_ConfigScreens.swift:188` | `testConfigScreenHeader` | `e2e/visual/mvp-screens.spec.ts` | M3-18 |
| 323 | `Screenshots/ScreenshotTests_ConfigScreens.swift:206` | `testConfigEmptyState` | `e2e/visual/mvp-screens.spec.ts` | M3-18 |
| 324 | `Screenshots/ScreenshotTests_ConfigScreens.swift:222` | `testConfigEmptyStateMCP` | `e2e/visual/mvp-screens.spec.ts` | M3-18 |
| 325 | `Screenshots/ScreenshotTests_DebugLog.swift:39` | `testDebugLogView` | `e2e/visual/mvp-screens.spec.ts` | M3-18 |
| 326 | `Screenshots/ScreenshotTests_EditDiff.swift:12` | `testSimpleDiff` | `e2e/visual/session.spec.ts` | M2-24 |
| 327 | `Screenshots/ScreenshotTests_EditDiff.swift:28` | `testMultiLineDiff` | `e2e/visual/session.spec.ts` | M2-24 |
| 328 | `Screenshots/ScreenshotTests_EditDiff.swift:44` | `testAddOnlyDiff` | `e2e/visual/session.spec.ts` | M2-24 |
| 329 | `Screenshots/ScreenshotTests_EditDiff.swift:60` | `testRemoveOnlyDiff` | `e2e/visual/session.spec.ts` | M2-24 |
| 330 | `Screenshots/ScreenshotTests_Facets.swift:10` | `testFacetsCardExpanded` | `e2e/visual/mvp-screens.spec.ts` | M3-18 |
| 331 | `Screenshots/ScreenshotTests_Facets.swift:24` | `testFacetsCardLight` | `e2e/visual/mvp-screens.spec.ts` | M3-18 |
| 332 | `Screenshots/ScreenshotTests_Facets.swift:39` | `testFacetsCardPartialOutcome` | `e2e/visual/mvp-screens.spec.ts` | M3-18 |
| 333 | `Screenshots/ScreenshotTests_FileHistory.swift:287` | `testFileHistory` | `e2e/visual/mvp-screens.spec.ts` | M3-18 |
| 334 | `Screenshots/ScreenshotTests_FullViews.swift:12` | `testHero` | `e2e/visual/session.spec.ts` | M2-24 |
| 335 | `Screenshots/ScreenshotTests_FullViews.swift:35` | `testSessionBrowser` | `e2e/visual/session.spec.ts` | M2-24 |
| 336 | `Screenshots/ScreenshotTests_FullViews.swift:56` | `testMultiAgentSessions` | `e2e/visual/session.spec.ts` | M2-24 |
| 337 | `Screenshots/ScreenshotTests_FullViews.swift:77` | `testConversationView` | `e2e/visual/session.spec.ts` | M2-24 |
| 338 | `Screenshots/ScreenshotTests_FullViews.swift:93` | `testToolBlocks` | `e2e/visual/session.spec.ts` | M2-24 |
| 339 | `Screenshots/ScreenshotTests_FullViews.swift:110` | `testThinkingBlocks` | `e2e/visual/session.spec.ts` | M2-24 |
| 340 | `Screenshots/ScreenshotTests_FullViews.swift:126` | `testSearch` | `e2e/visual/session.spec.ts` | M2-24 |
| 341 | `Screenshots/ScreenshotTests_FullViews.swift:147` | `testConfigScreen` | `e2e/visual/session.spec.ts` | M2-24 |
| 342 | `Screenshots/ScreenshotTests_FullViews.swift:164` | `testSettings` | `e2e/visual/session.spec.ts` | M2-24 |
| 343 | `Screenshots/ScreenshotTests_History.swift:11` | `testHistoryList` | `e2e/visual/mvp-screens.spec.ts` | M3-18 |
| 344 | `Screenshots/ScreenshotTests_History.swift:26` | `testHistoryFullApp` | `e2e/visual/mvp-screens.spec.ts` | M3-18 |
| 345 | `Screenshots/ScreenshotTests_History.swift:42` | `testHistoryGrid` | `e2e/visual/mvp-screens.spec.ts` | M3-18 |
| 346 | `Screenshots/ScreenshotTests_Memory.swift:16` | `testMemoryList` | `e2e/visual/mvp-screens.spec.ts` | M3-18 |
| 347 | `Screenshots/ScreenshotTests_Memory.swift:31` | `testMemoryFullApp` | `e2e/visual/mvp-screens.spec.ts` | M3-18 |
| 348 | `Screenshots/ScreenshotTests_Plans.swift:16` | `testPlansList` | `e2e/visual/mvp-screens.spec.ts` | M3-18 |
| 349 | `Screenshots/ScreenshotTests_Plans.swift:31` | `testPlansFullApp` | `e2e/visual/mvp-screens.spec.ts` | M3-18 |
| 350 | `Screenshots/ScreenshotTests_SessionDetail.swift:20` | `testSessionConversation` | `e2e/visual/session.spec.ts` | M2-24 |
| 351 | `Screenshots/ScreenshotTests_SessionDetail.swift:36` | `testSessionToolBlocks` | `e2e/visual/session.spec.ts` | M2-24 |
| 352 | `Screenshots/ScreenshotTests_SessionDetail.swift:52` | `testSessionThinking` | `e2e/visual/session.spec.ts` | M2-24 |
| 353 | `Screenshots/ScreenshotTests_SessionDetail.swift:68` | `testSessionErrors` | `e2e/visual/session.spec.ts` | M2-24 |
| 354 | `Screenshots/ScreenshotTests_SessionDetail.swift:85` | `testSessionEditDiffs` | `e2e/visual/session.spec.ts` | M2-24 |
| 355 | `Screenshots/ScreenshotTests_SessionDetail.swift:101` | `testSessionSearchActive` | `e2e/visual/session.spec.ts` | M2-24 |
| 356 | `Screenshots/ScreenshotTests_SessionDetail.swift:124` | `testSessionToolFilter` | `e2e/visual/session.spec.ts` | M2-24 |
| 357 | `Screenshots/ScreenshotTests_SessionDetail.swift:147` | `testSessionEmpty` | `e2e/visual/session.spec.ts` | M2-24 |
| 358 | `Screenshots/ScreenshotTests_Sidebar.swift:12` | `testSidebarWithProjects` | `e2e/visual/session.spec.ts` | M2-24 |
| 359 | `Screenshots/ScreenshotTests_Sidebar.swift:30` | `testSidebarLoading` | `e2e/visual/session.spec.ts` | M2-24 |
| 360 | `Screenshots/ScreenshotTests_Sidebar.swift:50` | `testSidebarConfigActive` | `e2e/visual/session.spec.ts` | M2-24 |
| 361 | `Screenshots/ScreenshotTests_Sidebar.swift:66` | `testSidebarSearch` | `e2e/visual/session.spec.ts` | M2-24 |
| 362 | `Screenshots/ScreenshotTests_Standalone.swift:12` | `testToastSuccess` | `e2e/visual/session.spec.ts` | M2-24 |
| 363 | `Screenshots/ScreenshotTests_Standalone.swift:25` | `testToastError` | `e2e/visual/session.spec.ts` | M2-24 |
| 364 | `Screenshots/ScreenshotTests_Standalone.swift:38` | `testToastInfo` | `e2e/visual/session.spec.ts` | M2-24 |
| 365 | `Screenshots/ScreenshotTests_Standalone.swift:53` | `testConfigBadgeAccent` | `e2e/visual/session.spec.ts` | M2-24 |
| 366 | `Screenshots/ScreenshotTests_Standalone.swift:63` | `testConfigBadgeBlue` | `e2e/visual/session.spec.ts` | M2-24 |
| 367 | `Screenshots/ScreenshotTests_Standalone.swift:73` | `testConfigBadgeGreen` | `e2e/visual/session.spec.ts` | M2-24 |
| 368 | `Screenshots/ScreenshotTests_Standalone.swift:85` | `testConfigScopeBadgeProject` | `e2e/visual/session.spec.ts` | M2-24 |
| 369 | `Screenshots/ScreenshotTests_Standalone.swift:95` | `testConfigScopeBadgeGlobal` | `e2e/visual/session.spec.ts` | M2-24 |
| 370 | `Screenshots/ScreenshotTests_Standalone.swift:107` | `testProjectSessionsGrid` | `e2e/visual/session.spec.ts` | M2-24 |
| 371 | `Screenshots/ScreenshotTests_Standalone.swift:124` | `testSessionSkeleton` | `e2e/visual/session.spec.ts` | M2-24 |
| 372 | `Screenshots/ScreenshotTests_ThinkingBlocks.swift:12` | `testThinkingCollapsed` | `e2e/visual/session.spec.ts` | M2-24 |
| 373 | `Screenshots/ScreenshotTests_ThinkingBlocks.swift:24` | `testThinkingExpanded` | `e2e/visual/session.spec.ts` | M2-24 |
| 374 | `Screenshots/ScreenshotTests_ThinkingBlocks.swift:56` | `testThinkingLongContent` | `e2e/visual/session.spec.ts` | M2-24 |
| 375 | `Screenshots/ScreenshotTests_ThinkingBlocks.swift:76` | `testThinkingShort` | `e2e/visual/session.spec.ts` | M2-24 |
| 376 | `Screenshots/ScreenshotTests_Todos.swift:10` | `testTodosList` | `e2e/visual/mvp-screens.spec.ts` | M3-18 |
| 377 | `Screenshots/ScreenshotTests_Todos.swift:24` | `testTodosFullApp` | `e2e/visual/mvp-screens.spec.ts` | M3-18 |
| 378 | `Screenshots/ScreenshotTests_Todos.swift:41` | `testTodosFullAppSolarized` | `e2e/visual/mvp-screens.spec.ts` | M3-18 |
| 379 | `Screenshots/ScreenshotTests_Todos.swift:61` | `testTodosFullAppHighContrast` | `e2e/visual/mvp-screens.spec.ts` | M3-18 |
| 380 | `Screenshots/ScreenshotTests_ToolBlocks.swift:12` | `testReadToolCollapsed` | `e2e/visual/session.spec.ts` | M2-24 |
| 381 | `Screenshots/ScreenshotTests_ToolBlocks.swift:24` | `testWriteToolCollapsed` | `e2e/visual/session.spec.ts` | M2-24 |
| 382 | `Screenshots/ScreenshotTests_ToolBlocks.swift:36` | `testEditToolCollapsed` | `e2e/visual/session.spec.ts` | M2-24 |
| 383 | `Screenshots/ScreenshotTests_ToolBlocks.swift:48` | `testBashToolCollapsed` | `e2e/visual/session.spec.ts` | M2-24 |
| 384 | `Screenshots/ScreenshotTests_ToolBlocks.swift:60` | `testGlobToolCollapsed` | `e2e/visual/session.spec.ts` | M2-24 |
| 385 | `Screenshots/ScreenshotTests_ToolBlocks.swift:72` | `testGrepToolCollapsed` | `e2e/visual/session.spec.ts` | M2-24 |
| 386 | `Screenshots/ScreenshotTests_ToolBlocks.swift:84` | `testTaskToolCollapsed` | `e2e/visual/session.spec.ts` | M2-24 |
| 387 | `Screenshots/ScreenshotTests_ToolBlocks.swift:98` | `testReadToolExpanded` | `e2e/visual/session.spec.ts` | M2-24 |
| 388 | `Screenshots/ScreenshotTests_ToolBlocks.swift:113` | `testBashToolExpanded` | `e2e/visual/session.spec.ts` | M2-24 |
| 389 | `Screenshots/ScreenshotTests_ToolBlocks.swift:130` | `testBashErrorTool` | `e2e/visual/session.spec.ts` | M2-24 |
| 390 | `Screenshots/ScreenshotTests_ToolBlocks.swift:147` | `testUnknownTool` | `e2e/visual/session.spec.ts` | M2-24 |
| 391 | `Screenshots/ScreenshotTests_ToolBlocks.swift:161` | `testEditDiffExpanded` | `e2e/visual/session.spec.ts` | M2-24 |
| 392 | `Screenshots/ScreenshotTests_ToolBlocks.swift:192` | `testLongContentTruncated` | `e2e/visual/session.spec.ts` | M2-24 |
| 393 | `Screenshots/ScreenshotTests_ToolBlocks.swift:209` | `testNoOutputTool` | `e2e/visual/session.spec.ts` | M2-24 |
| 394 | `Screenshots/ScreenshotTests_ToolBlocks.swift:226` | `testBashErrorCollapsed` | `e2e/visual/session.spec.ts` | M2-24 |
| 395 | `Screenshots/ScreenshotTests_Windows.swift:12` | `testSettingsWindow` | `e2e/visual/session.spec.ts` | M2-24 |
| 396 | `Screenshots/ScreenshotTests_Windows.swift:25` | `testOnboardingWindow` | `e2e/visual/session.spec.ts` | M2-24 |
| 397 | `Screenshots/ScreenshotTests_Windows.swift:39` | `testHelpWindow` | `e2e/visual/session.spec.ts` | M2-24 |
| 398 | `Services/ClaudeConfigLoaderMCPTests.swift:9` | `decodeClaudeConfig_userScopeServers` | `crates/heimdall-core/src/config/read/mcp.rs` | M1-26 |
| 399 | `Services/ClaudeConfigLoaderMCPTests.swift:45` | `decodeClaudeConfig_localScopeServers` | `crates/heimdall-core/src/config/read/mcp.rs` | M1-26 |
| 400 | `Services/ClaudeConfigLoaderMCPTests.swift:75` | `decodeMCPProjectConfig` | `crates/heimdall-core/src/config/read/mcp.rs` | M1-26 |
| 401 | `Services/ClaudeConfigLoaderMCPTests.swift:97` | `decodeClaudeConfig_emptyObject` | `crates/heimdall-core/src/config/read/mcp.rs` | M1-26 |
| 402 | `Services/ClaudeConfigLoaderMCPTests.swift:106` | `decodeMCPServerDefinition_minimalFields` | `crates/heimdall-core/src/config/read/mcp.rs` | M1-26 |
| 403 | `Services/ClaudeConfigLoaderMCPTests.swift:121` | `decodeMCPServerDefinition_allFields` | `crates/heimdall-core/src/config/read/mcp.rs` | M1-26 |
| 404 | `Services/ClaudeConfigLoaderMCPTests.swift:145` | `mcpServerModel_newFields` | `crates/heimdall-core/src/config/read/mcp.rs` | M1-26 |
| 405 | `Services/ClaudeConfigLoaderMCPTests.swift:169` | `mcpServerModel_httpType` | `crates/heimdall-core/src/config/read/mcp.rs` | M1-26 |
| 406 | `Services/DebugLogLoaderTests.swift:34` | `parse_emptyString_returnsEmpty` | `crates/heimdall-core/src/data/debug_log.rs` | M1-17 |
| 407 | `Services/DebugLogLoaderTests.swift:41` | `parse_singleDebugLine` | `crates/heimdall-core/src/data/debug_log.rs` | M1-17 |
| 408 | `Services/DebugLogLoaderTests.swift:52` | `parse_errorLevel` | `crates/heimdall-core/src/data/debug_log.rs` | M1-17 |
| 409 | `Services/DebugLogLoaderTests.swift:62` | `parse_warnLevel` | `crates/heimdall-core/src/data/debug_log.rs` | M1-17 |
| 410 | `Services/DebugLogLoaderTests.swift:72` | `parse_multipleLines` | `crates/heimdall-core/src/data/debug_log.rs` | M1-17 |
| 411 | `Services/DebugLogLoaderTests.swift:90` | `parse_skipsEmptyLines` | `crates/heimdall-core/src/data/debug_log.rs` | M1-17 |
| 412 | `Services/DebugLogLoaderTests.swift:102` | `parse_skipsInvalidLines` | `crates/heimdall-core/src/data/debug_log.rs` | M1-17 |
| 413 | `Services/DebugLogLoaderTests.swift:114` | `parse_messageWithBrackets` | `crates/heimdall-core/src/data/debug_log.rs` | M1-17 |
| 414 | `Services/DebugLogLoaderTests.swift:123` | `parse_timestamp_isParsedCorrectly` | `crates/heimdall-core/src/data/debug_log.rs` | M1-17 |
| 415 | `Services/DebugLogLoaderTests.swift:143` | `parse_unknownLevel_defaultsToDebug` | `crates/heimdall-core/src/data/debug_log.rs` | M1-17 |
| 416 | `Services/DebugLogLoaderTests.swift:155` | `loadEntries_nonExistentSession_returnsEmpty` | `crates/heimdall-core/src/data/debug_log.rs` | M1-17 |
| 417 | `Services/DebugLogLoaderTests.swift:164` | `loadEntries_validFile_returnsEntries` | `crates/heimdall-core/src/data/debug_log.rs` | M1-17 |
| 418 | `Services/DebugLogLoaderTests.swift:189` | `hasLog_noFile_returnsFalse` | `crates/heimdall-core/src/data/debug_log.rs` | M1-17 |
| 419 | `Services/DebugLogLoaderTests.swift:197` | `hasLog_fileExists_returnsTrue` | `crates/heimdall-core/src/data/debug_log.rs` | M1-17 |
| 420 | `Services/DebugLogLoaderTests.swift:214` | `allSessionIds_nonExistentPath_returnsEmpty` | `crates/heimdall-core/src/data/debug_log.rs` | M1-17 |
| 421 | `Services/DebugLogLoaderTests.swift:222` | `allSessionIds_returnsFilenames` | `crates/heimdall-core/src/data/debug_log.rs` | M1-17 |
| 422 | `Services/DebugLogLoaderTests.swift:240` | `loadEntries_paginated_firstPage` | `crates/heimdall-core/src/data/debug_log.rs` | M1-17 |
| 423 | `Services/DebugLogLoaderTests.swift:258` | `loadEntries_paginated_secondPage` | `crates/heimdall-core/src/data/debug_log.rs` | M1-17 |
| 424 | `Services/DebugLogLoaderTests.swift:276` | `loadEntries_paginated_lastPage_partialResults` | `crates/heimdall-core/src/data/debug_log.rs` | M1-17 |
| 425 | `Services/DebugLogLoaderTests.swift:294` | `loadEntries_paginated_offsetBeyondEnd_returnsEmpty` | `crates/heimdall-core/src/data/debug_log.rs` | M1-17 |
| 426 | `Services/DebugLogLoaderTests.swift:308` | `loadEntries_paginated_nonExistentSession` | `crates/heimdall-core/src/data/debug_log.rs` | M1-17 |
| 427 | `Services/DebugLogLoaderTests.swift:320` | `parsePaged_returnsCorrectTotalCount` | `crates/heimdall-core/src/data/debug_log.rs` | M1-17 |
| 428 | `Services/FacetsLoaderTests.swift:46` | `loadFacets_nonExistentPath_returnsNil` | `crates/heimdall-core/src/data/facets.rs` | M1-18 |
| 429 | `Services/FacetsLoaderTests.swift:53` | `loadFacets_existingFile_returnsFacets` | `crates/heimdall-core/src/data/facets.rs` | M1-18 |
| 430 | `Services/FacetsLoaderTests.swift:69` | `loadFacets_missingFile_returnsNil` | `crates/heimdall-core/src/data/facets.rs` | M1-18 |
| 431 | `Services/FacetsLoaderTests.swift:81` | `loadFacets_invalidJSON_returnsNil` | `crates/heimdall-core/src/data/facets.rs` | M1-18 |
| 432 | `Services/FacetsLoaderTests.swift:96` | `loadAllFacets_nonExistentPath_returnsEmpty` | `crates/heimdall-core/src/data/facets.rs` | M1-18 |
| 433 | `Services/FacetsLoaderTests.swift:103` | `loadAllFacets_emptyDirectory_returnsEmpty` | `crates/heimdall-core/src/data/facets.rs` | M1-18 |
| 434 | `Services/FacetsLoaderTests.swift:113` | `loadAllFacets_multipleFiles_returnsAll` | `crates/heimdall-core/src/data/facets.rs` | M1-18 |
| 435 | `Services/FacetsLoaderTests.swift:131` | `loadAllFacets_skipsInvalidFiles` | `crates/heimdall-core/src/data/facets.rs` | M1-18 |
| 436 | `Services/FacetsLoaderTests.swift:147` | `loadAllFacets_skipsNonJSONFiles` | `crates/heimdall-core/src/data/facets.rs` | M1-18 |
| 437 | `Services/FacetsLoaderTests.swift:162` | `loadAllFacets_keyedBySessionId` | `crates/heimdall-core/src/data/facets.rs` | M1-18 |
| 438 | `Services/FileHistoryLoaderTests.swift:30` | `load_includesDeltaVersionsMissingFromSnapshots` | `crates/heimdall-core/src/data/file_history.rs` | M1-19 |
| 439 | `Services/FileHistoryLoaderTests.swift:61` | `load_deltaOnlySession_stillReturnsFiles` | `crates/heimdall-core/src/data/file_history.rs` | M1-19 |
| 440 | `Services/FileHistoryLoaderTests.swift:82` | `sameFile_treatsRelativeAsSuffixOfAbsolute` | `crates/heimdall-core/src/data/file_history.rs` | M1-19 |
| 441 | `Services/FileHistoryLoaderTests.swift:90` | `load_mergesRelativeAndAbsolutePathsForSameFile` | `crates/heimdall-core/src/data/file_history.rs` | M1-19 |
| 442 | `Services/FileHistoryLoaderTests.swift:111` | `loadFileContent_readsBackup` | `crates/heimdall-core/src/data/file_history.rs` | M1-19 |
| 443 | `Services/FrontmatterParserTests.swift:8` | `parse_noFrontmatter_returnsOriginalBody` | `crates/heimdall-core/src/config/read/frontmatter.rs` | M1-23 |
| 444 | `Services/FrontmatterParserTests.swift:18` | `parse_unclosedFrontmatter_returnsOriginalBody` | `crates/heimdall-core/src/config/read/frontmatter.rs` | M1-23 |
| 445 | `Services/FrontmatterParserTests.swift:28` | `parse_wellFormed_extractsMetadata` | `crates/heimdall-core/src/config/read/frontmatter.rs` | M1-23 |
| 446 | `Services/FrontmatterParserTests.swift:43` | `parse_colonInValue_preservesFullValue` | `crates/heimdall-core/src/config/read/frontmatter.rs` | M1-23 |
| 447 | `Services/FrontmatterParserTests.swift:57` | `parse_emptyLinesInYAML_skipped` | `crates/heimdall-core/src/config/read/frontmatter.rs` | M1-23 |
| 448 | `Services/FrontmatterParserTests.swift:75` | `parse_bodyTrimmed_whitespaceRemoved` | `crates/heimdall-core/src/config/read/frontmatter.rs` | M1-23 |
| 449 | `Services/FrontmatterParserTests.swift:91` | `parse_multipleKeys_allExtracted` | `crates/heimdall-core/src/config/read/frontmatter.rs` | M1-23 |
| 450 | `Services/FrontmatterParserTests.swift:110` | `parse_emptyBody_afterFrontmatter` | `crates/heimdall-core/src/config/read/frontmatter.rs` | M1-23 |
| 451 | `Services/FrontmatterParserTests.swift:124` | `parse_foldedBlockScalar_joinsWithSpaces` | `crates/heimdall-core/src/config/read/frontmatter.rs` | M1-23 |
| 452 | `Services/FrontmatterParserTests.swift:135` | `parse_literalBlockScalar_joinsWithNewlines` | `crates/heimdall-core/src/config/read/frontmatter.rs` | M1-23 |
| 453 | `Services/FrontmatterParserTests.swift:145` | `parse_blockScalarFollowedByKey_bothExtracted` | `crates/heimdall-core/src/config/read/frontmatter.rs` | M1-23 |
| 454 | `Services/FrontmatterParserTests.swift:155` | `parse_blockScalarAtEnd_extracted` | `crates/heimdall-core/src/config/read/frontmatter.rs` | M1-23 |
| 455 | `Services/HistoryLoaderTests.swift:25` | `loadAll_nonExistentFile_returnsEmpty` | `crates/heimdall-core/src/data/history.rs` | M1-15 |
| 456 | `Services/HistoryLoaderTests.swift:32` | `loadAll_emptyFile_returnsEmpty` | `crates/heimdall-core/src/data/history.rs` | M1-15 |
| 457 | `Services/HistoryLoaderTests.swift:42` | `loadAll_singleEntry_parsesCorrectly` | `crates/heimdall-core/src/data/history.rs` | M1-15 |
| 458 | `Services/HistoryLoaderTests.swift:58` | `loadAll_multipleEntries_sortedByTimestampDescending` | `crates/heimdall-core/src/data/history.rs` | M1-15 |
| 459 | `Services/HistoryLoaderTests.swift:76` | `loadAll_skipsInvalidLines` | `crates/heimdall-core/src/data/history.rs` | M1-15 |
| 460 | `Services/HistoryLoaderTests.swift:91` | `loadAll_handlesPastedContents` | `crates/heimdall-core/src/data/history.rs` | M1-15 |
| 461 | `Services/HistoryLoaderTests.swift:105` | `loadAll_handlesNullPastedContents` | `crates/heimdall-core/src/data/history.rs` | M1-15 |
| 462 | `Services/HistoryLoaderTests.swift:119` | `loadAll_timestampConvertedFromMilliseconds` | `crates/heimdall-core/src/data/history.rs` | M1-15 |
| 463 | `Services/HistoryLoaderTests.swift:139` | `entryCount_nonExistentFile_returnsZero` | `crates/heimdall-core/src/data/history.rs` | M1-15 |
| 464 | `Services/HistoryLoaderTests.swift:145` | `entryCount_matchesLineCount` | `crates/heimdall-core/src/data/history.rs` | M1-15 |
| 465 | `Services/HistoryLoaderTests.swift:161` | `uniqueProjects_returnsDistinctPaths` | `crates/heimdall-core/src/data/history.rs` | M1-15 |
| 466 | `Services/HistoryLoaderTests.swift:179` | `uniqueProjects_nonExistentFile_returnsEmpty` | `crates/heimdall-core/src/data/history.rs` | M1-15 |
| 467 | `Services/MCPServerStatusCheckerTests.swift:37` | `authCacheDetection` — Server in auth cache returns needsAuth status | `crates/heimdall-core/src/mcp_status.rs` | M1-28 |
| 468 | `Services/MCPServerStatusCheckerTests.swift:47` | `serverNotInAuthCache` — Server not in auth cache is not marked needsAuth | `crates/heimdall-core/src/mcp_status.rs` | M1-28 |
| 469 | `Services/MCPServerStatusCheckerTests.swift:59` | `httpServerConnected` — HTTP server with valid URL shows connected | `crates/heimdall-core/src/mcp_status.rs` | M1-28 |
| 470 | `Services/MCPServerStatusCheckerTests.swift:72` | `httpServerNoURL` — HTTP server with no URL shows unreachable | `crates/heimdall-core/src/mcp_status.rs` | M1-28 |
| 471 | `Services/MCPServerStatusCheckerTests.swift:81` | `sseServerConnected` — SSE server with valid URL shows connected | `crates/heimdall-core/src/mcp_status.rs` | M1-28 |
| 472 | `Services/MCPServerStatusCheckerTests.swift:94` | `urlInferredHTTP` — Server with url field inferred as HTTP | `crates/heimdall-core/src/mcp_status.rs` | M1-28 |
| 473 | `Services/MCPServerStatusCheckerTests.swift:108` | `stdioNoCommand` — STDIO server with no command returns unknown | `crates/heimdall-core/src/mcp_status.rs` | M1-28 |
| 474 | `Services/MCPServerStatusCheckerTests.swift:119` | `noTransportInfo` — Server with no type/command/url returns unknown | `crates/heimdall-core/src/mcp_status.rs` | M1-28 |
| 475 | `Services/MCPServerStatusCheckerTests.swift:130` | `bulkResolution` — resolveStatuses updates all servers | `crates/heimdall-core/src/mcp_status.rs` | M1-28 |
| 476 | `Services/MCPServerStatusCheckerTests.swift:163` | `authCachePrecedence` — Auth cache takes precedence over HTTP connected | `crates/heimdall-core/src/mcp_status.rs` | M1-28 |
| 477 | `Services/MCPServerStatusCheckerTests.swift:177` | `authCachePrecedenceStdio` — Auth cache takes precedence over STDIO | `crates/heimdall-core/src/mcp_status.rs` | M1-28 |
| 478 | `Services/MCPServerStatusCheckerTests.swift:194` | `cloudIntegrationPreservesStatus` — Cloud integration server preserves its pre-set status | `crates/heimdall-core/src/mcp_status.rs` | M1-28 |
| 479 | `Services/MCPServerStatusCheckerTests.swift:207` | `cloudIntegrationIgnoresAuthCache` — Cloud integration server ignores auth cache lookup | `crates/heimdall-core/src/mcp_status.rs` | M1-28 |
| 480 | `Services/MCPServerStatusCheckerTests.swift:223` | `pluginServerNoProcess` — Plugin server with no running process returns unknown | `crates/heimdall-core/src/mcp_status.rs` | M1-28 |
| 481 | `Services/MemoryLoaderTests.swift:9` | `loadMemoryFiles_returnsEmptyForMissingDirectory` | `crates/heimdall-core/src/config/read/memory.rs` | M1-27 |
| 482 | `Services/MemoryLoaderTests.swift:17` | `loadMemoryFiles_loadsMarkdownFiles` | `crates/heimdall-core/src/config/read/memory.rs` | M1-27 |
| 483 | `Services/MemoryLoaderTests.swift:46` | `loadMemoryFiles_memoryMdSortedFirst` | `crates/heimdall-core/src/config/read/memory.rs` | M1-27 |
| 484 | `Services/MemoryLoaderTests.swift:75` | `loadMemoryFiles_setsProjectID` | `crates/heimdall-core/src/config/read/memory.rs` | M1-27 |
| 485 | `Services/MemoryLoaderTests.swift:93` | `loadMemoryFiles_customFolder_loadsMarkdownWithSourceLabel` | `crates/heimdall-core/src/config/read/memory.rs` | M1-27 |
| 486 | `Services/MemoryLoaderTests.swift:112` | `customMemoryFileCount_countsOnlyMarkdown` | `crates/heimdall-core/src/config/read/memory.rs` | M1-27 |
| 487 | `Services/MemoryLoaderTests.swift:123` | `loadMemoryFiles_customFolder_missing_returnsEmpty` | `crates/heimdall-core/src/config/read/memory.rs` | M1-27 |
| 488 | `Services/SessionLoaderTests.swift:89` | `discoverProjects_nonExistentPath_returnsEmpty` | `crates/heimdall-core/src/projects/discovery.rs` | M1-14 |
| 489 | `Services/SessionLoaderTests.swift:96` | `discoverProjects_emptyDirectory_returnsEmpty` | `crates/heimdall-core/src/projects/discovery.rs` | M1-14 |
| 490 | `Services/SessionLoaderTests.swift:106` | `discoverProjects_withSubdirectories_returnsProjects` | `crates/heimdall-core/src/projects/discovery.rs` | M1-14 |
| 491 | `Services/SessionLoaderTests.swift:129` | `discoverProjects_skipsFiles_onlyIncludesDirectories` | `crates/heimdall-core/src/projects/discovery.rs` | M1-14 |
| 492 | `Services/SessionLoaderTests.swift:151` | `discoverProjects_withJSONLFiles_parsesSessions` | `crates/heimdall-core/src/projects/discovery.rs` | M1-14 |
| 493 | `Services/SessionLoaderTests.swift:177` | `discoverProjects_fallbackPath_loadsAgentFiles` | `crates/heimdall-core/src/projects/subagents.rs` | M1-14 |
| 494 | `Services/SessionLoaderTests.swift:208` | `discoverProjects_sessionsSortedDescending` | `crates/heimdall-core/src/projects/discovery.rs` | M1-14 |
| 495 | `Services/SessionLoaderTests.swift:234` | `discoverProjects_usesOriginalPathFromIndex` | `crates/heimdall-core/src/projects/sessions_index.rs` | M1-14 |
| 496 | `Services/SessionLoaderTests.swift:275` | `discoverProjects_withMissingIndex_usesFallback` | `crates/heimdall-core/src/projects/sessions_index.rs` | M1-14 |
| 497 | `Services/SessionLoaderTests.swift:293` | `discoverProjects_emptyJSONLFile_producesNoSession` | `crates/heimdall-core/src/projects/discovery.rs` | M1-14 |
| 498 | `Services/SessionLoaderTests.swift:312` | `discoverProjects_indexPath_includesSidechainEntries` | `crates/heimdall-core/src/projects/sessions_index.rs` | M1-14 |
| 499 | `Services/SessionLoaderTests.swift:375` | `discoverProjects_indexPath_loadsAllSessions` | `crates/heimdall-core/src/projects/sessions_index.rs` | M1-14 |
| 500 | `Services/SessionLoaderTests.swift:407` | `discoverProjects_fallbackPath_usesHeaderParse` | `crates/heimdall-core/src/projects/discovery.rs` | M1-14 |
| 501 | `Services/SessionLoaderTests.swift:431` | `discoverProjects_indexPath_passesFirstPromptToSession` | `crates/heimdall-core/src/projects/sessions_index.rs` | M1-14 |
| 502 | `Services/SessionLoaderTests.swift:466` | `discoverProjects_indexPath_nilFirstPromptWhenMissing` | `crates/heimdall-core/src/projects/sessions_index.rs` | M1-14 |
| 503 | `Services/SessionLoaderTests.swift:500` | `discoverProjects_fallbackPath_firstPromptIsNil` | `crates/heimdall-core/src/projects/discovery.rs` | M1-14 |
| 504 | `Services/SettingsWriterMCPTests.swift:36` | `catalog_entriesAreValid` | `crates/heimdall-core/src/models/catalog/mcp_catalog.rs` | M1-10 |
| 505 | `Services/SettingsWriterMCPTests.swift:49` | `catalog_searchByCategory` | `crates/heimdall-core/src/models/catalog/mcp_catalog.rs` | M1-10 |
| 506 | `Services/SettingsWriterMCPTests.swift:60` | `envKey_defaultValues` | `crates/heimdall-core/src/models/catalog/mcp_catalog.rs` | M1-10 |
| 507 | `Services/SettingsWriterMCPTests.swift:72` | `envKey_customValues` | `crates/heimdall-core/src/models/catalog/mcp_catalog.rs` | M1-10 |
| 508 | `Services/StatsComputerTests.swift:7` | `compute_aggregatesSessionFromTranscript` | `crates/heimdall-core/src/data/stats.rs` | M1-20 |
| 509 | `Services/StatsComputerTests.swift:53` | `compute_noSessions_returnsNil` | `crates/heimdall-core/src/data/stats.rs` | M1-20 |
| 510 | `Services/StatsComputerTests.swift:63` | `compute_missingPath_returnsNil` | `crates/heimdall-core/src/data/stats.rs` | M1-20 |
| 511 | `Services/StatsComputerTests.swift:68` | `compute_includesTokensOnLaterDays` | `crates/heimdall-core/src/data/stats.rs` | M1-20 |
| 512 | `Services/StatsComputerTests.swift:99` | `compute_acceptsFloatingPointTokenCounts` | `crates/heimdall-core/src/data/stats.rs` | M1-20 |
| 513 | `Services/StatsComputerTests.swift:125` | `intValue_readsNSNumberAndIgnoresBool` | `crates/heimdall-core/src/data/stats.rs` | M1-20 |
| 514 | `Services/TodoLoaderTests.swift:27` | `loadTodos_nonExistentPath_returnsEmpty` | `crates/heimdall-core/src/data/todos.rs` | M1-16 |
| 515 | `Services/TodoLoaderTests.swift:34` | `loadTodos_emptyDirectory_returnsEmpty` | `crates/heimdall-core/src/data/todos.rs` | M1-16 |
| 516 | `Services/TodoLoaderTests.swift:44` | `loadTodos_matchesAgentFile` | `crates/heimdall-core/src/data/todos.rs` | M1-16 |
| 517 | `Services/TodoLoaderTests.swift:62` | `loadTodos_matchesExactSessionIdFile` | `crates/heimdall-core/src/data/todos.rs` | M1-16 |
| 518 | `Services/TodoLoaderTests.swift:79` | `loadTodos_mergesMultipleAgentFiles` | `crates/heimdall-core/src/data/todos.rs` | M1-16 |
| 519 | `Services/TodoLoaderTests.swift:102` | `loadTodos_doesNotMatchUnrelatedFiles` | `crates/heimdall-core/src/data/todos.rs` | M1-16 |
| 520 | `Services/TodoLoaderTests.swift:117` | `loadTodos_handlesEmptyArrayFile` | `crates/heimdall-core/src/data/todos.rs` | M1-16 |
| 521 | `Services/TodoLoaderTests.swift:129` | `loadTodos_skipsInvalidJSON` | `crates/heimdall-core/src/data/todos.rs` | M1-16 |
| 522 | `Services/TodoLoaderTests.swift:149` | `loadTodos_uuidSessionId_matchesAgentFile` | `crates/heimdall-core/src/data/todos.rs` | M1-16 |
| 523 | `Services/TodoLoaderTests.swift:173` | `loadAllTodos_nonExistentPath_returnsEmpty` | `crates/heimdall-core/src/data/todos.rs` | M1-16 |
| 524 | `Services/TodoLoaderTests.swift:180` | `loadAllTodos_groupsBySessionId` | `crates/heimdall-core/src/data/todos.rs` | M1-16 |
| 525 | `Services/TodoLoaderTests.swift:201` | `loadAllTodos_mergesAgentFilesUnderSameSession` | `crates/heimdall-core/src/data/todos.rs` | M1-16 |
| 526 | `Services/TodoLoaderTests.swift:221` | `loadAllTodos_skipsEmptyArrayFiles` | `crates/heimdall-core/src/data/todos.rs` | M1-16 |
| 527 | `Services/TodoLoaderTests.swift:241` | `deleteTodos_removesMatchingFiles` | `crates/heimdall-core/src/projects/delete.rs` | M3-43 |
| 528 | `Services/TodoLoaderTests.swift:262` | `deleteTodos_leavesUnrelatedFiles` | `crates/heimdall-core/src/projects/delete.rs` | M3-43 |
| 529 | `Services/TodoLoaderTests.swift:283` | `deleteTodos_nonExistentPath_doesNotCrash` | `crates/heimdall-core/src/projects/delete.rs` | M3-43 |
| 530 | `Services/TranscriptParserTests.swift:72` | `parse_emptyFile_returnsNil` | `crates/heimdall-core/src/transcript/full.rs` | M1-12 |
| 531 | `Services/TranscriptParserTests.swift:81` | `parse_onlySkippedRecordTypes_returnsNil` | `crates/heimdall-core/src/transcript/full.rs` | M1-12 |
| 532 | `Services/TranscriptParserTests.swift:94` | `parse_malformedLines_skipsGracefully` | `crates/heimdall-core/src/transcript/full.rs` | M1-12 |
| 533 | `Services/TranscriptParserTests.swift:109` | `parse_sidechainRecords_skipped` | `crates/heimdall-core/src/transcript/full.rs` | M1-12 |
| 534 | `Services/TranscriptParserTests.swift:126` | `parse_userStringContent_createsTextBlock` | `crates/heimdall-core/src/transcript/full.rs` | M1-12 |
| 535 | `Services/TranscriptParserTests.swift:139` | `parse_userToolResult_createsToolResultBlock` | `crates/heimdall-core/src/transcript/full.rs` | M1-12 |
| 536 | `Services/TranscriptParserTests.swift:165` | `parse_toolResult_stringContent_normalized` | `crates/heimdall-core/src/transcript/full.rs` | M1-12 |
| 537 | `Services/TranscriptParserTests.swift:186` | `parse_toolResult_arrayContent_extractsTextOnly` | `crates/heimdall-core/src/transcript/full.rs` | M1-12 |
| 538 | `Services/TranscriptParserTests.swift:211` | `parse_toolResult_isError_flagged` | `crates/heimdall-core/src/transcript/full.rs` | M1-12 |
| 539 | `Services/TranscriptParserTests.swift:235` | `parse_assistantTextBlock` | `crates/heimdall-core/src/transcript/full.rs` | M1-12 |
| 540 | `Services/TranscriptParserTests.swift:249` | `parse_assistantThinkingBlock` | `crates/heimdall-core/src/transcript/full.rs` | M1-12 |
| 541 | `Services/TranscriptParserTests.swift:260` | `parse_assistantToolUseBlock` | `crates/heimdall-core/src/transcript/full.rs` | M1-12 |
| 542 | `Services/TranscriptParserTests.swift:284` | `parse_toolUse_nonStringInputsStringified` | `crates/heimdall-core/src/transcript/full.rs` | M1-12 |
| 543 | `Services/TranscriptParserTests.swift:314` | `parse_syntheticModel_skipped` | `crates/heimdall-core/src/transcript/full.rs` | M1-12 |
| 544 | `Services/TranscriptParserTests.swift:329` | `parse_multipleRecordsSameId_grouped` | `crates/heimdall-core/src/transcript/full.rs` | M1-12 |
| 545 | `Services/TranscriptParserTests.swift:358` | `parse_usage_deduplicatedPerGroup` | `crates/heimdall-core/src/transcript/full.rs` | M1-12 |
| 546 | `Services/TranscriptParserTests.swift:384` | `parse_startedAt_usesEarliestTimestamp` | `crates/heimdall-core/src/transcript/full.rs` | M1-12 |
| 547 | `Services/TranscriptParserTests.swift:402` | `parse_model_fromFirstAssistantGroup` | `crates/heimdall-core/src/transcript/full.rs` | M1-12 |
| 548 | `Services/TranscriptParserTests.swift:425` | `parse_totalTokens_summedAcrossGroups` | `crates/heimdall-core/src/transcript/full.rs` | M1-12 |
| 549 | `Services/TranscriptParserTests.swift:450` | `parse_interleaved_userAndAssistant_preservesOrder` | `crates/heimdall-core/src/transcript/full.rs` | M1-12 |
| 550 | `Services/TranscriptParserTests.swift:482` | `parseSummary_emptyFile_returnsNil` | `crates/heimdall-core/src/transcript/summary.rs` | M1-13 |
| 551 | `Services/TranscriptParserTests.swift:491` | `parseSummary_extractsTitle` | `crates/heimdall-core/src/transcript/summary.rs` | M1-13 |
| 552 | `Services/TranscriptParserTests.swift:507` | `parseSummary_extractsTurnCount` | `crates/heimdall-core/src/transcript/summary.rs` | M1-13 |
| 553 | `Services/TranscriptParserTests.swift:523` | `parseSummary_extractsModel` | `crates/heimdall-core/src/transcript/summary.rs` | M1-13 |
| 554 | `Services/TranscriptParserTests.swift:536` | `parseSummary_sumsTokensPerGroup` | `crates/heimdall-core/src/transcript/summary.rs` | M1-13 |
| 555 | `Services/TranscriptParserTests.swift:569` | `parseSummary_skipsSidechainRecords` | `crates/heimdall-core/src/transcript/summary.rs` | M1-13 |
| 556 | `Services/TranscriptParserTests.swift:583` | `parseSummary_skipsSyntheticAssistant` | `crates/heimdall-core/src/transcript/summary.rs` | M1-13 |
| 557 | `Services/TranscriptParserTests.swift:595` | `parseSummary_usesEarliestTimestamp` | `crates/heimdall-core/src/transcript/summary.rs` | M1-13 |
| 558 | `Services/TranscriptParserTests.swift:614` | `parseSummary_searchableText_indexesConversationContent` | `crates/heimdall-core/src/transcript/summary.rs` | M1-13 |
| 559 | `Services/TranscriptParserTests.swift:641` | `parseSummary_searchableText_excludesSyntheticAndSidechain` | `crates/heimdall-core/src/transcript/summary.rs` | M1-13 |
| 560 | `Services/TranscriptParserTests.swift:656` | `parseSummary_searchableText_respectsCap` | `crates/heimdall-core/src/transcript/summary.rs` | M1-13 |
| 561 | `Services/TranscriptParserTests.swift:669` | `parse_searchableText_indexesContent` | `crates/heimdall-core/src/transcript/searchable.rs` | M1-12 |
| 562 | `Services/UpdateCheckerTests.swift:8` | `isNewer_higherMajor_returnsTrue` | DROP: replaced by the Tauri updater plugin (port-plan P-14) | — |
| 563 | `Services/UpdateCheckerTests.swift:13` | `isNewer_higherMinor_returnsTrue` | DROP: replaced by the Tauri updater plugin (port-plan P-14) | — |
| 564 | `Services/UpdateCheckerTests.swift:18` | `isNewer_higherPatch_returnsTrue` | DROP: replaced by the Tauri updater plugin (port-plan P-14) | — |
| 565 | `Services/UpdateCheckerTests.swift:23` | `isNewer_sameVersion_returnsFalse` | DROP: replaced by the Tauri updater plugin (port-plan P-14) | — |
| 566 | `Services/UpdateCheckerTests.swift:28` | `isNewer_lowerVersion_returnsFalse` | DROP: replaced by the Tauri updater plugin (port-plan P-14) | — |
| 567 | `Services/UpdateCheckerTests.swift:33` | `isNewer_lowerMinor_returnsFalse` | DROP: replaced by the Tauri updater plugin (port-plan P-14) | — |
| 568 | `Services/UpdateCheckerTests.swift:38` | `isNewer_differentLengths_handledCorrectly` | DROP: replaced by the Tauri updater plugin (port-plan P-14) | — |
| 569 | `Services/UpdateCheckerTests.swift:44` | `isNewer_singleComponent_works` | DROP: replaced by the Tauri updater plugin (port-plan P-14) | — |
| 570 | `Services/UpdateCheckerTests.swift:50` | `isNewer_emptyStrings_returnsFalse` | DROP: replaced by the Tauri updater plugin (port-plan P-14) | — |
| 571 | `Services/UpdateCheckerTests.swift:56` | `isNewer_largeVersionNumbers_works` | DROP: replaced by the Tauri updater plugin (port-plan P-14) | — |
| 572 | `Utilities/HighlightedTextTests.swift:6` | `matches_caseInsensitiveSubstring` | `crates/heimdall-core/src/search/matcher.rs` | M1-29 |
| 573 | `Utilities/HighlightedTextTests.swift:12` | `rejects_scatteredSubsequence_thatFuzzyWouldMatch` | `crates/heimdall-core/src/search/matcher.rs` | M1-29 |
| 574 | `Utilities/HighlightedTextTests.swift:20` | `ranksPrefixOverBoundaryOverMid` | `crates/heimdall-core/src/search/matcher.rs` | M1-29 |
| 575 | `Utilities/HighlightedTextTests.swift:29` | `emptyQuery_returnsZero` | `crates/heimdall-core/src/search/matcher.rs` | M1-29 |
| 576 | `Utilities/JSONBeautifierTests.swift:9` | `beautify_validObject_prettyPrints` | `crates/heimdall-core/src/transcript/tool_input.rs` | M1-12 |
| 577 | `Utilities/JSONBeautifierTests.swift:17` | `beautify_validArray_prettyPrints` | `crates/heimdall-core/src/transcript/tool_input.rs` | M1-12 |
| 578 | `Utilities/JSONBeautifierTests.swift:29` | `beautify_nonJSON_returnsOriginal` | `crates/heimdall-core/src/transcript/tool_input.rs` | M1-12 |
| 579 | `Utilities/JSONBeautifierTests.swift:38` | `beautify_doublyEncoded_decodesAndFormats` | `crates/heimdall-core/src/transcript/tool_input.rs` | M1-12 |
| 580 | `Utilities/JSONBeautifierTests.swift:49` | `beautify_emptyString_returnsOriginal` | `crates/heimdall-core/src/transcript/tool_input.rs` | M1-12 |
| 581 | `Utilities/JSONBeautifierTests.swift:55` | `beautify_whitespaceOnly_returnsOriginal` | `crates/heimdall-core/src/transcript/tool_input.rs` | M1-12 |
| 582 | `Utilities/JSONBeautifierTests.swift:64` | `beautify_sortedKeys_verified` | `crates/heimdall-core/src/transcript/tool_input.rs` | M1-12 |
| 583 | `Utilities/LineDiffTests.swift:6` | `identicalStrings_allContext` | `crates/heimdall-core/src/diff.rs` | M1-35 |
| 584 | `Utilities/LineDiffTests.swift:13` | `emptyOld_allAdded` | `crates/heimdall-core/src/diff.rs` | M1-35 |
| 585 | `Utilities/LineDiffTests.swift:21` | `emptyNew_allRemoved` | `crates/heimdall-core/src/diff.rs` | M1-35 |
| 586 | `Utilities/LineDiffTests.swift:29` | `singleLineChange_oneRemovedOneAdded` | `crates/heimdall-core/src/diff.rs` | M1-35 |
| 587 | `Utilities/LineDiffTests.swift:38` | `insertionInMiddle_preservesContext` | `crates/heimdall-core/src/diff.rs` | M1-35 |
| 588 | `Utilities/LineDiffTests.swift:49` | `deletionInMiddle_preservesContext` | `crates/heimdall-core/src/diff.rs` | M1-35 |
| 589 | `Utilities/LineDiffTests.swift:59` | `lineNumbers_correctForContext` | `crates/heimdall-core/src/diff.rs` | M1-35 |
| 590 | `Utilities/LineDiffTests.swift:70` | `lineNumbers_nilForAddedRemoved` | `crates/heimdall-core/src/diff.rs` | M1-35 |
| 591 | `Utilities/LineDiffTests.swift:81` | `sequentialIds` | `crates/heimdall-core/src/diff.rs` | M1-35 |
| 592 | `Utilities/LineDiffTests.swift:89` | `unifiedText_formatsCorrectly` | `crates/heimdall-core/src/diff.rs` | M1-35 |
| 593 | `Utilities/LineDiffTests.swift:98` | `bothEmpty_producesEmptyContextLine` | `crates/heimdall-core/src/diff.rs` | M1-35 |
| 594 | `Utilities/LineDiffTests.swift:106` | `prefixAndSuffixPreserved_middleReplacement` | `crates/heimdall-core/src/diff.rs` | M1-35 |
| 595 | `Utilities/LineDiffTests.swift:119` | `trailingInsertion_doesNotDropContext` | `crates/heimdall-core/src/diff.rs` | M1-35 |
| 596 | `Utilities/SystemContentParserTests.swift:6` | `parse_plainText_returnsFullTextNoBlocks` | `crates/heimdall-core/src/transcript/system_content.rs` | M1-12 |
| 597 | `Utilities/SystemContentParserTests.swift:13` | `parse_singleXMLTag_extractsBlock` | `crates/heimdall-core/src/transcript/system_content.rs` | M1-12 |
| 598 | `Utilities/SystemContentParserTests.swift:26` | `parse_multipleXMLTags_extractsAllBlocks` | `crates/heimdall-core/src/transcript/system_content.rs` | M1-12 |
| 599 | `Utilities/SystemContentParserTests.swift:40` | `parse_textBetweenTags_preservesAllUserText` | `crates/heimdall-core/src/transcript/system_content.rs` | M1-12 |
| 600 | `Utilities/SystemContentParserTests.swift:54` | `parse_onlyXMLTags_returnsEmptyUserText` | `crates/heimdall-core/src/transcript/system_content.rs` | M1-12 |
| 601 | `Utilities/SystemContentParserTests.swift:63` | `parse_emptyInput_returnsEmptyResult` | `crates/heimdall-core/src/transcript/system_content.rs` | M1-12 |
| 602 | `Utilities/SystemContentParserTests.swift:70` | `parse_unmatchedTags_treatsAsPlainText` | `crates/heimdall-core/src/transcript/system_content.rs` | M1-12 |
| 603 | `Utilities/SystemContentParserTests.swift:78` | `parse_multilineTagContent_preservesContent` | `crates/heimdall-core/src/transcript/system_content.rs` | M1-12 |
| 604 | `Utilities/SystemContentParserTests.swift:94` | `parse_trimsWhitespace_inSystemContent` | `crates/heimdall-core/src/transcript/system_content.rs` | M1-12 |
| 605 | `Views/AnalyticsCSVExporterTests.swift:64` | `dailyActivity_hasCorrectHeader` | `crates/heimdall-core/src/export/analytics_csv.rs` | M3-36 |
| 606 | `Views/AnalyticsCSVExporterTests.swift:72` | `dailyActivity_hasCorrectRowCount` | `crates/heimdall-core/src/export/analytics_csv.rs` | M3-36 |
| 607 | `Views/AnalyticsCSVExporterTests.swift:81` | `dailyActivity_hasCorrectValues` | `crates/heimdall-core/src/export/analytics_csv.rs` | M3-36 |
| 608 | `Views/AnalyticsCSVExporterTests.swift:92` | `modelUsage_hasCorrectHeader` | `crates/heimdall-core/src/export/analytics_csv.rs` | M3-36 |
| 609 | `Views/AnalyticsCSVExporterTests.swift:100` | `modelUsage_sortedByCostDescending` | `crates/heimdall-core/src/export/analytics_csv.rs` | M3-36 |
| 610 | `Views/AnalyticsCSVExporterTests.swift:111` | `dailyTokens_hasModelColumnsInHeader` | `crates/heimdall-core/src/export/analytics_csv.rs` | M3-36 |
| 611 | `Views/AnalyticsCSVExporterTests.swift:120` | `dailyTokens_hasMissingModelsAsZero` | `crates/heimdall-core/src/export/analytics_csv.rs` | M3-36 |
| 612 | `Views/AnalyticsCSVExporterTests.swift:132` | `hourlyActivity_hasCorrectHeader` | `crates/heimdall-core/src/export/analytics_csv.rs` | M3-36 |
| 613 | `Views/AnalyticsCSVExporterTests.swift:140` | `hourlyActivity_sortedByHour` | `crates/heimdall-core/src/export/analytics_csv.rs` | M3-36 |
| 614 | `Views/AnalyticsCSVExporterTests.swift:152` | `emptyDailyActivity_onlyHeader` | `crates/heimdall-core/src/export/analytics_csv.rs` | M3-36 |
| 615 | `Views/AnalyticsCSVExporterTests.swift:177` | `exportType_suggestedFileNames` | `crates/heimdall-core/src/export/analytics_csv.rs` | M3-36 |
| 616 | `Views/AnalyticsViewModelTests.swift:72` | `filteredDailyActivity_allRange_returnsAll` | `src/features/analytics/store.spec.ts` | M3-31 |
| 617 | `Views/AnalyticsViewModelTests.swift:82` | `filteredDailyActivity_weekRange_filtersOldEntries` | `src/features/analytics/store.spec.ts` | M3-31 |
| 618 | `Views/AnalyticsViewModelTests.swift:103` | `tokenTimeSeriesData_flatMapsModels` | `src/features/analytics/store.spec.ts` | M3-31 |
| 619 | `Views/AnalyticsViewModelTests.swift:114` | `tokenTimeSeriesData_usesFriendlyNames` | `src/features/analytics/store.spec.ts` | M3-31 |
| 620 | `Views/AnalyticsViewModelTests.swift:128` | `costBreakdownEntries_sortedByCostDescending` | `src/features/analytics/store.spec.ts` | M3-31 |
| 621 | `Views/AnalyticsViewModelTests.swift:141` | `totalCost_sumsModelCosts` | `src/features/analytics/store.spec.ts` | M3-31 |
| 622 | `Views/AnalyticsViewModelTests.swift:150` | `totalTokens_sumsInputAndOutput` | `src/features/analytics/store.spec.ts` | M3-31 |
| 623 | `Views/AnalyticsViewModelTests.swift:161` | `totalToolCalls_sumsDailyActivity` | `src/features/analytics/store.spec.ts` | M3-31 |
| 624 | `Views/AnalyticsViewModelTests.swift:170` | `timeSavedFormatted_displaysCorrectly` | `src/features/analytics/store.spec.ts` | M3-31 |
| 625 | `Views/AnalyticsViewModelTests.swift:179` | `timeSavedFormatted_zeroMs_showsDash` | `src/features/analytics/store.spec.ts` | M3-31 |
| 626 | `Views/AnalyticsViewModelTests.swift:189` | `hasCostData_zeroCost_returnsFalse` | `src/features/analytics/store.spec.ts` | M3-31 |
| 627 | `Views/AnalyticsViewModelTests.swift:198` | `hasCostData_withCost_returnsTrue` | `src/features/analytics/store.spec.ts` | M3-31 |
| 628 | `Views/AnalyticsViewModelTests.swift:209` | `customRange_filtersCorrectly` | `src/features/analytics/store.spec.ts` | M3-31 |
| 629 | `Views/AnalyticsViewModelTests.swift:235` | `heatmapData_mapsActivityEntries` | `src/features/analytics/store.spec.ts` | M3-31 |
| 630 | `Views/AnalyticsViewModelTests.swift:251` | `filteredDailyActivity_nilStats_returnsEmpty` | `src/features/analytics/store.spec.ts` | M3-31 |
| 631 | `Views/MenuBarStateTests.swift:33` | `initialState_hasCorrectDefaults` | `crates/heimdall-core/src/data/tray.rs` | M1-21 |
| 632 | `Views/MenuBarStateTests.swift:43` | `loadRecentSessions_sortsbyMostRecent` | `crates/heimdall-core/src/data/tray.rs` | M1-21 |
| 633 | `Views/MenuBarStateTests.swift:60` | `loadRecentSessions_limitsToTen` | `crates/heimdall-core/src/data/tray.rs` | M1-21 |
| 634 | `Views/MenuBarStateTests.swift:77` | `loadRecentSessions_includesProjectInfo` | `crates/heimdall-core/src/data/tray.rs` | M1-21 |
| 635 | `Views/MenuBarStateTests.swift:89` | `loadRecentSessions_emptyProjects` | `crates/heimdall-core/src/data/tray.rs` | M1-21 |
| 636 | `Views/MenuBarStateTests.swift:100` | `filteredSessions_emptyQueryReturnsAll` | `crates/heimdall-core/src/data/tray.rs` | M1-21 |
| 637 | `Views/MenuBarStateTests.swift:111` | `filteredSessions_whitespaceQueryReturnsAll` | `crates/heimdall-core/src/data/tray.rs` | M1-21 |
