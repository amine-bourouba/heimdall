# Heimdall port plan

Heimdall ports the Swift reference app, a native macOS SwiftUI companion app for Claude Code, to Windows, Linux and macOS on Tauri 2, Rust, Vue 3, TypeScript and Tailwind CSS v4. This plan is the entry point for every agent working on the port.

- **Source of truth for behaviour:** the Swift app at `$SWIFT_REF` = `C:\Users\pc\Desktop\tech\poirot` on this machine (see ADR 0001 for upstream provenance). It is read-only.
- **Companion documents:** [ADRs](adr/) (0001–0011), [parity matrix](parity-matrix.md), [IPC contract](ipc-contract.md), [task backlog](tasks/README.md) (174 tasks), [progress log](progress-log.md).
- **Status:** planning complete and every decision answered (§2.2, 2026-09-25). Work can start with M0-01 and M0-03.

## 1. How to use this plan

1. Pick a task from [docs/tasks/](tasks/README.md) whose dependencies have all landed.
2. Read only that task file and the files it lists as Inputs. Task files are self-contained.
3. Deliver its Outputs, prove each acceptance criterion, get the named reviewers, then append to the [progress log](progress-log.md).
4. After editing any task file, run `python docs/tasks/check_backlog.py --swift-ref <path-to-swift-reference-checkout>`. It fails on missing fields, unknown dependencies, dependency cycles, unordered writes to the same file, missing security review, cross-workstream writes, and Swift citations that do not exist.

## 2. Decisions

### 2.1 Settled by the user

These answers override the planning brief wherever they conflict.

| # | Topic | Answer | Consequence in this plan |
|---|---|---|---|
| 1 | Repo layout | Separate new project, pushed to GitHub later; Swift source stays at `$SWIFT_REF` as a read-only reference | No Swift tree, Swift CI or path filtering in this repo (ADR 0001) |
| 2 | Name and release repo | Heimdall; releases from https://github.com/amine-bourouba/heimdall; check Linux package-name collisions | Collision found; see D2 and ADR 0011 |
| 3 | Other CLI providers | Focus on Claude Code only for now | Provider abstraction removed (ADR 0008) |
| 4 | Usage Limits | Leave it out | Out of scope (parity row X1) |
| 5 | TS binding generator | tauri-specta | ADR 0004 |
| 6 | JSON key order on writes | Preserve the original order | ADR 0009 |
| 7 | Signing and channels | Unsigned Windows and macOS builds; do not reuse upstream Apple secrets; still generate a Tauri updater key; GitHub Releases only | ADR 0011, tasks M4-01, M4-02, M4-06, M4-09 |
| 8 | macOS architectures | Apple Silicon only | `aarch64-apple-darwin` only |
| 9 | Windows/Linux ARM64 | Not at launch; add later | x86_64 only at launch |

### 2.2 Follow-up decisions (answered 2026-09-25)

The user accepted the recommendation for D1–D3 and resolved D4 by moving the folder. D5 (raised by M1-13) was answered on 2026-09-26. No decision is pending.

| ID | Decision | Answer | Why it mattered | Unblocked |
|---|---|---|---|---|
| D1 | App identifier (reverse-DNS) | `io.github.amine-bourouba.heimdall` | The OS keys the preferences folder and window state by identifier; changing it after release orphans user settings. It must contain only letters, digits, `.` and `-`, and must not end in `.app`. | M0-02, M1-01, M1-03, M4-01 |
| D2 | Linux package and binary name | `heimdall-desktop` (display name stays "Heimdall") | `heimdall` is already taken by the Samsung firmware flasher: Arch `extra/heimdall`, Fedora `heimdall`, and Debian/Ubuntu `heimdall-flash`, which installs `/usr/bin/heimdall` (checked 2026-09-25). | M0-02, M1-01, M4-01 |
| D3 | How to run the Swift reference for golden outputs | GitHub-hosted macOS runners (tasks M1-50 to M1-52); no local Mac needed | the Swift reference app only builds on macOS. Without this, parity rests on PoirotTests-derived tests alone and the golden-parity tasks (M1-50 to M1-53) are dropped. | M1-50 |
| D4 | Where the Heimdall folder lives | Moved out of the `hydra` repository to `C:\Users\pc\Desktop\tech\Heimdall`; not yet a git repository | It sat inside another project's repository (note N1). | M0-03 |
| D5 | Per-message token usage: the Swift reference app's first record, or the last (final) record | Last record (answered 2026-09-26) | Session token totals are about 3× too low on real transcripts under the Swift reference app's rule (C-25). The two carried-over tests `parse_usage_deduplicatedPerGroup` and `parseSummary_sumsTokensPerGroup` now expect last-record totals, and Heimdall's totals differ from the Swift reference app's. | transcript/full.rs, transcript/summary.rs |

### 2.3 Plan-level decisions

These are my own calls. Each is reversible; override any of them by editing this table and the tasks it names.

| ID | Decision | Rationale | Tasks |
|---|---|---|---|
| P-1 | Line diffs, JSON pretty-printing and system-content parsing live in Rust; analytics range filtering and history date buckets live in Pinia stores | Keeps all Claude Code parsing in Rust (locked data flow) while matching the brief's test carry-over (analytics view-model tests go to Vitest) | M1-12, M1-35, M3-02, M3-31 |
| P-2 | Config screens ship read-only in the MVP; every write is Later | Writes carry the Windows file-locking and code-execution risks; universal search still needs somewhere to navigate | M3-11 to M3-14, M3-40 to M3-50 |
| P-3 | Navigation uses Vue Router hash history with no wrapper store: `history.state.forward` gives canGoForward, `router.replace` covers "skip re-push on hydration", and native history already trims forward entries on a new push | Fewest moving parts; the Swift semantics fall out of browser history | M2-06 |
| P-4 | Window chrome: macOS overlay title bar with traffic lights; Windows and Linux keep native decorations; back/forward sit in the in-app toolbar on every OS | Custom decorations on Windows/Linux add drag, snap and accessibility work with no user benefit | M2-09, M2-12 |
| P-5 | MCP HTTP/SSE status stays "URL is well-formed" exactly as the Swift reference app does it; a real probe is a possible later enhancement, not scheduled | The brief asks for an explicit decision instead of a silent change | M1-28 |
| P-6 | Settings, About and Help become in-app views instead of separate OS windows | One window means one main capability file and simpler navigation | M3-16, M3-17 |
| P-7 | The shortcut sheet lists every shortcut from one registry (the Swift reference app shows 9 of 11) | A generated list cannot drift from the real bindings | M3-17 |
| P-8 | E2E: Playwright against the built frontend with fixture-backed IPC mocks on all OSes (WebKit engine stands in for WKWebView and WebKitGTK); real-app smoke on Windows via Playwright over CDP to WebView2 and on Linux via tauri-driver with WebdriverIO | Playwright cannot drive tauri-driver (it is a WebDriver server), and macOS has no WebDriver for WKWebView [A-10]. WebdriverIO is a new dev dependency, added by M2-31 once it passes the audit gate. | M2-23, M2-31 |
| P-9 | Windows ships an NSIS installer only (per-user, no admin); MSI can be added later | The updater supports NSIS; MSI needs WiX and admin rights | M4-01 |
| P-10 | Version format `YY.M.BUILD` without zero padding (for example `26.8.5`) | the Swift reference app's `26.08.5` is not valid semver, which Tauri, Cargo and npm require | M1-02, M4-07 |
| P-11 | macOS minimum version 13.0 | the Swift reference app needed 15 for SwiftUI APIs; the webview port does not; 13 keeps recent WKWebView CSS support | M4-01 |
| P-12 | Single main window in the MVP; the Swift reference app's multi-window support is hidden | Multi-window multiplies state and capability work; rarely used | parity A2 |
| P-13 | Playwright baselines are committed for Linux only, generated in CI | Linux WebKitGTK is the weakest engine (risk R-2); one committed set avoids OS font noise | M2-24, M3-18, M3-61 |
| P-14 | The Tauri updater (`latest.json` on GitHub Releases) replaces the Swift reference app's GitHub-API update check; UpdateCheckerTests are dropped | One network path instead of two, with signature verification | M4-03, M4-04 |
| P-15 | Tray: a small webview window on Windows and macOS; a native menu with recent sessions on Linux | Tauri does not deliver tray click events on Linux; GNOME needs the AppIndicator extension | M3-56 |
| P-16 | Shiki output is converted to classes (`transformerStyleToClass`) and its CSS is injected as a constructable stylesheet; the CSP gets no `unsafe-inline` | Keeps the brief's CSP unchanged | M2-10, ADR 0010 |
| P-17 | No logging framework in the MVP; diagnostics travel as `AppError` to the UI and core denies stdout/stderr prints | Nothing to redact if nothing is logged; add a redacting logger when a real need appears | M1-05, ADR 0005 |
| P-18 | Analytics CSV is generated in Rust, where the file is written; its tests move to Rust | Avoids passing file contents back across IPC | M3-36 |

## 3. Corrections, assumptions and notes

### 3.1 Corrections from planning-time verification

Every cited path in this plan and the task files was checked against `$SWIFT_REF` at `a29dd6e`; the checker (§1) re-verifies Swift paths and line ranges in task files. The code differed from the brief as follows. The code wins.

| ID | Brief said | Code shows |
|---|---|---|
| C-1 | Parse-failure-then-overwrite at `SettingsWriter.swift:123-128` and `:359-365` | Also at `setProjectModel` (`:65-68`) and `loadSettingsDictFor` (`:336-338`, used by saveHook, deleteHook and importHooksFromJSON). updateMCPServer and removeMCPServer return early and are safe. |
| C-2 | — | A project-scope settings write with no project path silently writes the **global** file (`SettingsWriter.swift:318-326`). |
| C-3 | updateMCPServer drops type/url/headers | Confirmed (`:150-177`). Also, hook serialisation rebuilds handlers from known fields only and drops unknown keys (`:296-314`). |
| C-4 | EditorLauncher passes an unchecked path at `:34` | Also at `:11`, `:13` and `:15` (VS Code/Cursor `--goto`, Xcode, Zed). |
| C-5 | `TerminalLauncher.run` has no callers | It is called only by `launch()`, which has no callers; `open()` is used (`SessionToolbarContent.swift:115`, `ToolBlockView.swift:350`). |
| C-6 | "Prefer the `cwd` recorded in each transcript" | Swift never reads `cwd` (no match in `Poirot/Sources`), so this is new behaviour. `decodeProjectPath` (`:314-353`) returns only the last component, used as the display name; the path fallback at `:98` is a plain `-` → `/` replace. |
| C-7 | — | `sessions-index.json` is read with different keys: `TodosOverviewView.swift:372` uses top-level `originalPath`; `SessionLoader.swift:239-240` uses `entries[0].projectPath` or top-level `projectPath`. Fixtures must settle which Claude Code writes. |
| C-8 | Views ~16,400, Services ~3,500, Models ~1,800, App ~1,400, Theme ~600, Utilities ~500 lines | Raw `wc -l` at `a29dd6e`: Views 18,345, Services 4,122, Models 2,020, App 1,594, Theme 653, Utilities 565, plus a `Protocols/` folder (65) the brief omits. Tests total 12,376, matching the brief. |
| C-9 | 24 colour tokens per theme | 23 stored tokens per palette plus a derived `accentDim`. There is also a 4-step `IconSize` scale. |
| C-10 | Shortcut sheet shows 8 shortcuts | It shows 9 rows (`ShortcutHelpView.swift:57-70`). |
| C-11 | Six loaders injected via environment keys | Six environment keys: five loaders plus the provider (`Services/Environment+Services.swift`). |
| C-12 | — | `unit-tests.yml` still skips `ScreenshotTests_Usage`, a suite that no longer exists. |
| C-13 | "macOS share sheet" as a parity example | the Swift reference app has no share sheet (no `NSSharingService` or `ShareLink`). |
| C-14 | Inventory of screens | Missing from the brief: multi-window support with a Dock "New Window" item (`PoirotApp.swift:12-22`), Reveal in Finder (3 call sites), copy diff as image (`EditDiffView.swift:228`). Added as parity rows A2, C17, C18. |
| C-15 | Keep the `YY.MM.BUILD` scheme (`26.08.5`) | `26.08.5` is not valid semver; see P-10. |
| C-16 | Atomic-rename handling at `FileWatcher.swift:58-67` | The event mask (`.write, .rename, .delete, .extend`) is at `:29`; `:58-67` is the debounce. |
| C-17 | — | Linux name collision confirmed; see D2. |
| C-18 | Notarised macOS build, Azure Trusted Signing, universal macOS, Homebrew | Superseded by user answers 7–9: unsigned (ad-hoc on macOS), Apple Silicon only, GitHub Releases only. The brief's M4 exit "signed/notarized macOS build" becomes "ad-hoc-signed macOS build with signed updater artifacts". |
| C-19 | Six symbol-effect categories to replace | In use: bounce, breathe, rotate and content-transition replace, plus two multi-line call sites; pulse, variableColor and wiggle may be unused. M2-04 inventories them. |
| C-20 | — | The update check runs only from the "Check for Updates…" menu item, never automatically (`PoirotApp.swift:55-79`). |
| C-21 | — | the Swift reference app is English-only (`Localizable.xcstrings`, 401 strings), so localisation adds no parity work (row C15). |
| C-22 | Four watcher call sites | 13 watcher start sites in 8 files: the screens for hooks, MCP servers, sub-agents, history, plans and TODOs each watch their own files, and the sub-agents screen creates `~/.claude/agents` just to watch it (found by M0-05; parity row G7). |
| C-23 | — | History delete finds its line by an index embedded in the entry id and rewrites the whole file, so a line Claude Code appends in between is lost (`Services/HistoryLoader.swift:60-105`; found by M0-05). |
| C-24 | — | Current Claude Code writes tool results between the records of one assistant message, so the full parse splits that message into several groups and counts its usage once per group, while the summary tier counts it once. The two tiers disagree in the Swift reference app. Heimdall counts usage once per message id in both tiers (`TranscriptParser.swift:98-108`; found by M1-13 on the Windows fixtures). |
| C-25 | — | Both tiers take a message's usage from its first record, but streamed records carry growing `output_tokens` and only the last one is final. On the Windows fixtures this counts 118,515 output tokens where the last records sum to 373,792. Heimdall takes each message's last record (D5). |
| C-26 | — | StatsComputer adds a message's usage once for every streamed record of it (`StatsComputer.swift:218-245`), so recomputed analytics multiply input and cache-read tokens by the number of records per message. Heimdall's recomputation counts each message id once, from its last record, as D5 decided for sessions (found by M1-20). |
| C-27 | — | The Models screen always shows the catalog's "Opus 5" as the default and never reads the `model` in `settings.json`, and "Set as default" writes a display name such as `Opus 5` into Claude Code's `model` field, which expects a model id or alias (`ModelsListView.swift:59`, `SettingsWriter.swift:52-56`). Heimdall's view reads the configured model (M1-27); the write must store an id or alias (M3-40, found by M1-27). |

### 3.2 Assumptions (unverified until the named task confirms them)

| ID | Assumption | Verified by |
|---|---|---|
| A-1 | Claude Code on Windows writes the same `~/.claude` layout under `%USERPROFILE%` (the brief says this was checked on a real Windows 11 machine) | M0-14 |
| A-2 | Whether Claude Code honours a config-dir override such as `CLAUDE_CONFIG_DIR`; the Swift reference app never reads one | M1-06 |
| A-3 | Claude Code's own JSON writer uses 2-space indentation and does not sort keys | M1-22 (formatting is detected, not assumed) |
| A-4 | MarkdownUI never executes HTML today, so a webview Markdown renderer is a new XSS surface | ADR 0010 treats it as new |
| A-5 | The CSP and capability model behave the same on WebView2, WebKitGTK and WKWebView | M2-26, M4-12 |
| A-6 | Search skipping custom memory folders, and model search covering only the curated catalog, are intentional | Kept at parity (rows S10, S15); ask upstream if it matters |
| A-7 | the Swift reference app's single-folder watcher may miss new transcripts inside project subfolders (needs a Mac to confirm) | Moot: Heimdall watches recursively (M1-47) |
| A-8 | Preferences land in `%APPDATA%\<id>` (Windows), `~/Library/Application Support/<id>` (macOS), `~/.config/<id>` (Linux) | M1-39 |
| A-9 | `CFFIXED_USER_HOME` redirects the home folder for the Swift reference app's config loaders in the golden harness | M1-50 |
| A-10 | tauri-driver has no macOS support because WKWebView has no WebDriver | M2-23 |
| A-11 | `backdrop-filter` is an acceptable stand-in for native glass; WebKitGTK support is weaker | M2-02, M2-30 visual review |
| A-12 | Minimum Linux support: Ubuntu 22.04+, Debian 12+, current Fedora, current Arch | M4-12 |
| A-13 | The Flathub runtime provides WebKitGTK 4.1 (matters only for a later Flatpak) | M4-12 |
| A-14 | An ad-hoc-signed Apple Silicon app runs after the Gatekeeper override, and the Tauri updater can replace it in place | M4-11, M4-12 |
| A-15 | The pinned Tauri 2 version has handlers to refuse navigation and new windows | M2-12 |
| A-16 | tauri-specta, still pre-release, stays usable for the project's life | M1-37 records its status; fallback in risk R-10 |
| A-17 | `vscode://file/…`, `cursor://file/…` and `zed://file/…` open a file at a line on all three OSes | M3-54 |
| A-18 | Whether Claude Code on Windows writes CRLF or LF transcripts | **Answered by M0-14: LF.** CRLF is still tested through a converted copy |
| A-19 | A WSL2 capture is representative of native Linux | M0-15 records which was used |

### 3.3 Notes

- **N1.** Resolved: the Heimdall folder moved out of the `hydra` repository to `C:\Users\pc\Desktop\tech\Heimdall` on 2026-09-25 (D4). M0-03 runs `git init` there.
- **N2.** Heimdall needs its own icon. Until the user supplies one, M1-03 uses Tauri's placeholder icon set. The Swift reference app's icon is not reused, to avoid confusing the two apps.
- **N3.** License: MIT. The Swift reference app's notice (`Copyright (c) 2026 Leonardo Cardoso`) is preserved alongside Heimdall's (M0-04).

## 4. Locked stack

These choices are settled and not revisited: Tauri 2 with a Rust backend; Vue 3, TypeScript and Tailwind CSS v4 built with Vite; Pinia and Vue Router; ECharts through vue-echarts; markdown-it, DOMPurify and Shiki; Lucide icons; TanStack Virtual; Vitest with Vue Test Utils, and Playwright; theming through CSS custom properties per palette switched by a `data-*` attribute on the root element; `notify` with a debouncer crate, `sysinfo`, `serde`/`serde_json`; Tauri plugins opener, autostart, clipboard-manager, dialog, store and updater, plus the built-in tray; the Tauri path API or `dirs`; `tauri-action` for builds. All parsing happens in Rust and the webview receives ready-to-render data.

## 5. Execution model

### 5.1 Workstreams, owners and reviewers

| Workstream | Owner | Reviewers | Owns |
|---|---|---|---|
| Rust data core | rust-engineer | architect-reviewer (module boundaries); security-auditor (paths, file replacement, IPC) | `crates/heimdall-core/src/`, `src-tauri/src/commands/`, `state.rs`, `prefs.rs`, `watcher.rs`, `events.rs`, `src/ipc/bindings.ts` (generated) |
| Tauri shell / OS integration | rust-engineer | security-engineer (capability scoping); deployment-engineer (packaging) | `src-tauri/src/{lib,main,tray,menu,window,updater}.rs`, `platform/`, `build.rs`, `tauri.conf.json`, icons |
| Vue shell and design system | vue-expert, with ui-designer for tokens | architect-reviewer; accessibility-tester for motion, contrast, keyboard | `src/` outside `features/`, `lib/markdown/`, `lib/export/` and `test/`; Vite, TS and ESLint config; `docs/design/` |
| Feature screens | frontend-developer, with typescript-pro for typed IPC | code-reviewer; accessibility-tester | `src/features/`, `src/lib/export/` |
| Legacy parity extraction | legacy-modernizer, consulting swift-expert (read-only) | qa-expert | `docs/spec/`, `tools/golden-oracle/` (written by swift-expert once the extraction tasks are done) |
| Testing | test-automator | qa-expert | `fixtures/`, `e2e/`, `tests/`, `src/test/`, core `tests/` and `examples/`, `scripts/` (except those listed elsewhere), `docs/parity-matrix.md` (gate reviews only) |
| Security | security-engineer | security-auditor (sign-off gate, never implementer) | `src-tauri/capabilities/`, `src/lib/markdown/`, `scripts/lint-security-config.*` |
| Release and packaging | deployment-engineer, build-engineer; dependency-manager for manifests | architect-reviewer and security-auditor jointly | `.github/`, manifests and lockfiles, `deny.toml`, `rust-toolchain.toml`, per-OS `tauri.*.conf.json`, `scripts/setup-*`, `docs/release/` |
| Docs | documentation-engineer, technical-writer (write-up only, never deciding) | architect-reviewer or code-reviewer | `AI_INSTRUCTIONS.md`, `CLAUDE.md`, `README.md`, `LICENSE`, `NOTICE`, `docs/adr/`, `docs/install.md` |

The checker's `OWNERSHIP` table (§1) encodes this and rejects cross-workstream writes.

### 5.2 Review gates

- IPC surface, capabilities, or any write to Claude Code's files: **security-auditor** sign-off.
- Module boundaries or cross-workstream contracts: **architect-reviewer**.
- Keyboard navigation, contrast or motion: **accessibility-tester**.
- Release pipeline: **architect-reviewer and security-auditor** together.
- Everything else: **code-reviewer** at minimum.

### 5.3 File ownership

Each file belongs to one workstream (table in §5.1). A task writes only files of its own workstream. Two tasks that write the same file are always ordered by dependencies; the checker enforces this. `docs/progress-log.md` is append-only and exempt.

### 5.4 Placeholder rule

A skeleton task may create placeholder files that later tasks own, so wiring points exist before the code does: M1-03 (src-tauri modules), M1-05 (core modules), M2-06 (feature pages and availability flags), M2-09, M2-20 and M2-22 (mount points). A placeholder holds only a doc comment or an empty component plus the minimal signature the wiring needs. The owning task depends on the skeleton and replaces the placeholder completely.

### 5.5 Ordering and parallelism

- **Critical path:** M1-01 → M1-05 → models (M1-08 to M1-11) → M1-37 contract freeze → M1-38 → read commands and watcher → M1-59 gate → M2 feature screens → M2-30 → M3 MVP screens → M3-20 → M4-06 release workflow → M4-11 dry run → M4-14 sign-off.
- **Contract first:** M1-37 freezes every command and event as compiled stubs and commits the generated bindings. After it lands, Rust implementation (M1-38 to M1-47) and the Vue shell (M2-01 to M2-10) proceed in parallel against it.
- **Parallel lanes:** parsers and loaders for distinct data (M1-12 to M1-36) touch separate files and run in parallel once M1-05 lands. Config reads, session reads and search are independent. Later-scope M3 work (analytics, writes, launchers, tray) is parallel to the MVP path and does not block M4.
- **Entry gates:** M2 feature-screen tasks depend on the M1 gate (M1-59); M3 feature-screen tasks depend on the M2 gate (M2-30); the release workflow depends on the M3 MVP gate (M3-20). Shell, design-system, data-layer and packaging-prep tasks may start as soon as their own dependencies land.
- **Worktrees:** one git worktree per active workstream branch, so Rust work in `crates/` and `src-tauri/` never collides with Vue work in `src/`.

### 5.6 Security review triggers

The checker requires security-auditor on any task writing: `src-tauri/src/commands/`, `src-tauri/capabilities/`, `src-tauri/build.rs`, `src-tauri/tauri.conf.json`, `src/ipc/bindings.ts`, `crates/heimdall-core/src/config/write/`, `data/history_write.rs`, `projects/delete.rs`, `models/secret.rs`, `os_integration/`, `src/lib/markdown/`, or `.github/workflows/release.yml`.

### 5.7 Progress log protocol

Every task appends one entry to [progress-log.md](progress-log.md): task id, date, agent, files touched, evidence per acceptance criterion, follow-ups. Agents never edit another entry. Gate-review tasks copy statuses from the log into the parity matrix.

### 5.8 Dev identifiers, ports and worktrees

- Release identifier: `io.github.amine-bourouba.heimdall` (D1). Development builds use `io.github.amine-bourouba.heimdall.dev`; worktree N uses `io.github.amine-bourouba.heimdall.dev<N>`, passed with `tauri dev --config '{"identifier":"io.github.amine-bourouba.heimdall.dev<N>"}'`, so each worktree keeps separate preferences.
- The Vite port is `1420 + N`, read from `HEIMDALL_DEV_PORT` (M1-04).
- Debug builds honour `HEIMDALL_CLAUDE_HOME` to point at a fixture tree; release builds ignore it (M1-38).

## 6. Target repository layout

```text
Heimdall/
├─ AI_INSTRUCTIONS.md, CLAUDE.md, README.md, LICENSE, NOTICE
├─ Cargo.toml, Cargo.lock, rust-toolchain.toml, deny.toml      # Rust workspace
├─ package.json, package-lock.json, vite.config.ts, tsconfig*.json, eslint.config.js
├─ vitest.config.ts, playwright.config.ts, index.html, tray.html
├─ crates/heimdall-core/                                        # no tauri dependency; takes the Claude home as a parameter
│  ├─ src/{lib,error,paths,jsonl,classify,diff,mcp_status}.rs
│  ├─ src/models/ (+ config/, catalog/)    src/transcript/    src/projects/    src/data/
│  ├─ src/config/{read,write}/    src/search/    src/os_integration/    src/export/
│  ├─ tests/ (golden_parity, fixtures_conformance, secrets_redaction)    examples/
├─ src-tauri/
│  ├─ src/{main,lib,state,prefs,watcher,events,tray,menu,window,updater}.rs
│  ├─ src/platform/{mod,windows,macos,linux}.rs    src/commands/*.rs    tests/
│  ├─ build.rs, tauri.conf.json, tauri.{windows,linux,macos}.conf.json, capabilities/{main,tray}.json, icons/
├─ src/                                                         # Vue: rendering and view state only
│  ├─ main.ts, App.vue, ipc/{bindings,client,testing}.ts, router/, stores/, composables/, styles/, theme/
│  ├─ components/, layout/, lib/{markdown,export}/, test/, tray/
│  └─ features/{session,todos,history,plans,memory,analytics,settings,help,about,search,onboarding,tray,update,
│               config/{commands,skills,output-styles,hooks,mcp,plugins,models,subagents,shared}}/
├─ fixtures/{synthetic,screenshot,windows,linux,golden,dto,security}/
├─ e2e/{support,smoke,visual,security,perf,keyboard}/    tests/security/
├─ scripts/    tools/golden-oracle/ (until retired)    .github/{workflows,dependabot.yml}
└─ docs/{port-plan.md, parity-matrix.md, ipc-contract.md, progress-log.md, adr/, tasks/, spec/, design/, spikes/, release/, security/}
```

Feature page files created by M2-06: `session/SessionsPage.vue`, `todos/TodosPage.vue`, `history/HistoryPage.vue`, `plans/PlansPage.vue`, `memory/MemoryPage.vue`, `analytics/AnalyticsPage.vue`, `settings/SettingsPage.vue`, `help/HelpPage.vue`, `about/AboutDialog.vue`, and `config/<kind>/<Kind>Page.vue` for commands, skills, output-styles, hooks, mcp (`McpServersPage.vue`), plugins, models and subagents (`SubAgentsPage.vue`), each with an `index.ts` availability flag. No new top-level folder may differ from an existing one only by letter case.

## 7. Milestones and gates

Sizing: M0 S · M1 XL (critical path; M2–M4 depend on it) · M2 M · M3 L (parallel once M1 is stable) · M4 M. The most schedule buffer belongs to M0→M1: the real-machine captures (M0-14, M0-15) are the earliest blockers.

### M0 — Inventory (S)

- **Entry:** none.
- **Exit, all OSes:** an exhaustive written list of every `~/.claude` and `~/.claude.json` path read, written or watched (M0-05); the test carry-over map (M0-06); behaviour specs (M0-07 to M0-10); synthetic and screenshot fixture trees (M0-11, M0-12).
- **Exit, Windows:** a scrubbed real Windows tree with its MANIFEST, CRLF status recorded (M0-14).
- **Exit, Linux:** a scrubbed real Linux tree with its MANIFEST, native or WSL2 recorded (M0-15).
- **Exit, macOS:** the synthetic and screenshot trees stand in (they come from macOS-authored tests); a real macOS capture is optional.
- **Also:** rules file (M0-01), identity decisions recorded (M0-02), repository bootstrap (M0-03, M0-04), upstream-sync procedure (M0-16).

### M1 — Rust data layer (XL)

- **Entry:** M0 exit met (fixtures exist).
- **Exit, per OS (windows-latest, ubuntu-22.04, macos-latest in CI):** `cargo test --workspace` passes, including every PoirotTests-derived Rust test mapped in `docs/spec/test-carryover.md`; the fixture conformance suite passes on the synthetic, screenshot, Windows (CRLF, drive-letter `cwd`) and Linux trees with no folder-name decoding used where a `cwd` exists (M1-54); the watcher integration test fires on each OS (M1-47); golden parity passes where oracle outputs exist (M1-53; dropped if D3 is declined).
- **Exit, cross-cutting:** heimdall-core coverage ≥ 80% lines (M1-56); `cargo tree -p heimdall-core` has no tauri (M1-58); the IPC contract is frozen with committed bindings (M1-37); capabilities are in place (M1-48, M1-49); benchmarks recorded (M1-57).
- **Gate review:** M1-59. M1-32 (launcher argv) is Later scope and not part of the gate.

### M2 — Minimal UI (M)

- **Entry:** M1 exit met for feature-screen tasks. Shell and design-system tasks (M2-01 to M2-13) may start after M1-37.
- **Exit, Windows and Linux:** the session browser and conversation view (Markdown, tool, thinking and system blocks, edit diffs, context card, timeline, toolbar) match the Swift app on the screenshot fixture set by side-by-side review against the oracle's reference screenshots (M2-30); the real-app smoke passes (M2-23); the malicious-transcript test passes against the real app (M2-26).
- **Exit, macOS:** builds and passes the mocked suites; macOS is the oracle, not the bar.
- **Exit, cross-cutting:** Linux Playwright baselines committed (M2-24); XSS corpus green (M2-25); security lint green (M2-27); performance budgets measured (M2-28); frontend CI green on all OSes (M2-29).
- **Gate review:** M2-30.

### M3 — Analytics, config, search, tray (L)

- **Entry:** M2 exit met for feature screens.
- **Exit, MVP scope, each OS:** every MVP parity row is `AP`, or `DG`/`HD`/`RP` with its one recorded outcome (M3-20). Required tasks: M3-01 to M3-19.
- **Exit, later scope, each OS:** the same for later rows (M3-62). Later scope does not block M4.

### M4 — Packaging (M)

- **Entry:** M3 MVP exit met (the release workflow depends on M3-20); packaging preparation (M4-01 to M4-05, M4-07 to M4-09) may start earlier.
- **Exit, Windows:** an unsigned NSIS installer (x86_64) from CI installs and updates from the previous version (M4-06, M4-11, M4-12).
- **Exit, Linux:** deb, rpm and AppImage (x86_64) from a pinned ubuntu-22.04 runner install on the target distros; SHA-256 sums published (M4-06, M4-12).
- **Exit, macOS:** an ad-hoc-signed Apple Silicon app and DMG from CI install with the documented Gatekeeper override and update in place [A-14].
- **Exit, cross-cutting:** updater artifacts are signed and verified by a real update on each OS; install guide published (M4-09); MVP sign-off with security-auditor evidence for every ADR 0010 control (M4-14).
- **Swift oracle retirement** (M4-15, M4-16) happens only when every MVP parity row is `AP` on Windows and Linux **and** every parser-related PoirotTests row is covered by a passing Rust test. It is a condition, not a date.

## 8. Testing strategy

- **Carry-over.** `docs/spec/test-carryover.md` (M0-06) maps every Swift test. Services, parsers, loaders and models (34 files, about 6,000 lines) become Rust unit tests and are the spec for the data layer. App-state tests become Vitest specs against Pinia stores. `MenuBarStateTests` becomes a Rust test (M1-21). Analytics view-model tests become Vitest; analytics CSV tests become Rust (P-18). Pure utilities follow their code (P-1). Dropped: ClaudeCodeProviderTests (ADR 0008), HighlightedTextTests' AttributedString assertions, `*LoadingMock.swift`, UpdateCheckerTests (P-14).
- **Screenshots.** the Swift reference app's 17 screenshot suites are skipped in its CI and have no committed baselines. Heimdall replaces them with Playwright visual tests and commits Linux baselines (P-13): M2-24, M3-18, M3-61.
- **Fixtures.** Inline Swift fixtures become files (M0-11); `ScreenshotData.swift` becomes a fixture tree (M0-12); scrubbed real Windows and Linux trees (M0-13 to M0-15) cover CRLF, Windows separators and Linux folder naming. DTO fixtures for Vitest are generated from the real Rust loaders (M2-01).
- **Golden parity.** While the oracle exists, the Swift app computes expected outputs for every fixture tree on a macOS runner (M1-50, M1-51). Rust must match them (M1-53), with every tolerated difference listed and reviewed.
- **Cross-OS CI.** Rust on windows-latest, ubuntu-22.04 and macos-latest (M1-58); Vitest, lint, audits, Playwright and real-app smoke added in M2-29.
- **E2E per OS** (P-8): Windows real app via CDP; Linux real app via tauri-driver under `xvfb-run`; macOS mocked only [A-10].
- **Coverage gates:** heimdall-core ≥ 80% lines before feature UI work (M1-56, part of the M1 gate); Vitest ≥ 80% lines on stores and composables (M2-01, enforced per feature in CI); golden parity passing before a feature counts as ported; Linux baselines before UI work merges. No numeric E2E coverage target is set; E2E covers every MVP screen and every shortcut (M3-18, M3-19).
- **Security acceptance tests:** XSS corpus (M2-25, M3-35), malicious transcript (M2-26), security config lint (M2-27), corrupt-config-means-no-write (M1-22, M3-40), secrets never in DTOs or debug output (M1-55), cross-OS launcher rejection (M1-32), release checklist (M4-10).

## 9. Performance budgets

These are plan-level targets (my own, not requirements from the brief), validated on the real fixture trees and generated large trees by M1-57 and M2-28. Reference hardware is the maintainer's Windows 11 machine; CI runner numbers are indicative only.

| Budget | Target |
|---|---|
| List projects and session summaries, 30 projects × 500 sessions, cold | ≤ 800 ms |
| Same, cached reload | ≤ 100 ms |
| First page of a 10 MB transcript visible after selection, uncached | ≤ 1 s (served from a partial streaming read, M1-41) |
| Full parse of a 50 MB transcript | ≤ 2 s, while other commands still answer in ≤ 100 ms |
| App start to interactive sidebar | ≤ 1.5 s on Windows and macOS, ≤ 2 s on Linux |
| Transcript scrolling at 10,000+ blocks | median frame ≤ 16.7 ms, p95 ≤ 33 ms |
| Shiki: first highlight after loading a new language | ≤ 150 ms |
| File change on disk to refreshed screen | ≤ 1.5 s (1 s debounce plus reload) |
| JavaScript bundle | initial route ≤ 250 KB gzip; each lazy route ≤ 150 KB gzip; ECharts only in the analytics chunk |

WebKitGTK fallbacks for lagging CSS features (`backdrop-filter`, custom properties in `@theme`) are documented in `docs/design/webkitgtk-fallbacks.md` (M2-02).

## 10. CI, release and local development

- **CI** (M1-58, M2-29): matrix on windows-latest, ubuntu-22.04, macos-latest; Linux installs `libwebkit2gtk-4.1-dev` and `libayatana-appindicator3-dev`; `Swatinem/rust-cache` saves only on main; `actions/setup-node` with `cache: npm`; every action pinned by SHA; minimal per-job permissions; `timeout-minutes` on every job, revisited after real runs.
- **Release** (M4-06): tauri-action on version tags; Windows `x86_64-pc-windows-msvc` NSIS; Linux AppImage, deb and rpm from a pinned ubuntu-22.04 runner (never ubuntu-latest, which raises the glibc floor); macOS `aarch64-apple-darwin`, ad-hoc signed; updater artifacts signed with `TAURI_SIGNING_PRIVATE_KEY`; `latest.json` and `SHA256SUMS` published to a draft GitHub Release; only the publish job gets `contents: write`.
- **Version bump** (M4-07): manual dispatch, calendar version per P-10, single source in `package.json`.
- **Local development** (M4-08): one setup script per OS and `npm run build` → `tauri build`, output under `src-tauri/target/release/bundle/`; measured setup time and WebView2/webkit2gtk troubleshooting in `docs/dev-setup.md`.

## 11. MVP and later scope

**MVP (first release):** session browser with projects, sessions and sub-agent sessions; conversation view with text, thinking, tool, system and edit-diff blocks, context card, timeline, AI summary card, session todos, file-history viewer, debug-log viewer, and Markdown/patch export; prompt history (read); plans; memory, including adding custom folders; cross-project TODOs; universal search over all 16 categories; read-only config screens (commands, skills, output styles, hooks, plugins, models, MCP servers with status, sub-agents); settings, onboarding, help, about and the shortcut sheet; themes and font scale; the updater.

**Later:** analytics dashboard and its CSV/PNG exports; every write to Claude Code's files (plugin toggles, models, hooks, MCP servers, sub-agents, templates, deletes); history, session and project deletes; open in editor, open terminal and reveal; tray; launch at login; multi-window; window-state restore.

**Out of scope:** Usage Limits; the provider abstraction; package channels other than GitHub Releases; code signing and notarisation; Intel macOS and ARM64 Windows/Linux builds at launch; an i18n framework (parity section X).

## 12. Risk register

| ID | Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R-1 | Claude Code changes `~/.claude` formats mid-port | Medium | High | Fixture MANIFESTs record the Claude Code version; tolerant parsing everywhere; re-run M1-54 on Claude Code updates; monthly upstream sync (M0-16) |
| R-2 | WebKitGTK rendering or CSS gaps on Linux | High | Medium | Linux-first baselines (P-13); fallbacks doc (M2-02); real-app Linux smoke (M2-23) |
| R-3 | Performance on huge transcripts | Medium | High | Summary tier for lists, IPC paging, virtualisation from day one; benchmarks in M1-57 and M2-28 |
| R-4 | Windows file locking on `~/.claude.json` writes | High | Medium | Temp file plus rename with retry and backoff, `ReplaceFileW` fallback, clear `busy` error (M1-22); writes are Later scope |
| R-5 | Scope creep toward full parity before shipping | High | Medium | MVP/Later split (§11); gates in §7; parity matrix as the gate |
| R-6 | Fork drifts from upstream the Swift reference app | Medium | Medium | Pinned reference; monthly sync producing new matrix rows (M0-16, §13) |
| R-7 | Analytics PNG export and diff copy-as-image have no settled approach (DOM-to-image is inconsistent across WebView2, WebKitGTK and WKWebView) | Medium | Low | Open risk; spike M3-37 decides before M3-39; Later scope |
| R-8 | Unsigned builds deter users: SmartScreen warnings, and macOS 15 removed the right-click "Open" bypass | High | Medium | Install guide (M4-09); revisit signing once there are users |
| R-9 | The updater may fail to replace an ad-hoc-signed macOS app | Medium | High | Verify N-1→N updates on each OS (M4-11); fall back to manual download instructions |
| R-10 | tauri-specta is still pre-release | Medium | Medium | Pin the exact version (M1-37); bindings are committed so builds never depend on regeneration; fallback is ts-rs plus hand-written wrappers (would need a new ADR) |
| R-11 | No Mac available for oracle runs or macOS testing | Medium | Medium | GitHub-hosted macOS runners (D3); macOS is the oracle, not the bar |
| R-12 | Linux package-name collision with the Samsung flasher | High | Low | Resolved by D2: package and binary are `heimdall-desktop` |
| R-13 | Linux inotify watch limits or Windows watcher overflow | Medium | Medium | Per-folder fallback, `watcher-status` event, full reload on overflow (M1-47) |
| R-14 | No tray on GNOME without the AppIndicator extension | High | Low | Documented degrade (P-15, M4-09); tray is Later scope |

## 13. Staying in sync with upstream

Once a month, run the procedure in `docs/spec/upstream-sync.md` (M0-16): list upstream commits since the pinned SHA that touch `Poirot/Sources`, `README.md` or `SHOWCASE.md`. Genuinely new capabilities become new parity rows with new tasks; path or format changes become fixture updates. Finished milestones are never reopened. Moving the pinned SHA is a deliberate change reviewed by architect-reviewer, followed by a fresh oracle run.

## 14. Requirements traceability

Every requirement in the planning brief's §6 maps to at least one task, or is out of scope with a reason.

| Brief § | Requirement | Tasks / decision |
|---|---|---|
| 6.1 | Rust workspace; core without tauri, checked in CI | M1-01, M1-05, M1-58 |
| 6.1 | SessionLoader → projects (index fast path, JSONL fallback, sub-agents, path decoding as fallback only) | M1-14 |
| 6.1 | TranscriptParser three tiers kept distinct and exposed separately | M1-12, M1-13, M1-41 |
| 6.1 | FileWatcher → watcher | M1-47 |
| 6.1 | Config read/write split; frontmatter; MCP status | M1-23 to M1-28, M1-22, M3-40 to M3-42 |
| 6.1 | FileHistory, History, Todo, DebugLog, Facets, StatsCache, full StatsComputer port | M1-15 to M1-20 |
| 6.1 | SessionExporter → export | M1-36, M3-09 |
| 6.1 | Providers → no trait | ADR 0008, M1-10, M2-06 |
| 6.1 | Terminal and editor launchers per OS; PATH search for `code`, `cursor`, `zed` and `claude` | M1-31, M1-32, M3-52 to M3-55 |
| 6.1 | UpdateChecker pointed at the fork | P-14, M4-03, M4-04 |
| 6.1 | Models → serde structs shared by core and commands | M1-08 to M1-11 |
| 6.1 | `dirs`-based home; config-dir override check; `cwd`-first paths; retire both decode fallbacks; one normalisation utility; case strategy | M1-06, M1-14, M1-19, M3-05, ADR 0003 |
| 6.1 | Streaming JSONL: `\r` trim, line streaming, skip bad lines, permissive types, content-block enum, token usage, `<synthetic>` filter, mtime/size cache, sessions-index fast path | M1-07, M1-08, M1-12, M1-13, M1-14, M1-38 |
| 6.1 | Recursive watching with `notify-debouncer-full`, platform limits and fallback, atomic rename as change, `data-changed` events | M1-34, M1-47 |
| 6.1 | Safe writes: refuse on parse failure, one-time `.bak`, generic document, unknown keys kept, Windows retry, surfaced errors | M1-22, M3-40 to M3-43, ADR 0009 |
| 6.1 | Process detection on `sysinfo`, Windows `npx.cmd` shape, auth cache, cloud prefix, explicit HTTP/SSE decision | M1-26, M1-28, P-5 |
| 6.1 | Async commands, one `AppError`, skipped-line counts | M1-05, M1-37, M1-38, ADR 0005 |
| 6.1 | Performance targets | §9, M1-41, M1-57, M2-28 |
| 6.2 | Feature folders, Pinia per boundary, `<script setup lang="ts">`, navigation as router routes with a documented semantics decision | M2-05, M2-06, M2-14, P-3 |
| 6.2 | 20 palettes, token set, `isDark`, typography, spacing, radius, continuous font scale via `calc()`, explicit text colours | M2-02, M2-05 |
| 6.2 | Shared components and bespoke skeletons | M2-07, M2-20, M3-11, M3-32 |
| 6.2 | Virtualised transcript; Markdown pipeline; lazy Shiki with theme re-highlight; tool, thinking and diff thresholds; diffs off the UI thread | M2-10, M2-17 to M2-20, M1-35, M1-42 |
| 6.2 | Six charts in ChartCard; bespoke heatmap; PNG export flagged as a risk | M3-32 to M3-34, M3-37, R-7 |
| 6.2 | Lucide, named motion utilities behind one reduced-motion composable; keyboard navigation composable; focus rings and ARIA | M2-04, M2-07, M2-08, M2-09, M3-19 |
| 6.2 | 16 search categories, 2 quick actions, both modes, all SearchAction variants, Ctrl/Cmd+K, platform-aware labels | M1-29, M1-30, M1-46, M3-01, M2-08, M3-17 |
| 6.2 | Per-OS window chrome; explicit budgets; WebKitGTK fallback doc | P-4, M2-12, §9, M2-02, M2-28 |
| 6.3 | Starting-state problems (no service boundary, 4 watcher sites, uninvalidated cache, 18 prefs in 19 files) | ADR 0002, ADR 0006, ADR 0007, M1-38, M1-39, M1-47 |
| 6.3 | Layer boundaries | ADR 0002, M1-04, M1-05, M1-58 |
| 6.3 | IPC contract: transport, generated types with drift check, prefs `schemaVersion`, tolerant parsing, tagged errors | ADR 0004, ADR 0005, `docs/ipc-contract.md`, M1-37, M1-39 |
| 6.3 | State flow and caching | ADR 0006, M1-34, M1-38, M1-47 |
| 6.3 | Rust-owned preferences via the store plugin; dev identifiers; no UserDefaults import; nothing under `~/.claude` | ADR 0007, M1-11, M1-39, §5.8 |
| 6.3 | ADRs on 11 topics | `docs/adr/0001`–`0011` |
| 6.3 | Rules-file rewrite as the first task | M0-01 |
| 6.4 | Threat model | ADR 0010 |
| 6.4 | Weaknesses not to port: parse-then-overwrite, dropped MCP fields, naive args split, leaky mask, unchecked editor argv, shell-string terminal launch, uncalled hook import, tag-pinned Actions, workflow-wide `contents: write` | M1-22, M3-40, M3-41, M3-48, M1-32, M3-52, M3-44, M1-58, M4-06 |
| 6.4 | Controls 1–10: rendering, links, CSP, capabilities, IPC shape, code-execution writes, process launch, secrets, updates and signing, supply chain | ADR 0010 maps each to tasks; M4-14 collects evidence |
| 6.4 | Acceptance tests | M2-25, M2-26, M2-27, M1-22, M3-40, M1-55, M1-32, M4-10, M3-35 |
| 6.5 | Test inventory and carry-over | M0-06 and each task's carry-over criteria |
| 6.5 | Fixture extraction and real anonymised trees | M0-11 to M0-15 |
| 6.5 | Golden-output parity while Swift exists | M1-50 to M1-53, M4-15, M4-16 |
| 6.5 | Cross-OS CI and E2E, macOS WebDriver assumption | M1-58, M2-23, M2-29, P-8, A-10 |
| 6.5 | Coverage gates; baseline decision | M1-56, M2-01, M1-53, M2-24, P-13 |
| 6.6 | Port the three workflows | M1-58, M2-29, M4-06, M4-07 |
| 6.6 | Build matrix; Windows installer choice; Linux targets on pinned ubuntu-22.04 | M4-01, M4-06, P-9 |
| 6.6 | Signing | Out of scope for Windows/macOS by user answer 7; updater key M4-02; Linux checksums M4-06 |
| 6.6 | Channels | GitHub Releases only (user answer 7) |
| 6.6 | Linux build deps; distro verification | M1-58, M4-06, M4-08, M4-12 |
| 6.6 | Setup and build scripts with measured times | M4-08 |
| 6.6 | CI caching and time budgets | M1-58, M2-29, M4-06 |
| 6.7 | Milestones with per-OS gates | §7; M1-59, M2-30, M3-20, M3-62, M4-14 |
| 6.7 | Parity matrix with one outcome per non-parity row | `docs/parity-matrix.md` |
| 6.7 | Swift app as reference oracle with an exit condition | ADR 0001, M1-50 to M1-53, M4-15 |
| 6.7 | MVP and later scope | §11 |
| 6.7 | Risk register | §12 |
| 6.7 | Upstream sync | §13, M0-16 |

## 15. Self-check

| Check | Result |
|---|---|
| Every cited file path spot-checked; discrepancies noted | Done: §3.1; Swift paths and line ranges in task files re-verified by `check_backlog.py --swift-ref` |
| Every open decision presented with a recommendation, none silently resolved | Brief items 1–8 answered by the user (§2.1); follow-up identity, oracle and repo-location questions D1–D4 answered by the user on 2026-09-25 (§2.2) |
| Every domain requirement maps to a task or is out of scope with a reason | §14 |
| Every task file has all required fields, one workstream's files, real dependency ids | `check_backlog.py`: 174 tasks, 0 errors |
| No two parallel tasks write the same file | Enforced by `check_backlog.py` |
| Parity matrix has a row per screen, feature, data source, preference key and shortcut in the brief's appendix | Sections A–L, plus rows for items the brief missed (C-14) |
| The IPC contract exists and both Rust and Vue tasks point to it | `docs/ipc-contract.md`; M1-37 turns it into generated bindings that Vue tasks consume |
| Every task touching IPC, capabilities or a `~/.claude` write has security-auditor | Enforced by `check_backlog.py` (§5.6) |
| Milestone gates match M0–M4 and are stated per OS | §7 |
| No `.rs`, `.vue`, `.ts` or `.swift` file created or modified during planning | Only Markdown files and `docs/tasks/check_backlog.py` were written; `$SWIFT_REF` is untouched (`git status` there shows only the brief) |
| MVP/later split stated explicitly | §11 and the Scope column of every matrix row and task |
