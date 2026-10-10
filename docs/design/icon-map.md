# Icon map: SF Symbols to Lucide

Every SF Symbol name `$POIROT` actually uses, mapped to a real Lucide icon (verified against
`node_modules/@lucide/vue`'s own exported component names — never guessed). The live mapping is
`src/components/iconMap.ts`; this document is its provenance table plus the real gaps, not a
second source of truth to keep in sync by hand.

**A cross-cutting gap, not per-icon:** Lucide ships outline icons only — there is no separate
"filled" icon set. Every SF Symbol `.fill` variant (`checkmark.circle.fill`, `star.fill`, …) maps
to the *same* Lucide icon as its non-fill counterpart. This is not a mistake per entry below; it
is the one structural limit every `.fill` row inherits.

## 1. Direct `Image(systemName:)` literals (61, pinned at commit `a29dd6e`)

| SF Symbol | Lucide | Notes |
|---|---|---|
| `arrow.clockwise` | `RotateCw` | |
| `arrow.down` | `ArrowDown` | |
| `arrow.right.circle` | `CircleArrowRight` | |
| `arrow.triangle.branch` | `GitBranch` | |
| `arrow.up.arrow.down` | `ArrowDownUp` | |
| `arrow.up.forward` | `ArrowUpRight` | |
| `arrow.up.forward.square` | `SquareArrowOutUpRight` | |
| `brain.head.profile` | `Brain` | |
| `chart.xyaxis.line` | `ChartLine` | |
| `checklist` | `ListChecks` | |
| `checkmark.circle` | `CircleCheck` | |
| `checkmark.circle.fill` | `CircleCheck` | fill gap |
| `chevron.down` | `ChevronDown` | |
| `chevron.left` | `ChevronLeft` | |
| `chevron.right` | `ChevronRight` | |
| `circle` | `Circle` | |
| `circle.dotted` | `CircleDashed` | |
| `clock.arrow.2.circlepath` | `RotateCwClock` | **gap**: no direct "history" icon exists; closest real match |
| `cpu` | `Cpu` | |
| `curlybraces` | `Braces` | |
| `doc.on.doc` | `Files` | |
| `doc.text` | `FileText` | |
| `doc.text.magnifyingglass` | `FileSearch` | |
| `ellipsis.circle` | `CircleEllipsis` | |
| `exclamationmark.circle.fill` | `CircleAlert` | fill gap |
| `exclamationmark.triangle` | `TriangleAlert` | |
| `exclamationmark.triangle.fill` | `TriangleAlert` | fill gap |
| `folder` | `Folder` | |
| `folder.badge.minus` | `FolderMinus` | |
| `folder.badge.plus` | `FolderPlus` | |
| `folder.fill` | `Folder` | fill gap |
| `heart.fill` | `Heart` | fill gap |
| `info.circle` | `Info` | Lucide's own `Info` glyph is already circled |
| `keyboard` | `Keyboard` | |
| `ladybug` | `Bug` | |
| `ladybug.fill` | `Bug` | fill gap |
| `line.3.horizontal.decrease` | `ListFilter` | |
| `line.3.horizontal.decrease.circle` | `ListFilter` | no circled variant exists; reused |
| `list.bullet.clipboard.fill` | `ClipboardList` | fill gap |
| `lock.shield` | `Lock` | no shield+lock combo exists; reused |
| `magnifyingglass` | `Search` | |
| `minus` | `Minus` | |
| `minus.circle.fill` | `CircleMinus` | fill gap |
| `pencil` | `Pencil` | |
| `plus` | `Plus` | |
| `powerplug` | `Plug` | |
| `questionmark.circle.fill` | `CircleQuestionMark` | fill gap |
| `rectangle.stack` | `Layers` | |
| `rectangle.stack.fill` | `Layers` | fill gap |
| `shippingbox` | `Package` | |
| `sparkles` | `Sparkles` | |
| `square.and.arrow.down` | `SquareArrowDown` | |
| `square.and.arrow.up` | `SquareArrowUp` | |
| `star` | `Star` | |
| `star.fill` | `Star` | fill gap |
| `target` | `Target` | |
| `terminal` | `Terminal` | |
| `text.bubble` | `MessageSquare` | |
| `trash` | `Trash` | |
| `xmark` | `X` | |
| `xmark.circle.fill` | `CircleX` | fill gap |
| `xmark.circle` | `CircleX` | (also appears bare, HookEvent.icon) |

## 2. `NavigationItem.systemImage` (`Models/Provider/NavigationItem.swift`)

| SF Symbol | Lucide | Notes |
|---|---|---|
| `rectangle.stack.fill` | `Layers` | (listed above too — same icon) |
| `checklist` | `ListChecks` | (listed above too) |
| `apple.terminal.fill` | `SquareTerminal` | boxed look stands in for "apple.terminal" |
| `bolt.fill` | `Zap` | |
| `list.bullet.clipboard.fill` | `ClipboardList` | (listed above too) |
| `powerplug.fill` | `Plug` | fill gap |
| `brain.fill` | `Brain` | fill gap |
| `person.2.fill` | `Users` | fill gap |
| `arrow.triangle.branch` | `GitBranch` | (listed above too) |
| `puzzlepiece.fill` | `Puzzle` | fill gap |
| `speaker.wave.3.fill` | `Volume2` | fill gap |
| `brain.head.profile.fill` | `Brain` | fill gap |
| `chart.xyaxis.line` | `ChartLine` | (listed above too) |
| `clock.arrow.circlepath` | `RotateCwClock` | same gap as `clock.arrow.2.circlepath` above |

## 3. `HookEvent.icon` (`Models/Config/Hook.swift`, 18 cases)

| SF Symbol | Lucide | Notes |
|---|---|---|
| `hand.raised` | `Hand` | |
| `checkmark.circle` | `CircleCheck` | (listed above too) |
| `xmark.circle` | `CircleX` | |
| `lock.shield` | `Lock` | (listed above too) |
| `play.circle` | `CirclePlay` | |
| `stop.circle` | `CircleStop` | |
| `text.bubble` | `MessageSquare` | (listed above too) |
| `bell` | `Bell` | |
| `person.badge.plus` | `UserPlus` | |
| `person.badge.minus` | `UserMinus` | |
| `stop.fill` | `Square` | **gap**: no dedicated "filled stop square" icon; plain `Square` stands in |
| `person.crop.circle.badge.clock` | `UserRoundCog` | **gap**: no person+clock-badge combo exists; closest real match |
| `checkmark.seal` | `BadgeCheck` | |
| `doc.text` | `FileText` | (listed above too) |
| `gearshape` | `Settings` | |
| `plus.rectangle.on.folder` | `FolderInput` | |
| `minus.rectangle` | `SquareMinus` | |
| `arrow.down.right.and.arrow.up.left` | `Minimize2` | |

## 4. `SubAgent.icon` built-in defaults (`Models/Config/SubAgent.swift`)

`SubAgent.icon` is a free-text `String` field with no fixed case set (confirmed: no icon picker
list exists anywhere in `$POIROT`'s source) — only the four built-in agents' own default values
have a real mapping. **Gap:** a user-authored sub-agent can set `icon` to any string; `AppIcon`'s
own fallback (a `CircleQuestionMark` marker, not a silent blank) is what actually handles that
case, not an entry in this table.

| SF Symbol | Lucide | Notes |
|---|---|---|
| `magnifyingglass` | `Search` | (listed above too) |
| `map` | `Map` | |
| `terminal` | `Terminal` | (listed above too) |
| `person.fill` | `UserRound` | fill gap |

## 5. MCP catalog entry and category icons (`Models/Config/MCPServerCatalog.swift`)

| SF Symbol | Lucide | Notes |
|---|---|---|
| `arrow.triangle.branch` | `GitBranch` | (listed above too) |
| `theatermasks` | `Drama` | exact semantic match |
| `globe` | `Globe` | |
| `ladybug` | `Bug` | (listed above too) |
| `list.bullet.rectangle` | `List` | |
| `number` | `Hash` | |
| `doc.richtext` | `FileText` | no distinct "rich text doc" icon exists; reused |
| `externaldrive.badge.icloud` | `CloudUpload` | |
| `magnifyingglass` | `Search` | (listed above too) |
| `arrow.down.doc` | `FileDown` | |
| `cylinder` | `Database` | |
| `tablecells` | `Table` | |
| `folder.badge.gearshape` | `FolderCog` | exact semantic match |
| `brain` | `Brain` | |
| `paintpalette` | `Palette` | |
| `hammer` | `Hammer` | (category icon) |
| `tray.full` | `Inbox` | (category icon) |
| `magnifyingglass.circle` | `Search` | **gap**: no circled-magnifying-glass icon exists; reused (category icon) |
| `folder` | `Folder` | (category icon, listed above too) |

## Motion effects (task M2-04, `src/styles/motion.css`)

The AC names seven utilities (`bounce`, `pulse`, `breathe`, `variable-color`, `rotate`, `wiggle`,
`replace`) — the full vocabulary of SwiftUI's built-in symbol effects. Checked directly against
real usage (`grep -rn "symbolEffect" Poirot/Sources/`): `bounce`, `rotate`, `replace`, `pulse` and
`breathe` are each used for real (`ToastOverlay.swift`, `AnalyticsDashboardView.swift`,
`MCPServerStatusIndicator.swift`, `MemoryListView.swift`, and the many `.contentTransition(.symbolEffect(.replace))`
sites). **`variable-color` and `wiggle` have zero real call sites anywhere in `$POIROT`'s
source** — implemented in `motion.css` per the AC regardless, with `variable-color` built as an
opacity-cycle approximation (a single-path Lucide icon has no internal layered regions to
sequentially highlight, which is what SF Symbols' real variable-color effect does).

## Summary

- 101 distinct SF Symbol names across all five sources map to 82 distinct Lucide icons.
- Real gaps (no faithful Lucide equivalent exists, a close stand-in was chosen instead): the
  "history" icons (`clock.arrow.2.circlepath`, `clock.arrow.circlepath`), `person.crop.circle.badge.clock`,
  `stop.fill`, `magnifyingglass.circle`, and every `SubAgent.icon` value beyond the four built-in
  defaults.
- Every `.fill` SF Symbol variant reuses its non-fill Lucide mapping (Lucide has no filled set).

## 6. Added after M2-04: `SessionsNavigationView.swift`'s own literals (task M2-15)

Outside the five sources above (M2-04's own closed inventory) because this view belongs to a
later milestone's own task, not M2-04's. Found while building M2-15's sessions list.

| SF Symbol | Lucide icon | Note |
|---|---|---|
| `person.2` | `Users` | the agent-sessions toggle's "off" state |
| `person.2.wave.2` | `Users` | the toggle's "on" state — another real gap, no "waving hand" variant exists in Lucide, so both states share the one icon (same fill-gap convention as `person.2.fill` above) |
| `checkmark` (bare, not `.circle`/`.circle.fill`) | `Check` | the sort menu's active-option indicator |

## 7. Added after M2-04: `ProjectSessionsView.swift`'s own literals (task M2-16)

| SF Symbol | Lucide icon | Note |
|---|---|---|
| `list.bullet` | `List` | the layout-toggle button, shown when the grid is active |
| `square.grid.2x2` | `Grid2x2` | the layout-toggle button, shown when the list is active |

## 8. Added after M2-04: `ThinkingBlockView.swift`'s own literals (task M2-17)

| SF Symbol | Lucide icon | Note |
|---|---|---|
| `text.word.spacing` | `TextWrap` | the wrap-toggle button's "wrapping on" state |
| `arrow.right.to.line` | `ArrowRightToLine` | the wrap-toggle button's "wrapping off" (horizontal-scroll) state |

## 9. Added after M2-04: `ToolBlockView.swift`'s own literals (task M2-18)

| SF Symbol | Lucide icon | Note |
|---|---|---|
| `doc.text.fill` | `FileText` | the `Write` tool's own icon — fill-gap reuse of `doc.text`'s mapping |
| `doc.plaintext` | `File` | the markdown-toggle button's "raw" state — deliberately not `doc.richtext`'s own `FileText`, so the two toggle states look different |
| `text.magnifyingglass` | `FileSearch` | the `Grep` tool's own icon |
| `wrench` | `Wrench` | the fallback icon for any tool name with no entry in the real tool-definitions table |

## 10. Added after M2-04: `SessionToolbarContent.swift`'s own literals (task M2-22)

| SF Symbol | Lucide icon | Note |
|---|---|---|
| `arrow.uturn.forward` | `CornerUpRight` | the Resume button's own icon |
| `arrow.up.left.and.arrow.down.right` | `Maximize2` | "Expand All" — the mirror of `arrow.down.right.and.arrow.up.left`'s own `Minimize2` ("Collapse All", table 3) |
