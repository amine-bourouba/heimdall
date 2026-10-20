# Task backlog

One file per task. Each file is self-contained: an agent needs only its task file and the files it lists as Inputs.

- **Scope** says whether the feature ships in the MVP (`MVP`), after it (`Later`), or is project infrastructure (`Infra`). Milestone gates list their required tasks explicitly in `docs/port-plan.md` §7.
- **Placeholders.** A skeleton task may create empty placeholder files that a later task owns (port-plan §5.4). The owner always depends on the skeleton.
- **Parallel safety.** Two tasks that write the same file are always ordered by dependencies. `docs/progress-log.md` is append-only and exempt.
- **Check.** Run `python docs/tasks/check_backlog.py --swift-ref <path-to-swift-reference-checkout>` after editing any task.

## M0 — Inventory (16 tasks)

| ID | Task | Workstream | Scope | Size | Depends on |
|---|---|---|---|---|---|
| [M0-01](M0-01-rules-file.md) | Write Heimdall's AI_INSTRUCTIONS.md and CLAUDE.md | Docs | Infra | S | — |
| [M0-02](M0-02-record-identity-decisions.md) | Record the final app identifier and Linux package name | Docs | Infra | S | — |
| [M0-03](M0-03-repo-bootstrap.md) | Initialise the Heimdall repository and its ignore/attribute rules | Release and packaging | Infra | S | — |
| [M0-04](M0-04-license-readme.md) | Add LICENSE, NOTICE and a scoped README | Docs | Infra | S | — |
| [M0-05](M0-05-claude-home-inventory.md) | Exhaustive inventory of every ~/.claude path the Swift reference app reads or writes | Legacy parity extraction | Infra | S | — |
| [M0-06](M0-06-test-carryover-map.md) | Map every PoirotTests case to its Heimdall destination | Legacy parity extraction | Infra | M | — |
| [M0-07](M0-07-spec-search-overlay.md) | Behaviour spec for universal search | Legacy parity extraction | Infra | S | M0-05 |
| [M0-08](M0-08-spec-session-views.md) | Behaviour spec for the session browser and conversation view | Legacy parity extraction | Infra | M | M0-05 |
| [M0-09](M0-09-spec-config-screens.md) | Behaviour spec for the configuration screens, forms and MCP wizard | Legacy parity extraction | Infra | M | M0-05 |
| [M0-10](M0-10-spec-analytics.md) | Behaviour spec for analytics, exports and the context ring | Legacy parity extraction | Infra | S | M0-05 |
| [M0-11](M0-11-fixtures-synthetic.md) | Extract PoirotTests inline fixtures into a shared synthetic ~/.claude tree | Testing | Infra | M | M0-03, M0-05, M0-06 |
| [M0-12](M0-12-fixtures-screenshot.md) | Convert ScreenshotData.swift into a deterministic fixture tree | Testing | Infra | M | M0-03, M0-05 |
| [M0-13](M0-13-scrub-tool.md) | Write the ~/.claude scrubbing tool | Testing | Infra | S | M0-03, M0-05 |
| [M0-14](M0-14-fixtures-windows-capture.md) | Capture a scrubbed real Windows ~/.claude tree | Testing | Infra | S | M0-11, M0-13 |
| [M0-15](M0-15-fixtures-linux-capture.md) | Capture a scrubbed real Linux ~/.claude tree | Testing | Infra | S | M0-11, M0-13 |
| [M0-16](M0-16-upstream-sync-procedure.md) | Define and run the upstream the Swift reference app sync check | Legacy parity extraction | Infra | S | — |

## M1 — Rust data layer (59 tasks)

| ID | Task | Workstream | Scope | Size | Depends on |
|---|---|---|---|---|---|
| [M1-01](M1-01-cargo-workspace.md) | Create the Cargo workspace, crate manifests and supply-chain policy | Release and packaging | Infra | M | M0-02, M0-03 |
| [M1-02](M1-02-npm-manifest.md) | Create package.json with the full frontend toolchain | Release and packaging | Infra | S | M0-03 |
| [M1-03](M1-03-tauri-skeleton.md) | Scaffold the Tauri app crate with final identity, CSP and module stubs | Tauri shell / OS integration | Infra | M | M1-01, M1-02, M1-04, M0-02 |
| [M1-04](M1-04-vue-scaffold.md) | Scaffold the Vite + Vue 3 + TypeScript + Tailwind v4 frontend | Vue shell and design system | Infra | S | M1-02 |
| [M1-05](M1-05-core-skeleton-errors.md) | Declare every heimdall-core module and implement AppError | Rust data core | MVP | M | M1-01 |
| [M1-06](M1-06-paths.md) | Claude home resolution, path normalisation and comparison keys | Rust data core | MVP | M | M1-05 |
| [M1-07](M1-07-jsonl-reader.md) | Streaming JSONL reader with CRLF and truncation tolerance | Rust data core | MVP | S | M1-05 |
| [M1-08](M1-08-models-transcript.md) | Serde models for messages, sessions and projects | Rust data core | MVP | M | M1-05, M0-06 |
| [M1-09](M1-09-models-data.md) | Serde models for stats, history, todos, facets, debug log and file history | Rust data core | MVP | M | M1-05, M0-06 |
| [M1-10](M1-10-models-config-secrets.md) | Config models, catalogs and the redacting Secret type | Rust data core | MVP | L | M1-05, M0-06 |
| [M1-11](M1-11-models-prefs.md) | Typed Prefs model with per-OS defaults | Rust data core | MVP | S | M1-05 |
| [M1-12](M1-12-transcript-full.md) | Transcript parser: full-parse tier, blocks, system content, tool input | Rust data core | MVP | L | M0-08, M0-11, M0-14, M1-07, M1-08 |
| [M1-13](M1-13-transcript-header-summary.md) | Transcript parser: header and summary tiers | Rust data core | MVP | M | M0-08, M1-12 |
| [M1-14](M1-14-projects-loader.md) | Project and session discovery with cwd-first project paths | Rust data core | MVP | L | M0-05, M1-06, M1-13 |
| [M1-15](M1-15-history-reader.md) | Prompt history reader | Rust data core | MVP | S | M1-07, M1-09 |
| [M1-16](M1-16-todos-loader.md) | Todo loader | Rust data core | MVP | S | M1-06, M1-09 |
| [M1-17](M1-17-debug-log-loader.md) | Debug log loader | Rust data core | MVP | S | M1-07, M1-09 |
| [M1-18](M1-18-facets-loader.md) | Session facets loader | Rust data core | MVP | S | M1-09 |
| [M1-19](M1-19-file-history-loader.md) | File history loader with cross-OS file matching | Rust data core | MVP | M | M1-06, M1-07, M1-09 |
| [M1-20](M1-20-stats-engine.md) | Stats cache reader, staleness rules and the full StatsComputer port | Rust data core | Later | L | M0-10, M1-07, M1-09 |
| [M1-21](M1-21-tray-aggregation.md) | Tray snapshot aggregation | Rust data core | Later | S | M1-14, M1-20, M1-29 |
| [M1-22](M1-22-safe-write-primitive.md) | Safe JSON write primitive for Claude Code's files | Rust data core | Later | L | M0-11, M0-14, M1-05, M1-06 |
| [M1-23](M1-23-frontmatter.md) | Frontmatter parser | Rust data core | MVP | S | M1-05 |
| [M1-24](M1-24-config-read-markdown-items.md) | Loaders for commands, skills, plans, output styles and agents | Rust data core | MVP | M | M1-06, M1-10, M1-23 |
| [M1-25](M1-25-config-read-settings-hooks.md) | Settings and hooks readers | Rust data core | MVP | M | M1-06, M1-10 |
| [M1-26](M1-26-config-read-mcp-plugins.md) | MCP server (five sources) and plugin readers | Rust data core | MVP | M | M1-06, M1-10 |
| [M1-27](M1-27-config-read-memory-models.md) | Memory and models readers | Rust data core | MVP | M | M1-10, M1-11, M1-14 |
| [M1-28](M1-28-mcp-status.md) | MCP process status on sysinfo | Rust data core | Later | M | M1-26 |
| [M1-29](M1-29-search-matcher.md) | Fuzzy and exact matcher with UTF-16 highlight ranges | Rust data core | MVP | M | M1-05, M0-07 |
| [M1-30](M1-30-search-providers.md) | Search providers for all 16 categories | Rust data core | MVP | M | M1-14, M1-15, M1-16, M1-17, M1-18, M1-19, M1-24, M1-25, M1-26, M1-27, M1-29 |
| [M1-31](M1-31-cli-locate.md) | Cross-platform PATH search for claude and editor CLIs | Rust data core | MVP | S | M1-06 |
| [M1-32](M1-32-launch-argv.md) | Validated argv builders for terminals and editors | Rust data core | Later | M | M1-06, M1-11, M1-31 |
| [M1-33](M1-33-link-policy.md) | External URL allowlist | Rust data core | MVP | S | M1-05 |
| [M1-34](M1-34-path-classifier.md) | Changed-path to data-area classifier | Rust data core | MVP | S | M1-06 |
| [M1-35](M1-35-line-diff.md) | Line diff port | Rust data core | MVP | S | M1-05 |
| [M1-36](M1-36-session-export-core.md) | Session Markdown and patch export | Rust data core | MVP | M | M0-08, M1-12, M1-35 |
| [M1-37](M1-37-ipc-contract-freeze.md) | Freeze the IPC contract as compiled stubs and generated TypeScript bindings | Rust data core | MVP | L | M1-03, M1-04, M1-05, M1-08, M1-09, M1-10, M1-11 |
| [M1-38](M1-38-state-cache.md) | App state and the (mtime, size)-validated cache | Rust data core | MVP | M | M1-06, M1-37 |
| [M1-39](M1-39-prefs-store.md) | Preferences storage and preference commands | Rust data core | MVP | M | M1-11, M1-37, M1-38 |
| [M1-40](M1-40-cmd-app.md) | App commands: app info, sidebar counts, CLI detection | Rust data core | MVP | S | M1-15, M1-16, M1-24, M1-25, M1-26, M1-27, M1-31, M1-38 |
| [M1-41](M1-41-cmd-sessions-read.md) | Session read commands | Rust data core | MVP | M | M1-12, M1-13, M1-14, M1-38 |
| [M1-42](M1-42-cmd-session-data.md) | Session data commands: todos, file history, diffs, debug log, facets | Rust data core | MVP | M | M1-16, M1-17, M1-18, M1-19, M1-35, M1-38 |
| [M1-43](M1-43-cmd-history-read.md) | History read command | Rust data core | MVP | S | M1-15, M1-38 |
| [M1-44](M1-44-cmd-config-read.md) | Config read commands | Rust data core | MVP | M | M1-24, M1-25, M1-26, M1-27, M1-38, M1-39 |
| [M1-45](M1-45-cmd-mcp-read.md) | MCP read commands | Rust data core | MVP | S | M1-26, M1-28, M1-38 |
| [M1-46](M1-46-cmd-search.md) | Search command | Rust data core | MVP | S | M1-30, M1-38 |
| [M1-47](M1-47-watcher.md) | Recursive watcher with debounce, classification, fallback and data-changed events | Rust data core | MVP | L | M1-34, M1-37, M1-38, M1-39 |
| [M1-48](M1-48-capabilities.md) | Per-window capability files | Security | MVP | M | M1-49 |
| [M1-49](M1-49-command-scoping-build.md) | Scope app commands with AppManifest and verify exposure | Tauri shell / OS integration | MVP | S | M1-37 |
| [M1-50](M1-50-golden-oracle-harness.md) | Swift golden-output harness for the reference app | Legacy parity extraction | MVP | M | M0-05, M0-06, M0-07, M0-08, M0-09, M0-10, M0-11, M0-12, M0-14, M0-15, M1-08, M1-09 |
| [M1-51](M1-51-golden-oracle-workflow.md) | macOS workflow that runs the golden oracle | Release and packaging | MVP | M | M1-50 |
| [M1-52](M1-52-golden-outputs-commit.md) | Commit the golden outputs | Testing | MVP | S | M1-51 |
| [M1-53](M1-53-golden-parity-tests.md) | Rust versus Swift golden parity tests | Testing | MVP | M | M1-12, M1-13, M1-14, M1-15, M1-16, M1-17, M1-18, M1-19, M1-20, M1-35, M1-36, M1-52 |
| [M1-54](M1-54-fixture-conformance-tests.md) | Cross-OS fixture conformance tests | Testing | MVP | M | M0-11, M0-12, M0-14, M0-15, M1-14, M1-15, M1-16, M1-17, M1-18, M1-19, M1-20, M1-24, M1-25, M1-26, M1-27 |
| [M1-55](M1-55-secrets-redaction-tests.md) | Secrets never reach DTOs or debug output | Testing | MVP | S | M0-11, M1-10, M1-25, M1-26 |
| [M1-56](M1-56-core-coverage-gate.md) | Core coverage measurement and 80% gate | Testing | MVP | S | M1-12, M1-13, M1-14, M1-15, M1-16, M1-17, M1-18, M1-19, M1-20, M1-21, M1-22, M1-23, M1-24, M1-25, M1-26, M1-27, M1-28, M1-29, M1-30, M1-31, M1-33, M1-34, M1-35, M1-36 |
| [M1-57](M1-57-load-benchmark.md) | Load-time benchmark against the performance budgets | Testing | MVP | S | M0-14, M0-15, M1-12, M1-13, M1-14 |
| [M1-58](M1-58-ci-rust-matrix.md) | Cross-OS CI for the Rust workspace | Release and packaging | MVP | M | M1-01, M1-02, M1-37, M1-56 |
| [M1-59](M1-59-m1-gate-review.md) | M1 exit gate review | Testing | MVP | S | M1-40, M1-41, M1-42, M1-43, M1-44, M1-45, M1-46, M1-47, M1-48, M1-53, M1-54, M1-55, M1-56, M1-57, M1-58 |

## M2 — Minimal UI (31 tasks)

| ID | Task | Workstream | Scope | Size | Depends on |
|---|---|---|---|---|---|
| [M2-01](M2-01-vitest-setup.md) | Vitest configuration and fixture-backed DTO snapshots | Testing | MVP | M | M0-12, M1-12, M1-13, M1-14, M1-15, M1-16, M1-17, M1-18, M1-19, M1-24, M1-25, M1-26, M1-27, M1-37 |
| [M2-02](M2-02-theme-tokens.md) | Theme tokens for 20 palettes and the Tailwind v4 theme | Vue shell and design system | MVP | L | M1-04, M0-06 |
| [M2-03](M2-03-ipc-client.md) | Typed IPC client wrapper and test helpers | Vue shell and design system | MVP | S | M1-37, M2-01 |
| [M2-04](M2-04-icons-motion.md) | Lucide icon component, motion utilities and the SF Symbol map | Vue shell and design system | MVP | M | M0-01, M2-02, M2-05 |
| [M2-05](M2-05-app-stores.md) | Prefs, theme, toasts and platform stores | Vue shell and design system | MVP | M | M1-39, M1-40, M2-02, M2-03 |
| [M2-06](M2-06-router-navigation.md) | Router, navigation history and page placeholders | Vue shell and design system | MVP | M | M2-03, M2-05 |
| [M2-07](M2-07-shared-components.md) | Shared components | Vue shell and design system | MVP | L | M2-04, M2-05, M0-06 |
| [M2-08](M2-08-keyboard-shortcuts.md) | Keyboard navigation, global shortcuts and app actions | Vue shell and design system | MVP | M | M2-05, M2-06 |
| [M2-09](M2-09-app-layout.md) | App shell: split view, sidebar with live counts, toolbar, overlays | Vue shell and design system | MVP | M | M2-06, M2-07, M2-08, M2-12, M2-13 |
| [M2-10](M2-10-markdown-pipeline.md) | renderMarkdown, Shiki highlighting and the single v-html component | Security | MVP | M | M1-04, M2-03, M2-05 |
| [M2-11](M2-11-cmd-open-external-url.md) | open_external_url command | Rust data core | MVP | S | M1-33, M1-37 |
| [M2-12](M2-12-window-chrome-navigation-guard.md) | Per-OS window chrome and navigation guard | Tauri shell / OS integration | MVP | M | M1-03 |
| [M2-13](M2-13-app-menu.md) | Native macOS app menu emitting menu-action events | Tauri shell / OS integration | MVP | S | M1-03, M1-37 |
| [M2-14](M2-14-sessions-store.md) | Sessions store | Feature screens | MVP | M | M1-41, M2-03, M2-05, M2-06, M0-06, M1-59 |
| [M2-15](M2-15-sessions-navigation.md) | Projects and sessions navigation list | Feature screens | MVP | M | M2-07, M2-08, M2-09, M2-14, M0-08 |
| [M2-16](M2-16-project-sessions.md) | Project sessions grid and list | Feature screens | MVP | M | M2-07, M2-14, M0-08 |
| [M2-17](M2-17-text-thinking-system-blocks.md) | Text, thinking and system-content blocks | Feature screens | MVP | M | M0-08, M2-04, M2-07, M2-10, M1-59 |
| [M2-18](M2-18-tool-block.md) | Tool-use and tool-result block | Feature screens | MVP | M | M2-04, M2-07, M2-10, M0-08, M1-59 |
| [M2-19](M2-19-edit-diff.md) | Edit diff view | Feature screens | MVP | M | M0-08, M1-42, M2-07, M1-59 |
| [M2-20](M2-20-session-detail.md) | Virtualised conversation view | Feature screens | MVP | L | M2-09, M2-14, M2-17, M2-18, M2-19 |
| [M2-21](M2-21-context-card-timeline.md) | Context-window card and tool timeline | Feature screens | MVP | M | M2-07, M2-14 |
| [M2-22](M2-22-session-toolbar.md) | Session toolbar | Feature screens | MVP | S | M0-08, M2-09, M2-14 |
| [M2-23](M2-23-e2e-harness.md) | Playwright and real-app E2E harness | Testing | MVP | M | M1-38, M2-01, M2-09, M2-31 |
| [M2-24](M2-24-visual-baselines-session.md) | Visual baselines for the session screens | Testing | MVP | M | M2-15, M2-16, M2-17, M2-18, M2-19, M2-20, M2-21, M2-22, M2-23 |
| [M2-25](M2-25-xss-corpus.md) | XSS corpus through renderMarkdown | Testing | MVP | S | M2-01, M2-10 |
| [M2-26](M2-26-malicious-transcript-e2e.md) | Malicious transcript end-to-end test | Testing | MVP | M | M2-12, M2-20, M2-23, M2-25 |
| [M2-27](M2-27-security-config-lint.md) | Security configuration lint | Security | MVP | S | M1-48, M2-12 |
| [M2-28](M2-28-perf-budget-checks.md) | Performance budget checks | Testing | MVP | M | M1-57, M2-20, M2-23 |
| [M2-29](M2-29-ci-frontend.md) | Frontend, security and E2E jobs in CI | Release and packaging | MVP | M | M1-58, M2-01, M2-23, M2-27, M2-28 |
| [M2-30](M2-30-m2-gate-review.md) | M2 exit gate review | Testing | MVP | S | M1-59, M2-11, M2-24, M2-26, M2-29 |
| [M2-31](M2-31-e2e-client-deps.md) | Add the WebDriver client for Linux real-app smoke tests | Release and packaging | MVP | S | M1-02 |

## M3 — Analytics, config, search, tray (53 tasks)

| ID | Task | Workstream | Scope | Size | Depends on |
|---|---|---|---|---|---|
| [M3-01](M3-01-search-overlay.md) | Universal search overlay | Feature screens | MVP | L | M0-07, M1-46, M2-08, M2-09, M2-14, M2-30 |
| [M3-02](M3-02-history-screen.md) | Prompt history screen | Feature screens | MVP | M | M1-43, M2-09, M2-10, M0-06, M2-30 |
| [M3-03](M3-03-plans-screen.md) | Plans list and detail | Feature screens | MVP | M | M1-44, M2-09, M2-10, M2-30 |
| [M3-04](M3-04-memory-screen.md) | Memory list and detail | Feature screens | MVP | M | M1-39, M1-44, M2-09, M2-10, M2-30 |
| [M3-05](M3-05-todos-overview.md) | Cross-project TODOs overview | Feature screens | MVP | M | M1-41, M1-42, M2-09, M2-30 |
| [M3-06](M3-06-session-todos-facets.md) | Session todos and AI summary card | Feature screens | MVP | M | M1-42, M2-20, M2-30 |
| [M3-07](M3-07-file-history-view.md) | Per-session file history viewer | Feature screens | MVP | M | M1-42, M2-19, M2-20, M2-30 |
| [M3-08](M3-08-debug-log-sheet.md) | Debug log viewer | Feature screens | MVP | M | M1-42, M2-22, M2-30 |
| [M3-09](M3-09-cmd-session-export.md) | Session export commands | Rust data core | MVP | M | M1-36, M1-37, M1-41 |
| [M3-10](M3-10-export-options-dialog.md) | Export options dialog and save-as-patch | Feature screens | MVP | S | M2-19, M2-22, M3-09, M2-30 |
| [M3-11](M3-11-config-shared.md) | Shared config list, detail, toolbar, picker and skeleton | Feature screens | MVP | L | M0-09, M1-44, M2-07, M2-09, M2-10, M0-06, M2-30 |
| [M3-12](M3-12-config-commands-skills-styles.md) | Commands, skills and output styles screens (read-only) | Feature screens | MVP | M | M3-11 |
| [M3-13](M3-13-config-hooks-plugins-models.md) | Hooks, plugins and models screens (read-only) | Feature screens | MVP | M | M3-11 |
| [M3-14](M3-14-config-mcp-subagents.md) | MCP servers and sub-agents screens (read-only) | Feature screens | MVP | M | M1-45, M3-11 |
| [M3-15](M3-15-onboarding.md) | Onboarding flow | Feature screens | MVP | M | M1-40, M2-08, M2-09, M2-30 |
| [M3-16](M3-16-settings.md) | Settings screen | Feature screens | MVP | M | M1-39, M2-05, M2-09, M2-30 |
| [M3-17](M3-17-help-about-shortcuts.md) | Help, About and the full shortcut sheet | Feature screens | MVP | S | M1-40, M2-08, M2-09, M2-11, M2-30 |
| [M3-18](M3-18-visual-baselines-mvp.md) | Visual baselines for the remaining MVP screens | Testing | MVP | M | M2-23, M3-01, M3-02, M3-03, M3-04, M3-05, M3-06, M3-07, M3-08, M3-12, M3-13, M3-14, M3-15, M3-16 |
| [M3-19](M3-19-keyboard-e2e.md) | Keyboard navigation end-to-end tests | Testing | MVP | S | M2-20, M2-23, M3-01, M3-15 |
| [M3-20](M3-20-mvp-gate-review.md) | MVP feature gate review | Testing | MVP | S | M2-30, M3-01, M3-02, M3-03, M3-04, M3-05, M3-06, M3-07, M3-08, M3-10, M3-12, M3-13, M3-14, M3-15, M3-16, M3-17, M3-18, M3-19 |
| [M3-30](M3-30-cmd-stats-tray.md) | Analytics and tray commands | Rust data core | Later | S | M1-20, M1-21, M1-37, M1-38 |
| [M3-31](M3-31-analytics-store.md) | Analytics store | Feature screens | Later | M | M0-10, M2-05, M3-30, M0-06, M2-30 |
| [M3-32](M3-32-analytics-dashboard.md) | Analytics dashboard, stat cards and CSV export menu | Feature screens | Later | M | M0-10, M2-06, M2-07, M3-31, M3-36 |
| [M3-33](M3-33-analytics-charts.md) | The six analytics charts | Feature screens | Later | L | M3-32 |
| [M3-34](M3-34-contribution-heatmap.md) | Contribution heatmap | Feature screens | Later | S | M0-10, M3-31 |
| [M3-35](M3-35-chart-xss-tests.md) | XSS tests for chart formatters | Testing | Later | S | M2-25, M3-33 |
| [M3-36](M3-36-analytics-csv-export.md) | Analytics CSV export and PNG save command | Rust data core | Later | S | M1-20, M3-09 |
| [M3-37](M3-37-png-export-spike.md) | Spike: PNG export approach (open risk R-7) | Feature screens | Later | M | M3-33 |
| [M3-38](M3-38-png-export-deps.md) | Add the PNG export dependency | Release and packaging | Later | S | M3-37, M4-08 |
| [M3-39](M3-39-png-export.md) | Analytics PNG export and diff copy-as-image | Feature screens | Later | M | M3-10, M3-32, M3-36, M3-38 |
| [M3-40](M3-40-core-settings-writes.md) | Core writes for settings.json | Rust data core | Later | M | M1-22, M1-25 |
| [M3-41](M3-41-core-mcp-writes.md) | Core writes for MCP servers in ~/.claude.json | Rust data core | Later | M | M1-22, M1-26, M3-40 |
| [M3-42](M3-42-core-file-writes.md) | Core writes for agent, command, skill and output-style files | Rust data core | Later | M | M1-22, M1-24 |
| [M3-43](M3-43-core-deletes.md) | Core history, session and project deletes | Rust data core | Later | S | M1-14, M1-15, M1-16, M1-22, M0-06 |
| [M3-44](M3-44-cmd-config-writes.md) | Config write commands with native confirmations | Rust data core | Later | L | M1-37, M1-38, M3-40, M3-42 |
| [M3-45](M3-45-cmd-mcp-writes.md) | MCP write commands with native confirmations | Rust data core | Later | M | M1-45, M3-41 |
| [M3-46](M3-46-cmd-deletes.md) | Delete commands for history, sessions and projects | Rust data core | Later | S | M1-41, M1-43, M3-43 |
| [M3-47](M3-47-hooks-form.md) | Hook create/edit form and hook actions | Feature screens | Later | M | M0-09, M3-13, M3-44 |
| [M3-48](M3-48-mcp-wizard.md) | MCP setup wizard and server actions | Feature screens | Later | L | M0-09, M3-14, M3-45 |
| [M3-49](M3-49-subagent-form.md) | Sub-agent form, import and export | Feature screens | Later | M | M3-14, M3-44 |
| [M3-50](M3-50-config-write-actions.md) | Write actions on plugins, models, commands, skills and output styles | Feature screens | Later | M | M3-12, M3-13, M3-44 |
| [M3-51](M3-51-delete-ui.md) | Delete actions for history, sessions and projects | Feature screens | Later | S | M2-15, M2-16, M3-02, M3-46 |
| [M3-52](M3-52-platform-launch.md) | Platform execution of launch actions | Tauri shell / OS integration | Later | M | M1-03, M1-32, M3-54 |
| [M3-53](M3-53-cmd-launch.md) | Editor, terminal and reveal commands | Rust data core | Later | M | M1-41, M1-44, M2-11, M3-52 |
| [M3-54](M3-54-editor-url-scheme-verification.md) | Verify editor URL schemes per OS | Testing | Later | S | — |
| [M3-55](M3-55-launcher-ui.md) | Editor, terminal and reveal buttons | Feature screens | Later | S | M2-18, M2-22, M3-11, M3-48, M3-53 |
| [M3-56](M3-56-tray-shell.md) | Tray icon and tray window | Tauri shell / OS integration | Later | M | M1-48, M3-30, M3-57 |
| [M3-57](M3-57-tray-entry.md) | Separate frontend entry for the tray window | Vue shell and design system | Later | S | M1-04, M2-05 |
| [M3-58](M3-58-tray-view.md) | Tray view and Menu Bar settings tab | Feature screens | Later | M | M3-16, M3-30, M3-57 |
| [M3-59](M3-59-autostart-capability.md) | Grant launch-at-login permissions | Security | Later | S | M1-48, M2-27 |
| [M3-60](M3-60-launch-at-login.md) | Launch-at-login toggle | Feature screens | Later | S | M3-58, M3-59 |
| [M3-61](M3-61-visual-baselines-later.md) | Visual baselines for later-scope screens | Testing | Later | M | M2-23, M3-32, M3-33, M3-34, M3-47, M3-48, M3-49, M3-58 |
| [M3-62](M3-62-later-gate-review.md) | Later-scope gate review | Testing | Later | S | M3-35, M3-39, M3-50, M3-51, M3-55, M3-56, M3-60, M3-61, M4-14 |

## M4 — Packaging and signing (16 tasks)

| ID | Task | Workstream | Scope | Size | Depends on |
|---|---|---|---|---|---|
| [M4-01](M4-01-bundle-config.md) | Per-OS bundle configuration | Release and packaging | MVP | M | M0-02, M1-03, M2-12 |
| [M4-02](M4-02-updater-key.md) | Generate and store the updater signing key | Release and packaging | MVP | S | — |
| [M4-03](M4-03-updater.md) | Updater plugin wiring | Tauri shell / OS integration | MVP | M | M1-48, M1-49, M2-12, M2-13, M4-02 |
| [M4-04](M4-04-cmd-update.md) | Update commands | Rust data core | MVP | S | M1-37, M4-03 |
| [M4-05](M4-05-update-toast.md) | Check-for-updates flow in the UI | Feature screens | MVP | S | M2-05, M2-08, M4-04 |
| [M4-06](M4-06-release-workflow.md) | Multi-platform release workflow | Release and packaging | MVP | L | M1-58, M4-01, M4-02, M4-03, M3-20 |
| [M4-07](M4-07-version-bump.md) | Calendar-version bump workflow | Release and packaging | MVP | S | M1-02, M1-03 |
| [M4-08](M4-08-dev-setup-scripts.md) | Per-OS developer setup scripts | Release and packaging | MVP | M | M1-02, M2-31 |
| [M4-09](M4-09-install-docs.md) | Install guide for unsigned builds | Docs | MVP | S | M4-01 |
| [M4-10](M4-10-release-checklist.md) | Release checklist | Release and packaging | MVP | S | M4-06 |
| [M4-11](M4-11-release-dry-run.md) | Dry-run a tagged pre-release | Release and packaging | MVP | S | M4-06, M4-07, M4-10 |
| [M4-12](M4-12-platform-verification.md) | Install and smoke-test on target platforms | Testing | MVP | M | M4-09, M4-11 |
| [M4-13](M4-13-readme-final.md) | Final README for the MVP release | Docs | MVP | S | M0-04, M2-24, M3-18, M4-09, M4-12 |
| [M4-14](M4-14-mvp-signoff.md) | MVP release sign-off | Testing | MVP | S | M3-20, M4-10, M4-12 |
| [M4-15](M4-15-oracle-harness-retirement.md) | Retire the Swift golden harness (conditional) | Legacy parity extraction | MVP | S | M4-14 |
| [M4-16](M4-16-oracle-workflow-retirement.md) | Retire the golden-oracle workflow | Release and packaging | MVP | S | M4-15 |
