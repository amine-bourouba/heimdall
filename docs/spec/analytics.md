# Analytics, exports and the context ring — behaviour spec

Source of truth: the Swift reference app (`$SWIFT_REF` = `C:\Users\pc\Desktop\tech\poirot`, commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Every claim below cites a Swift `file:line`. Heimdall vocabulary (`StatsView`, `CsvKind`, `PngKind`, `ContextUsage`) is from `docs/ipc-contract.md` §5.

Files read for this spec:
- `Poirot/Sources/Views/Analytics/AnalyticsViewModel.swift`
- `Poirot/Sources/Views/Analytics/AnalyticsDashboardView.swift`
- `Poirot/Sources/Views/Analytics/Helpers/AnalyticsFormatters.swift`
- `Poirot/Sources/Views/Analytics/Helpers/AnalyticsColorPalette.swift`
- `Poirot/Sources/Views/Analytics/Charts/{CostBreakdownView,DailyActivityChart,HourlyActivityChart,ModelUsageChart,TokenUsageOverTimeChart,ToolCallsOverTimeChart}.swift`
- `Poirot/Sources/Views/Analytics/Components/{ContributionHeatmap,StatCard,ChartCard,AnalyticsShimmerView}.swift`
- `Poirot/Sources/Views/Analytics/Export/{AnalyticsCSVExporter,AnalyticsImageExporter,AnalyticsSnapshotView}.swift`
- `Poirot/Sources/Models/StatsCache.swift`
- `Poirot/Sources/Models/ContextUsage.swift`
- `Poirot/Sources/Models/Message.swift` (for `TokenUsage.contextTokens`)
- `Poirot/Sources/Services/StatsComputer.swift`
- `Poirot/Sources/Services/StatsCacheLoader.swift`
- `Poirot/Sources/Views/Session/ConversationContextCard.swift`
- `Poirot/Sources/Models/Usage/UsageWindow.swift` (severity thresholds referenced by the context ring)
- `Poirot/Sources/Theme/PoirotTheme.swift` (for the default palette's concrete hex values)

---

## 1. Data source and staleness rules

`StatsView = { stats: StatsCache; source: "cache" | "computed" }` (ipc-contract.md:164) mirrors the two-path loader below.

### 1.1 `StatsCacheLoader`

- Default cache path: `~/.claude/stats-cache.json` (`StatsCacheLoader.swift:5-8`).
- `load(from:)` reads and `JSONDecoder`-decodes the file; returns `nil` on any I/O or decode failure, silently (`StatsCacheLoader.swift:11-20`) — Heimdall must not swallow this silently per ADR 0005; it should surface a `skipped`/`Loaded<T>`-style signal instead.
- `loadAnalytics(cachePath:, projectsPath:)` (`StatsCacheLoader.swift:26-33`):
  1. Load the cache.
  2. Compute `latestSessionDay` = the max local-calendar day (`yyyy-MM-dd`) of `contentModificationDate` across every UUID-named `.jsonl` file under every project directory (`StatsComputer.swift:61-86`). This is a **file mtime**, not a parse of the transcript's own timestamps — a copied/touched file can mis-report staleness.
  3. If a cache exists **and** `isStale(cache, latestSessionDay)` is `false`, return the cache verbatim (`source: "cache"`).
  4. Otherwise recompute from JSONL via `StatsComputer.compute(projectsPath:)`, falling back to the (stale) cache if the recompute itself returns `nil` (e.g. permissions error) (`StatsCacheLoader.swift:32`) → `source: "computed"`.

### 1.2 `isStale` rule (exact)

`StatsCacheLoader.swift:37-51`:
```
lastActivity = max(cache.dailyActivity[].date) or ""
lastTokens   = max(cache.dailyModelTokens[].date) or ""

stale if lastActivity != "" AND lastTokens < lastActivity   // tokens lag activity within the cache itself
stale if latestSessionDay != nil AND lastTokens < latestSessionDay
stale if latestSessionDay != nil AND lastActivity < latestSessionDay
otherwise not stale
```
Comparisons are lexicographic string comparisons on `yyyy-MM-dd`, which is chronologically correct for that format. The rationale comment: Claude Code's own cache is known to freeze (`lastComputedDate` stuck, token series truncated) while sessions keep growing (`AnalyticsViewModel.swift:80-82`).

### 1.3 `StatsComputer.compute` (the local-recompute fallback)

Scans every project directory's UUID-named `.jsonl` files (main-thread sessions only) via `SessionLoader.projectDirectoryURLs` (`StatsComputer.swift:11-32`; file filter `StatsComputer.swift:25-27`). Sub-agent sidechains (`isSidechain == true`, or files not named as a bare UUID, e.g. `agent-*`) are excluded to avoid double counting (`StatsComputer.swift:8-9, 25-27, 137`). Returns `nil` when no session produced a countable record (`totalSessions == 0`, `StatsComputer.swift:211`) — this is when the dashboard shows the "No session analytics yet" empty state.

Per-line rules (`StatsComputer.swift:132-160`):
- Only `type == "user"` or `type == "assistant"` records with `isSidechain != true` count (`:134-138`).
- Assistant records with `message.model == "<synthetic>"` (Claude Code's internal calls) are skipped entirely (`:141`).
- A record only counts if its `timestamp` string parses (ISO-8601 with or without fractional seconds, `:55-57, 142-143`).
- Day bucket and hour bucket use `Calendar.current` — **local time**, not UTC (`:152-153`). See §5.2 for the mismatch with `HourlyActivityChart`'s "(UTC)" subtitle.
- Tool calls per day = count of assistant content blocks with `type == "tool_use"` (`:184-187`).
- Token usage per assistant message pulled from `message.usage`: `input_tokens`, `output_tokens`, `cache_read_input_tokens`, `cache_creation_input_tokens`, and `usage.server_tool_use.web_search_requests` (`:190-196`), read through `intValue(from:)` which accepts `NSNumber` and rejects booleans (`:90-94`, guards against `JSONSerialization` handing back a `Bool` for a numeric-looking key).
- `dailyModelTokens[day][model]` totals `input + output + cacheRead + cacheCreation` (`:206-207`) — cache-inclusive, matching `StatsCache.totalInputTokens`'s convention (§2.2).
- Session start/end = min/max message timestamp in the file; a session counts only if at least one record was seen (`:162-165`). Longest-session duration = `(end - start) * 1000` ms, wall clock (`:167-175`).
- Per-model `contextWindow` in the recomputed cache: `1_000_000` if that model's cumulative `input + cacheRead + cacheCreation` exceeds `200_000` across the whole scan, else `200_000` (`:228-230`) — a **model-granularity** version of the same 200k/1M inference `ContextUsage.from` does per-conversation (§6).
- `maxOutputTokens` is hardcoded `32000` for every model (`:241`) — a placeholder, not derived from data.
- **`costUSD` is always `0`** for every model when recomputed locally (`:236-239`); the code comment states this explicitly: "Subscription usage has no per-call dollar cost; the dashboard renders this as 'included in subscription'." This is the concrete mechanism behind the acceptance criterion "costUSD always 0 when recomputed" — it is **only** true on this fallback path. When Claude Code's own `stats-cache.json` is used directly (not stale), `costUSD` comes from Claude Code and can be non-zero for API-key users.
- `totalSpeculationTimeSavedMs` is hardcoded `0` when recomputed (`:260`) — matches the "Time Saved" card always showing "no cache data" on this path (§4).
- `lastComputedDate` of a recomputed cache = today's local date (`:251`) — so after a fallback recompute, the view model's `referenceDate` (§2) becomes "today", not a historical marker.

None of `StatsCache.ModelUsage.contextWindow` or `.maxOutputTokens` (`StatsCache.swift:40-41`) is read anywhere in the analytics dashboard or the context ring (grep confirms no reference outside their declaration/construction) — see Open questions §11.7-11.8.

---

## 2. Date ranges and the reference-date rule

`AnalyticsDateRange` (`AnalyticsViewModel.swift:6-26`): `.week` (id `"7d"`), `.month` (`"30d"`), `.quarter` (`"90d"`), `.all` (`"All"`), `.custom(start, end)` (`"Custom"`). `presets = [.week, .month, .quarter, .all]` (`:25`) — `.custom` is deliberately excluded from the preset list (see Open questions §11.1 for the picker-binding consequence).

### 2.1 Reference date (exact rule)

```swift
private var referenceDate: Date {
    if let stats { return AnalyticsFormatters.parseDate(stats.lastComputedDate) }
    return .now
}
```
(`AnalyticsViewModel.swift:93-98`) — the reference date is **the loaded stats' `lastComputedDate`** (parsed as `yyyy-MM-dd`, `AnalyticsFormatters.swift:4-13`), not wall-clock "now", whenever stats are loaded. Only falls back to `.now` when `stats` is `nil`.

### 2.2 Range → `(start, end)` (exact rule)

`AnalyticsViewModel.swift:100-117`:
```
.week:    (Calendar.current.date(byAdding: .day, value: -7,  to: ref), ref)
.month:   (Calendar.current.date(byAdding: .day, value: -30, to: ref), ref)
.quarter: (Calendar.current.date(byAdding: .day, value: -90, to: ref), ref)
.all:     nil                                          // no filtering at all
.custom(start, end): (start, end)                       // used verbatim, no normalization/clamping
```
Arithmetic uses `Calendar.current.date(byAdding:)` (calendar-aware, not raw `86400`-second math, so DST-observing calendars shift correctly). `.custom` does not enforce `start <= end`, does not clamp to available data, and is not intersected with `.all`'s bounds.

`applyCustomRange()` sets `selectedDateRange = .custom(customStartDate, customEndDate)` and closes the popover (`:119-122`). `isCustomRange` is `true` only for the `.custom` case (`:124-127`).

### 2.3 Filtering (inclusive on both ends)

```swift
var filteredDailyActivity: [StatsCache.DailyActivity] {
    guard let stats else { return [] }
    guard let range = dateRange else { return stats.dailyActivity }   // .all → everything, unfiltered
    return stats.dailyActivity.filter {
        let date = AnalyticsFormatters.parseDate($0.date)
        return date >= range.start && date <= range.end               // inclusive both ends
    }
}
```
(`AnalyticsViewModel.swift:131-138`). `filteredDailyModelTokens` is the identical pattern over `stats.dailyModelTokens` (`:140-147`).

**Not every total respects the selected range.** `costBreakdownEntries` (§3.4), `totalCost`, `totalTokens`, `totalToolCalls` (§4) and the model-usage/cost CSV exports (§7) all read `stats.modelUsage` or `stats` fields directly — they are **all-time**, independent of `selectedDateRange`. Only the daily-activity-derived series (heatmap, daily-activity chart, tool-calls chart) and the token-time-series chart are range-filtered.

---

## 3. Derived series (view-model formulas)

### 3.1 `tokenTimeSeriesData` — `TokenTimeSeriesEntry { date, model, tokens }`
```
filteredDailyModelTokens.flatMap { day in
    day.tokensByModel.map { model, tokens in
        (date: parseDate(day.date), model: StatsCache.friendlyModelName(model), tokens: tokens)
    }
}
```
(`AnalyticsViewModel.swift:151-161`). One entry per (day, model) pair; date-range filtered via `filteredDailyModelTokens`.

### 3.2 `heatmapData` — `HeatmapEntry { date, dateString, messages, sessions }`
```
filteredDailyActivity.map { day in
    (date: parseDate(day.date), dateString: day.date, messages: day.messageCount, sessions: day.sessionCount)
}
```
(`:165-174`). `dateString` is the original `yyyy-MM-dd` string, preserved (not re-derived from `date`), and used as the dictionary key for hover-anchor lookup (§5.1).

### 3.3 `costBreakdownEntries` — `CostBreakdownEntry { model, cost, inputTokens, outputTokens, cacheTokens }`
```
stats.modelUsage.map { key, value in
    (model: friendlyModelName(key), cost: value.costUSD,
     inputTokens: value.inputTokens, outputTokens: value.outputTokens,
     cacheTokens: value.cacheReadInputTokens + value.cacheCreationInputTokens)
}
.sorted { $0.cost > $1.cost }
```
(`:178-190`). All-time (`stats.modelUsage`, not filtered). Sort is by cost descending; ties are in whatever order `Dictionary.map` yields (undefined/unstable across runs since Swift dictionaries are unordered — see Open questions §11.9).

### 3.4 Summary totals
| Property | Formula | file:line | Range-filtered? |
|---|---|---|---|
| `totalCost` | `stats?.totalCostUSD ?? 0` | `:194-196` | No (all-time) |
| `hasCostData` | `totalCost > 0` | `:198-200` | — |
| `totalTokens` | `stats.totalInputTokens + stats.totalOutputTokens` | `:202-205` | No (all-time) |
| `totalToolCalls` | `stats?.totalToolCalls ?? 0` | `:207-209` | No (all-time) |
| `filteredTotalMessages` | `sum(filteredDailyActivity.messageCount)` | `:211-213` | Yes, but **unused by any view** |
| `filteredTotalSessions` | `sum(filteredDailyActivity.sessionCount)` | `:215-217` | Yes, but **unused by any view** |
| `filteredTotalToolCalls` | `sum(filteredDailyActivity.toolCallCount)` | `:219-221` | Yes, but **unused by any view** |
| `timeSavedFormatted` | `"—"` if `stats == nil`; `"—"` if `totalSpeculationTimeSavedMs == 0`; else `formatDuration(ms)` | `:223-228` | No (all-time) |
| `hasTimeSavedData` | `(stats?.totalSpeculationTimeSavedMs ?? 0) > 0` | `:230-232` | — |

`StatsCache.totalInputTokens` is **cache-inclusive**: `modelUsage.values.reduce(0) { $0 + $1.inputTokens + $1.cacheReadInputTokens + $1.cacheCreationInputTokens }` (`StatsCache.swift:78-81`), so `totalTokens` on the dashboard already counts cache reads/writes as "input". `totalOutputTokens` sums only `outputTokens` (`StatsCache.swift:73-76`). `totalToolCalls` sums `dailyActivity[].toolCallCount` across **every** day in the cache, not just the filtered range (`StatsCache.swift:88-91`).

Confirmed by grep: `filteredTotalMessages`/`filteredTotalSessions`/`filteredTotalToolCalls` have no callers anywhere in `Poirot/Sources` outside their own declaration.

---

## 4. Formatters (`AnalyticsFormatters`)

| Function | Rule | file:line |
|---|---|---|
| `parseDate(String) -> Date` | `DateFormatter` with `dateFormat = "yyyy-MM-dd"`, `Locale(identifier: "en_US_POSIX")`; falls back to `.now` if unparsable | `:4-13` |
| `formatFirstSessionDate(Date?) -> String` | `nil` → `"Unknown"`; else `DateFormatter` `.medium` date style, no time (locale-dependent, e.g. "Feb 17, 2026") | `:15-21` |
| `formatLargeNumber(Int) -> String` | `>= 1_000_000` → `"%.1fM"`; `>= 1000` → `"%.1fK"`; else plain `"\(n)"` | `:23-31` |
| `formatHour(Int) -> String` | 12-hour clock with `AM`/`PM` suffix (no space); `hour == 0` → `"12AM"`; `hour == 12` → `"12PM"`; else `hour > 12 ? hour-12 : hour` | `:33-37` |
| `formatCost(Double) -> String` | `>= 1000` → `"$%.0f"`; `>= 100` → `"$%.1f"`; else `"$%.2f"` | `:39-47` |
| `formatShortDate(Date) -> String` | `dateFormat = "MMM d"` (e.g. `"Feb 17"`) | `:49-53` |
| `formatLocalizedDate(String) -> String` | `parseDate` then `.medium` date, no time | `:55-62` |
| `dateToString(Date) -> String` | Inverse of `parseDate` (`yyyy-MM-dd`) | `:64-67` |
| `formatLocalizedDateTime(String) -> String` | Tries `ISO8601DateFormatter` with `[.withInternetDateTime, .withFractionalSeconds]` first, falls back to `parseDate`; formats `.medium` date + `.short` time | `:69-80` |
| `formatDuration(milliseconds: Int) -> String` | `totalSeconds = ms/1000; hours = totalSeconds/3600; minutes = (totalSeconds%3600)/60`; `hours > 0` → `"\(hours)h \(minutes)m"`; else `"\(minutes)m"` | `:82-90` |

`StatsCache.longestSessionFormatted` (`StatsCache.swift:62-71`) duplicates the same hours/minutes formula inline (not via `AnalyticsFormatters.formatDuration`) — same output shape, separate implementation.

`StatsCache.friendlyModelName(String) -> String` (`StatsCache.swift:104-111`), used by charts, cards and CSV exports alike:
```
contains "opus-4-6"    → "Opus 4.6"
contains "opus-4-5"    → "Opus 4.5"
contains "sonnet-4-5"  → "Sonnet 4.5"
contains "sonnet-4"    → "Sonnet 4"
contains "haiku"       → "Haiku"
else                   → the raw model id, unchanged
```
Order matters (first match wins) since `"sonnet-4-5".contains("sonnet-4")` is also true.

---

## 5. Colour palette

`AnalyticsColorPalette.all` (`AnalyticsColorPalette.swift:4-11`) is a fixed 6-slot array, in this order: `accent, blue, purple, teal, green, orange` (all `PoirotTheme.Colors.*`, theme-dependent tokens, not fixed hex). `colors(count:) = Array(all.prefix(count))` (`:13-15`) — **no wraparound**: a 7th+ series has no colour slot at all.

Default (dark, since exports force dark — see §8) hex values from `PoirotTheme.swift:53-59` (`ThemePalette.default`, `darkHex`):
| Token | Dark hex |
|---|---|
| `accent` | `#E8A642` |
| `blue` | `#0A84FF` |
| `purple` | `#AF52DE` |
| `teal` | `#30D5C8` |
| `green` | `#32D74B` |
| `orange` | `#FF9F0A` |

Three different consumers each build their own model-ordered subset of this palette by array position, and they do **not** agree on model order:
- `ModelUsageChart.modelData` is sorted by `tokens` (`input+output`) descending (`ModelUsageChart.swift:9-17`).
- `CostBreakdownView`'s `costBreakdownEntries` is sorted by `cost` descending (`AnalyticsViewModel.swift:189`).
- `TokenUsageOverTimeChart.models` is `Array(Set(data.map(\.model))).sorted()` — **alphabetical** (`TokenUsageOverTimeChart.swift:9-11`).

So the same model can render in a different colour on different charts, purely because the position in each chart's own sorted/deduplicated model list differs. See Open questions §11.3.

`ContributionHeatmap`'s cell colour is a 4-step opacity ramp of `accent` alone (not the full palette) — see §6.

---

## 6. Activity heatmap (Monday-first ISO-week grid)

Component: `ContributionHeatmap.swift`. Title "Activity Heatmap", subtitle "Daily message intensity" (`:50`). Cell size 12pt, spacing 3pt, day labels `["Mon", "", "Wed", "", "Fri", "", "Sun"]` — row 0/2/4/6 labeled, others blank (`:11-13, 54-62`).

### 6.1 Grid algorithm (exact)

`ContributionHeatmap.swift:16-43` (byte-identical duplicate for the static PNG export in `AnalyticsSnapshotView.computeGrid`, `:275-305`):
```
guard !entries.isEmpty else { return [] }                      // empty input → 0×7 grid, no explicit "no data" message

calendar = Calendar(identifier: .iso8601)                       // weekday numbering unaffected: 1=Sun..7=Sat
sorted = entries.sorted by date ascending
firstDate = sorted.first.date; lastDate = sorted.last.date
firstWeekday = calendar.component(.weekday, from: firstDate)    // 1=Sun ... 7=Sat
totalDays = calendar.dateComponents([.day], from: firstDate, to: lastDate).day ?? 0
adjustedFirstDay = (firstWeekday + 5) % 7                       // remaps to Monday=0 ... Sunday=6
totalSlots = adjustedFirstDay + totalDays + 1
weekCount = (totalSlots + 6) / 7                                 // ceil(totalSlots / 7)

result = weekCount columns × 7 rows, all nil                    // [week][dayOfWeek], dayOfWeek 0=Mon..6=Sun
lookup = { entry.dateString -> entry }                           // keyed by the ORIGINAL day-string, not recomputed

for dayOffset in 0...totalDays:
    date = firstDate + dayOffset days
    weekday = calendar.component(.weekday, from: date)
    row = (weekday + 5) % 7                                       // Monday=0 ... Sunday=6
    col = (adjustedFirstDay + dayOffset) / 7                       // integer division → week column index
    dateString = format(date, "yyyy-MM-dd", en_US_POSIX)           // recomputed, must match lookup keys exactly
    result[col][row] = lookup[dateString]                          // nil if no activity that day
```
Columns are week buckets (left = earliest week, containing whatever partial days lead up to the first Monday-aligned slot); rows are Monday(0)…Sunday(6). A day with no entry (not present in `filteredDailyActivity`, i.e. `dateString` not in `lookup`) stays `nil`, rendered as the "0 messages" colour (§6.2), visually indistinguishable from a day with exactly the minimum non-zero bucket's floor... no — `nil` and `messages == 0` both hit the same `guard messages > 0 else { return bgCard }` branch (`:127-129`), so missing-data days and zero-activity days render identically.

### 6.2 Cell colour scale

`ContributionHeatmap.swift:127-141` (and the PNG's `AnalyticsSnapshotView.heatmapColor`, `:264-273`, identical thresholds):
```
maxMessages = entries.map(\.messages).max() ?? 1
messages == 0            → PoirotTheme.Colors.bgCard
ratio = messages / maxMessages
0.00 ..< 0.25             → accent, 20% opacity
0.25 ..< 0.50             → accent, 40% opacity
0.50 ..< 0.75             → accent, 65% opacity
0.75 ... 1.00             → accent, 100% opacity
```
`maxMessages` is computed over the **currently filtered** `entries` (i.e. `viewModel.heatmapData`, which already respects `selectedDateRange`) — so the color scale re-normalizes per range selection, not against all-time max.

### 6.3 Tooltip / empty state

Hover tooltip (interactive dashboard only — the PNG's `staticHeatmap` has no hover, `:228-262`): shows `entry.dateString` then `"\(messages) messages · \(sessions) sessions"`, positioned via `anchorPreference`/`overlayPreferenceValue` 40pt above the hovered cell (`:77-123`). There is no explicit "no data" placeholder for an empty heatmap — an empty `entries` array simply renders a 0-column grid (just the day-label column) inside the `ChartCard` chrome.

---

## 7. The six charts (`Views/Analytics/Charts/`)

All six live in this exact folder (`CostBreakdownView`, `DailyActivityChart`, `HourlyActivityChart`, `ModelUsageChart`, `TokenUsageOverTimeChart`, `ToolCallsOverTimeChart`); the heatmap (`Components/ContributionHeatmap.swift`) is documented separately in §6 per the task's acceptance criteria. All six are wrapped in the shared `ChartCard` chrome: title + subtitle header, content, `Spacer(minLength: 0)`, padded `lg`, `bgCard` fill + `border` stroke (`ChartCard.swift:1-36`). None of the six charts has an explicit "no data" placeholder string — an empty series just renders empty axes/sectors inside the same card chrome.

### 7.1 Daily Activity (`DailyActivityChart.swift`)
- Type: `BarMark`. X = `parseDate(day.date)`, Y = `day.messageCount`; bar fill `accent` at 80% opacity, corner radius 2 (`:11-19`).
- Series: `viewModel.filteredDailyActivity` (range-filtered, passed in from `AnalyticsDashboardView.swift:217`).
- X axis: `.stride(by: .day, count: max(dailyActivity.count / 8, 7))` — adaptive spacing, never denser than every 7 days; labels `"MMM d"` abbreviated (`:28-36`).
- Y axis: default `AxisMarks`, dashed gridlines, `textTertiary` labels (`:37-45`).
- Selection/tooltip: `chartXSelection($selectedDate)`; a dashed `RuleMark` at the selected x; a floating tooltip box (`formatLocalizedDate(entry.date)` + `"N msgs · M sessions"`) clamped between `x=80` and `width-80`, `y=12` (`:21-25, 50-62, 72-87`).
- Height 220; plot background `bgCard` at 30% opacity.

### 7.2 Activity by Hour (`HourlyActivityChart.swift`)
- Type: `BarMark`. X = hour (0-23), Y = session-start count; gradient fill `[accent@60%, accent]`, corner radius 3 (`:12-23`).
- Series: `stats.sortedHourCounts` (`AnalyticsDashboardView.swift:221`) — **all-time**, not range-filtered (only chart in the six that ignores `selectedDateRange` entirely).
- X axis: fixed tick set `[0, 6, 12, 18, 23]`, labels via `formatHour` (`:32-43`).
- Y axis: default (`:45-53`).
- Selection: local `@State selectedHour`, `chartXSelection`; dashed rule; tooltip box `formatHour(hour)` + `"N sessions"`, clamped `x∈[70, width-70]`, `y=12` (`:25-31, 58-83`).
- Subtitle text is `"Session start times (UTC)"` (`:11`) — but the underlying hour bucketing (in `StatsComputer`, and presumably in Claude Code's own cache generator) is **not documented as UTC**; `StatsComputer` buckets with `Calendar.current.component(.hour, ...)`, i.e. local time (`StatsComputer.swift:153`). See Open questions §11.2.
- Height 200.

### 7.3 Model Usage (`ModelUsageChart.swift`)
- Type: `SectorMark` donut, `innerRadius: .ratio(0.6)`, `angularInset: 2`, corner radius 4 (`:21-29`).
- Series: `modelUsage.map { (friendlyModelName(key), value.outputTokens + value.inputTokens) }` sorted by tokens descending (`:9-17`) — **excludes cache tokens**, unlike `CostBreakdownEntry.cacheTokens`.
- Data source: `stats.modelUsage` directly (`AnalyticsDashboardView.swift:236-239`) — **all-time**, not range-filtered.
- Colour scale: `chartForegroundStyleScale(domain: modelData.map(model), range: colors(count: modelData.count))` (`:32-35`).
- Legend: bottom, coloured dot + model name only (no value) (`:36-49`).
- Selection: `chartAngleSelection($selectedAngle)`; unselected sectors dim to 70% opacity (`:29`); `findEntry(at:)` walks a cumulative-sum scan over `modelData` to map the reported angle value to an entry (`:71-80`) — `chartAngleSelection` reports the **cumulative value under the pointer** (in token units, since `angle: .value("Tokens", ...)`), not a raw degree angle.
- Center label (`chartBackground`) only appears **while a sector is selected** (`entry.model` + `formatLargeNumber(entry.tokens)`, `:50-61`) — unlike `CostBreakdownView`'s donut, which always shows a "Total" label (§7.4). No default/idle center content.
- Height 200. No hover tooltip box (unlike the line/bar charts); the only feedback is the dim/undim + center label.

### 7.4 Cost & Token Breakdown (`CostBreakdownView.swift`)
- Two mutually exclusive layouts, switched on `totalCost > 0` (`hasCostData`, `:8-20`):
  - `costAndTokenLayout` (API users): donut + token table side by side, fixed height 280 (`:24-32`).
  - `tokenOnlyLayout` (subscription users): an info banner — `"info.circle"` icon + `"Cost tracking is available for API users only"` — above the token table, auto height (`:36-48`).
- Donut: `SectorMark`, `angle: .value("Cost", entry.cost)`, same `innerRadius`/`angularInset`/corner radius as §7.3; `chartForegroundStyleScale(domain: entries.map(model), range: colors(count: entries.count))`; legend hidden (`.chartLegend(.hidden)`) in favor of a **custom** legend row below (`:52-93`).
- Center label (`chartBackground`) is **always** shown, static — `"Total"` + `formatCost(totalCost)` — with no selection interactivity on this donut at all (`:67-76`).
- Custom legend: one row per entry, coloured dot + `"\(model) \(formatCost(cost))"` (`:81-92`).
- Token table: header row `Model | Input | Output | Cache` (numeric columns right-aligned, fixed width 70) (`:100-113`); one row per entry with `formatLargeNumber` on each token count (`:117-130`).
- Series: `viewModel.costBreakdownEntries` (§3.3) — **all-time**, cost-sorted descending.
- Empty state: an empty `entries` array renders the table header with zero rows (no explicit "no data" text); if `!hasCostData` and `entries` is also empty, only the info banner + empty table show.

### 7.5 Token Usage Over Time (`TokenUsageOverTimeChart.swift`)
- Type: `AreaMark` (30% opacity) + `LineMark` (width 1.5), both `catmullRom`-interpolated, coloured by model (`:15-30`).
- Series: `viewModel.tokenTimeSeriesData` (§3.1) — **range-filtered** (via `filteredDailyModelTokens`).
- Colour scale: `chartForegroundStyleScale(domain: models, range: colors(count: models.count))` where `models` is the **alphabetically sorted** unique model list (`:9-11, 39-42`) — see §5's cross-chart colour-order mismatch.
- X axis: fixed `.stride(by: .day, count: 14)` (not adaptive, unlike §7.1/§7.6), labels `"MMM d"` (`:43-51`).
- Y axis: default (`:52-60`).
- Legend: bottom (`:61`).
- Selection/tooltip: `chartXSelection($selectedDate)`; dashed rule; tooltip lists `formatLocalizedDate(selectedDate)` then one line per model matching that calendar day — `"\(model): \(formatLargeNumber(tokens))"` — clamped `x∈[90, width-90]`, `y=12` (`:32-38, 66-99`).
- Height 240.

### 7.6 Tool Calls Over Time (`ToolCallsOverTimeChart.swift`)
- Type: `AreaMark` (`teal` @15% opacity) + `LineMark` (`teal`, width 2), `catmullRom` (`:14-27`).
- Series: `viewModel.filteredDailyActivity` (range-filtered), Y = `day.toolCallCount` (`AnalyticsDashboardView.swift:230-233`).
- X axis: `.stride(by: .day, count: max(dailyActivity.count / 8, 7))`, labels `"MMM d"` (`:36-44`) — same adaptive rule as §7.1.
- Y axis: default (`:45-53`).
- Selection/tooltip: `chartXSelection($selectedDate)`; dashed rule; tooltip `formatLocalizedDate(entry.date)` + `"N tool calls"`, clamped `x∈[80, width-80]`, `y=12` (`:29-35, 58-83`).
- Height 200.

---

## 8. Stat cards

Component `StatCard.swift:1-57`: icon (14pt, tinted `color`) top-left; optional `InfoTooltipButton` (an "ⓘ" that opens a click-to-toggle popover, not a hover tooltip — `InfoTooltipButton.swift:1-39`) top-right; then `value` (heading font, `textTertiary` if `dimmed` else `textPrimary`), `title` (small, `textSecondary`), and a subtitle line that renders literal `" "` (a single space) when `subtitle` is `nil`, to hold the row height constant (`:35-41`).

`AnalyticsDashboardView.summaryCards` (`:342-417`) renders exactly 8, in a 4-column `LazyVGrid`, each `height: 110`:

| # | Title | Value | Subtitle | Icon | Color | Dimmed when | Info tooltip |
|---|---|---|---|---|---|---|---|
| 1 | Total Sessions | `"\(stats.totalSessions)"` | — | `rectangle.stack.fill` | accent | never | — |
| 2 | Total Messages | `formatLargeNumber(stats.totalMessages)` | — | `message.fill` | blue | never | — |
| 3 | Longest Session | `stats.longestSessionFormatted` | `"\(stats.longestSession.messageCount) messages"` | `timer` | orange | never | — |
| 4 | First Session | `formatFirstSessionDate(stats.firstSessionParsedDate)` | — | `calendar` | green | never | — |
| 5 | Total Cost | `hasCostData ? formatCost(totalCost) : "—"` | `hasCostData ? nil : "included in subscription"` | `dollarsign.circle.fill` | green | `!hasCostData` | `!hasCostData` → *"API users see per-model costs here. Subscription plans (Max, Pro) include usage at no extra charge."* |
| 6 | Total Tokens | `formatLargeNumber(viewModel.totalTokens)` | — | `number.circle.fill` | purple | never | — |
| 7 | Tool Calls | `formatLargeNumber(viewModel.totalToolCalls)` | — | `wrench.and.screwdriver.fill` | teal | never | — |
| 8 | Time Saved | `viewModel.timeSavedFormatted` | `hasTimeSavedData ? "speculation cache" : "no cache data"` | `clock.arrow.2.circlepath` | blue | `!hasTimeSavedData` | — |

This is the exact mechanism behind "API-only/subscription messaging": card 5 shows `"—"` + the exact tooltip copy above whenever `totalCost == 0` (`AnalyticsViewModel.swift:198-200`), which — per §1.3 — is guaranteed whenever the dashboard is running off `StatsComputer`'s local recompute (subscription users, or any time the cache is stale). Card 8 similarly shows "no cache data" whenever `totalSpeculationTimeSavedMs == 0`, which is *always* true on the recompute path (§1.3).

Loading state: `AnalyticsShimmerView.swift:1-106` — while `viewModel.isLoading` is `true` (initially, and during any `loadStats()` call, `AnalyticsViewModel.swift:77-89`), the dashboard shows a shimmer skeleton (header, 8 card placeholders in the same 4-col grid, three single chart placeholders at heights 220/240/200, then two side-by-side placeholders at height 280) with a 1.5s linear, infinitely-repeating gradient sweep (`:41-45, 84-105`). No-data state (`stats == nil` after loading, i.e. `StatsComputer.compute` returned `nil` and there was no cache either): the header still renders, but the charts area shows `noStatsNotice` — icon `chart.xyaxis.line`, title `"No session analytics yet"`, body `"No Claude Code sessions found yet. Analytics are computed locally from your sessions and appear once you've used Claude Code in a project."` (`AnalyticsDashboardView.swift:253-274`).

---

## 9. CSV exports (`AnalyticsCSVExporter.swift`) — maps to `CsvKind`

All four operate on the **full, unfiltered `StatsCache`** passed into `export(_:type:)` (`:23-30`) — the CSV exports always cover all-time data, **ignoring the dashboard's currently selected date range**. Each `AnalyticsExportType` case (`:4-20`) maps 1:1 to `docs/ipc-contract.md`'s `CsvKind` (`dailyActivity | modelUsage | dailyTokens | hourlyActivity`, `ipc-contract.md:169`), and `AnalyticsExportType.allCases` (declaration order, `CaseIterable`) is the order the export menu lists them in (`AnalyticsDashboardView.swift:147-155`).

### 9.1 Daily Activity → `daily-activity.csv`
Header: `Date,Messages,Sessions,Tool Calls`. One row per `stats.dailyActivity` entry, in the array's existing order (`StatsComputer`'s local recompute sorts this ascending by date, `StatsComputer.swift:222`; a cache loaded straight from disk keeps whatever order Claude Code wrote): `date,messageCount,sessionCount,toolCallCount` — raw integers, no quoting/escaping (`AnalyticsCSVExporter.swift:34-40`).

### 9.2 Model Usage → `model-usage.csv`
Header: `Model,Input Tokens,Output Tokens,Cache Read Tokens,Cache Creation Tokens,Cost USD`. Rows from `stats.modelUsage`, **sorted by `costUSD` descending** (`:46`). Columns: `friendlyModelName(modelId)`, `inputTokens`, `outputTokens`, `cacheReadInputTokens`, `cacheCreationInputTokens`, `costUSD` formatted with `String(format: "%.2f", ...)` — always 2 decimal places (note: **not** the tiered `formatCost` used elsewhere in the UI) (`:44-53`).

### 9.3 Daily Tokens → `daily-tokens.csv`
`allModels = Set(stats.dailyModelTokens.flatMap(\.tokensByModel.keys)).sorted()` — the union of every raw model id seen across all days, sorted, then mapped to friendly names (`:58-59`). Header: `Date,` + friendly names joined by `,`. **Column headers are friendly names, not raw ids** — if two distinct raw model ids map to the same friendly name (e.g. two dated snapshots of "sonnet-4-5"), the header row will contain a duplicate column name. One row per day: `date,` + one value per model column, `tokensByModel[model] ?? 0` for missing entries (`:57-66`).

### 9.4 Hourly Activity → `hourly-activity.csv`
Header: `Hour,Sessions`. One row per `stats.sortedHourCounts` entry (`StatsCache.swift:94-101`, hour ascending 0-23) — **only hours present in the sparse `hourCounts` dictionary appear**; hours with zero sessions are simply absent rows, not zero-filled (`AnalyticsCSVExporter.swift:70-76`).

### 9.5 Save behaviour
`NSSavePanel` with `allowedContentTypes = [.commaSeparatedText]`, `nameFieldStringValue = type.suggestedFileName` (as listed under each heading above), `canCreateDirectories = true`; on confirm, writes the CSV string atomically as UTF-8 (`:81-90`). Write failures are silently swallowed via `try?` (`:89`) — for Heimdall's `export_analytics_csv` command this must instead surface `WriteOutcome`/`AppError` per ADR 0005/the IPC contract, not swallow.

---

## 10. PNG snapshot export — maps to `PngKind.analytics`

Entry point: `AnalyticsDashboardView.shareAsImage(_:)` builds `AnalyticsSnapshotView(stats: stats, viewModel: viewModel)` and passes it to `AnalyticsImageExporter.renderToImage` (`AnalyticsDashboardView.swift:168-175`).

### 10.1 Rendering (forced dark, branded header, 2x scale)
`AnalyticsImageExporter.renderToImage(content, width: 1280)` (`AnalyticsImageExporter.swift:7-24`):
```swift
let compositeView = VStack(spacing: 0) {
    AnalyticsBrandHeader()
    content
}
.frame(width: width)                       // fixed width 1280pt
.environment(\.colorScheme, .dark)         // forced dark, regardless of the app's current theme/appearance
.background(PoirotTheme.Colors.bgApp)

let renderer = ImageRenderer(content: compositeView)
renderer.scale = 2.0                        // 2x / Retina raster scale
```
The returned `NSImage` is sized at logical points (`cgImage.width/2`, `cgImage.height/2`) even though the backing raster is 2x (`:22-23`).

Brand header (`AnalyticsBrandHeader`, `AnalyticsImageExporter.swift:46-75`): horizontal row — `PoirotLogo` image (24pt tall) — `"the Swift reference app"` (heading font) — `"·"` separator — `"poirot.fyi"` (small font, `accent` colour) — spacer — `"Claude Code Companion for macOS"` (micro font, `textTertiary`, trailing). Padding `xxl` horizontal / `lg` vertical, background `bgApp`.

Save panel (`:26-42`): `NSSavePanel`, `allowedContentTypes = [.png]`, default filename `"poirot-analytics.png"`, `canCreateDirectories = true`; writes via `tiffRepresentation` → `NSBitmapImageRep` → `.png` representation.

### 10.2 Snapshot content and layout (`AnalyticsSnapshotView.swift`)
A **statically rebuilt** copy of the dashboard, not a re-render of `AnalyticsDashboardView` — differences from the live/interactive dashboard:

- No toolbar, no date-range picker, no export menu. Still shows the currently-`selectedDateRange`-filtered data (via the same `viewModel.filteredDailyActivity`/`tokenTimeSeriesData`), but the picker control itself isn't drawn. Header is simplified to two lines: `"Session Analytics"` + `"Last computed: \(formatLocalizedDate(stats.lastComputedDate))"` (`:139-149`) — no "last computed" capsule styling, no custom-range badge (both present in the live header, `AnalyticsDashboardView.swift:295-319`).
- Layout order, top to bottom: header → summary cards (8, identical grid) → Daily Activity (inline `Chart`, no selection/overlay) → Hourly Activity (reuses `HourlyActivityChart` as-is) → static heatmap → Token Usage Over Time (inline, no selection/overlay) → Tool Calls Over Time (inline, no selection/overlay) → `HStack[donut Model Usage, CostBreakdownView]` (`:12-135`). Whole thing padded `xxl`, background `bgApp`.
- Summary cards (`:153-223`) are pixel-identical to the live dashboard's 8 cards (§8) **except** the Total Cost card omits the `info:` parameter entirely (`:188-195`) — the API-only/subscription explanatory tooltip never appears in the PNG (there is no tooltip affordance at all in a static raster).
- Daily Activity / Token Usage / Tool Calls charts are hand-rewritten inline versions with the axis/plot-style config duplicated verbatim but with **no** `chartXSelection`, `RuleMark`, or `chartOverlay` tooltip — appropriate since `ImageRenderer` takes one static pass (`:18-54, 61-82, 85-105`).
- **Token Usage Over Time and Model Usage donut omit `chartForegroundStyleScale` entirely** (`:61-82, 110-121`) — unlike their live-dashboard counterparts (§7.3, §7.5), so these two charts' colours in the PNG come from SwiftUI's own default categorical palette, not `AnalyticsColorPalette`. This is a real behavioural discrepancy between the live dashboard and the exported image (see Open questions §11.4).
- The Model Usage donut in the PNG is **not** a reuse of `ModelUsageChart`; it's built inline directly from `stats.modelUsage.sorted(by: input+output desc)` (`:110-121`) with no legend and no center label at all (the live `ModelUsageChart` has both a bottom legend and a selection-driven center label, §7.3).
- `CostBreakdownView` **is** reused unmodified (`:125-128`).
- Static heatmap (`:228-262`) — see §6.1; the grid/colour algorithms are copy-pasted verbatim (not shared code) from `ContributionHeatmap`, dropping the `ScrollView`/`GeometryReader`/hover-tooltip machinery (a raster export can't be interactive).

---

## 11. Context window (`ContextUsage.swift`) — standard vs extended window

`ContextUsage` (`ContextUsage.swift:7-58`) is derived **entirely from local transcript per-turn `usage` data** — no network, no Keychain (`:3-6`). It is used by the **session** header's `ConversationContextCard`, not the analytics dashboard.

### 11.1 Constants
```swift
static let standardWindow = 200_000
static let extendedWindow = 1_000_000
```
(`:9-10`) — comment: "the 1M tier is a beta only some models/turns use" (`:8`).

### 11.2 Construction (`ContextUsage.from(_ messages: [Message])`, `:39-57`)
```
usages = messages.compactMap(\.tokenUsage)                          // per-message TokenUsage, nil-filtered out
series = usages.map(\.contextTokens).filter { $0 > 0 }               // contextTokens = input + cacheRead + cacheCreation (Message.swift:151)
peak   = series.max() ?? 0
window = peak > standardWindow ? extendedWindow : standardWindow      // 1,000,000 if ANY turn exceeded 200,000, else 200,000
current = series.last ?? 0                                            // most recent non-zero-context turn
inputTokens        = sum(usages.input)          // NOTE: sums ALL usages, not the zero-filtered `series`
outputTokens       = sum(usages.output)
cacheReadTokens     = sum(usages.cacheRead)
cacheCreationTokens = sum(usages.cacheCreation)
```
The **window rule** (exact, per the acceptance criterion): the extended (1,000,000-token) window is inferred if and only if the largest single-turn `contextTokens` value across the whole conversation exceeds `standardWindow` (200,000); otherwise the standard 200,000-token window is assumed. There is no model-name lookup — it's inferred purely from observed usage, per conversation. (Contrast with `StatsComputer`'s separate, model-granularity version of the same 200k/1M rule at aggregate-cache-build time, §1.3.)

### 11.3 Derived values
- `isEmpty = series.isEmpty` (`:24`).
- `fraction = window > 0 ? clamp(current/window, 0, 1) : 0` (`:27-29`).
- `percent = Int((fraction * 100).rounded())` (`:31`).
- `cacheHitRate = promptTotal > 0 ? cacheReadTokens / promptTotal : 0`, where `promptTotal = inputTokens + cacheReadTokens + cacheCreationTokens` (`:34-37`).

### 11.4 `ConversationContextCard` (the context ring UI)
`ConversationContextCard.swift`:
- Ring: `UsageWindow(utilization: context.fraction * 100, resetsAt: nil)` (`:9-11`) rendered via `UsageGaugeRing`, 50×50pt (`:39-40`). Label text: `"Context window"`, then `"\(formatLargeNumber(current)) / \(formatLargeNumber(window))"`, then `"\(round(cacheHitRate*100))% from cache"` (`:43-51`, `formatLargeNumber` = `AnalyticsFormatters.formatLargeNumber`).
- Token-mix bar: a 150×6pt capsule-clipped horizontal bar with three proportional segments, in this order: cache (`cacheReadTokens + cacheCreationTokens`, `blue`), input (`inputTokens`, `teal`), output (`outputTokens`, `accent`) — segment width = `barWidth * value / max(1, total)` (`:58-93`). Legend chips below repeat the same three values/colours/order, labelled `"Cache"`, `"In"`, `"Out"` (`:68-72, 96-105`).
- Sparkline: only rendered when `context.series.count > 1` (`:19-21`), labelled `"Context over N turns"`. `ContextSparkline` (`:129-150`) is a plain polyline (no fill, no smoothing — unlike the analytics area charts' `catmullRom`), normalizing each `series[i]` against `ceiling = context.window` (i.e. against the inferred 200k/1M window, not against `peak`), stroked with `ringWindow.severity.tint`, width 1.5, `lineJoin: .round`, size 150×32pt (`:109-118`).
- Severity thresholds driving `ringWindow.severity.tint` (`UsageWindow.swift:7-23`): `utilization < 80` → `.normal`; `80 ≤ utilization < 95` → `.warning`; `utilization ≥ 95` → `.critical` (utilization is a 0-100 percentage, `fraction * 100`). The actual `.tint` colour values are defined on `UsageSeverity`/`UsageGaugeRing` outside this task's file list — see Open questions §11.6.

---

## 12. Open questions

1. **Picker binding never highlights "Custom".** `AnalyticsDashboardView`'s segmented `Picker`'s `get` closure returns `"Custom"` when a custom range is active, but the picker's items are built only from `AnalyticsDateRange.presets`, which excludes `.custom` (`AnalyticsDashboardView.swift:65-87`; `AnalyticsViewModel.swift:25`). So no segment is ever visibly selected while a custom range is active. Should Heimdall reproduce "nothing highlighted", or add a fifth segment / different affordance?
2. **"(UTC)" subtitle vs local-time bucketing.** `HourlyActivityChart`'s subtitle claims `"Session start times (UTC)"` (`HourlyActivityChart.swift:11`), but `StatsComputer` buckets hours with `Calendar.current.component(.hour, from:)`, i.e. local time (`StatsComputer.swift:153`). It's undocumented (upstream) whether Claude Code's own `stats-cache.json` `hourCounts` (used verbatim whenever the cache is fresh) is UTC or local. Which should Heimdall's Rust port use, and what should the axis subtitle say?
3. **Palette assignment is per-chart, not per-model.** `AnalyticsColorPalette.colors(count:)` only has 6 slots and is re-sliced independently by `ModelUsageChart` (sorted by tokens desc), `CostBreakdownView` (sorted by cost desc) and `TokenUsageOverTimeChart` (sorted alphabetically) — the same model can get a different colour on different charts, and a 7th+ model has no colour at all (§5, §7.3, §7.4, §7.5). Intentional, or should Heimdall assign one stable colour per model id across all charts?
4. **PNG export drops the palette on two charts.** `AnalyticsSnapshotView`'s inline Token-Usage-Over-Time chart and Model-Usage donut omit `chartForegroundStyleScale` (`AnalyticsSnapshotView.swift:61-82, 110-121`), so their colours in the exported PNG are SwiftUI's default categorical palette, not `AnalyticsColorPalette` — a discrepancy from the live dashboard (§10.2). Is this an oversight to fix in the Heimdall port, or intentional?
5. **Dead view-model properties.** `filteredTotalMessages`, `filteredTotalSessions`, `filteredTotalToolCalls` (`AnalyticsViewModel.swift:211-221`) are computed but have no callers anywhere in `Poirot/Sources` (confirmed by grep). Should Heimdall implement equivalents (e.g. for a future "range totals" strip) or omit them as dead code?
6. **Severity tint colours are out of scope.** `UsageWindow.swift` (read for this task) defines only the 80%/95% thresholds; the actual colour values for `.normal`/`.warning`/`.critical` live in `UsageGaugeRing`/`UsageSeverity` extensions this task's input list didn't include. Needs its own citation before Heimdall implements the context ring's exact colours.
7. **`StatsCache.ModelUsage.contextWindow`/`.maxOutputTokens` appear unused.** They're decoded from Claude Code's cache (and written by `StatsComputer`'s recompute, with a hardcoded `32000` for `maxOutputTokens`, `StatsComputer.swift:228-230, 241`) but nothing in the analytics dashboard or `ConversationContextCard` reads either field — only `ContextUsage.window` (inferred per-conversation) drives the ring. Confirm Heimdall's port can ignore these two `ModelUsage` fields for now.
8. **`costBreakdownEntries` sort stability.** `stats.modelUsage` is a Swift `[String: ModelUsage]` dictionary with no defined iteration order; `costBreakdownEntries`/CSV model-usage export both sort by `cost`/`costUSD` descending, but ties (e.g. multiple free-tier models all at `$0`) have unspecified relative order (`AnalyticsViewModel.swift:178-190`; `AnalyticsCSVExporter.swift:46`). Should Heimdall define a deterministic tiebreaker (e.g. model id) for parity/testability?
9. **`latestSessionDay` staleness check uses file mtime, not content.** `StatsComputer.latestSessionDay` reads `contentModificationDate` (`StatsComputer.swift:61-86`), not the transcript's own last timestamp. A copied/touched/restored-from-backup `.jsonl` file could trigger a false "stale" recompute or mask genuine staleness. Worth confirming Heimdall's Rust port wants the same heuristic or should parse the file's last line's timestamp instead.
