# Progress log

Append-only record of every task. Gate-review tasks (M1-59, M2-30, M3-20, M3-62, M4-14) copy statuses from here into the parity matrix.

## Rules

1. Add one entry at the **bottom** when a task finishes (or is blocked). Never edit or delete another entry; correct a mistake with a new entry that references the old one.
2. Use the template exactly. Keep it factual: what changed, how each acceptance criterion was proven, what is left.
3. Evidence is a link or a command with its result: a CI run URL, a test command and its pass count, a screenshot path, a manual check with OS and version.
4. Record anything a task asks you to record here (for example the tauri-specta version in M1-37, the `CLAUDE_CONFIG_DIR` finding in M1-06, the first coverage figure in M1-56).
5. If you find a problem outside your task, list it under Follow-ups with a proposed owner workstream; do not fix it in your task.

## Template

```markdown
### <TASK-ID> · <task title> · <YYYY-MM-DD>

- **Agent:** <subagent type>   **Reviewers:** <who signed off>
- **Status:** done | blocked | partially done
- **Files touched:** `path`, `path`
- **Evidence:**
  - AC1: <how it was proven>
  - AC2: <…>
- **Per-OS notes:** Windows … · Linux … · macOS …   (omit if not applicable)
- **Follow-ups:** <item> → <owner workstream>   (or "none")
```

## Entries

### PLAN · Planning package produced · 2026-09-25

- **Agent:** planning agent   **Reviewers:** pending (user)
- **Status:** done
- **Files touched:** `docs/port-plan.md`, `docs/parity-matrix.md`, `docs/ipc-contract.md`, `docs/progress-log.md`, `docs/adr/0001`–`0011`, `docs/tasks/README.md`, `docs/tasks/M0-01` … `M4-16` (174 files), `docs/tasks/check_backlog.py`
- **Evidence:**
  - Brief citations spot-checked against `$POIROT` at `a29dd6e`; 21 corrections recorded in port-plan §3.1.
  - `python docs/tasks/check_backlog.py --poirot C:\Users\pc\Desktop\tech\poirot` → 174 tasks checked, 0 errors.
  - Linux name collision checked on packages.debian.org, archlinux.org and packages.fedoraproject.org (ADR 0011).
  - No `.rs`, `.vue`, `.ts` or `.swift` file created or modified.
- **Follow-ups:** user answers D1–D4 (port-plan §2.2) → Docs (M0-02) and Release (M0-03)

### M0-02 · Record the final app identifier and Linux package name · 2026-09-25

- **Agent:** planning agent, acting as technical-writer   **Reviewers:** pending (architect-reviewer)
- **Status:** done
- **Files touched:** `docs/adr/0011-app-identifier-and-release-repo.md`, `docs/adr/0001-repo-layout-and-swift-reference.md`, `docs/adr/0007-preferences-storage-and-migration.md`, `docs/port-plan.md`, `docs/tasks/M0-01-rules-file.md`, `docs/tasks/M0-02-record-identity-decisions.md`, `docs/tasks/M0-03-repo-bootstrap.md`, `docs/tasks/M2-04-icons-motion.md`
- **Evidence:**
  - User answer, 2026-09-25: follow the recommendations for D1–D3; the folder was moved out of `hydra` (D4).
  - AC1: ADR 0011 status reads Accepted with identifier `io.github.amine-bourouba.heimdall` and package/binary name `heimdall-desktop`.
  - AC2: the identifier uses only `A-Z a-z 0-9 . -` and does not end in `.app`.
  - AC3: `heimdall-desktop` is absent from the Debian/Ubuntu, Arch and Fedora name searches recorded in ADR 0011, each of which lists every package containing "heimdall".
  - AC4: `grep -rn -E "<APP_ID>|<LINUX_PKG>" docs/adr docs/port-plan.md` returns nothing.
  - D4: the project now lives at `C:\Users\pc\Desktop\tech\Heimdall`; `git rev-parse --show-toplevel` there reports "not a git repository", so it is outside `hydra`.
  - Task graph: M0-02 no longer depends on M0-01, and M0-01 now writes the real identifier directly instead of a placeholder. M2-04 gained an explicit M0-01 dependency because it reads the rules file. `check_backlog.py` → 174 tasks checked, 0 errors.
- **Follow-ups:** M0-03 runs `git init` in the new location → Release; M0-01 is the next task to start → Docs

### M0-03 · Initialise the Heimdall repository and its ignore/attribute rules · 2026-09-25

- **Agent:** build-engineer (executed by the coordinating agent)   **Reviewers:** pending (architect-reviewer)
- **Status:** done
- **Files touched:** `.gitignore`, `.gitattributes`, `.editorconfig` (plus `git init -b main`)
- **Evidence:**
  - AC1: `git rev-parse --show-toplevel` → `C:/Users/pc/Desktop/tech/Heimdall`.
  - AC2: `.gitattributes` has `* text=auto eol=lf` and `fixtures/** -text`; `git check-attr text -- fixtures/windows/x.jsonl` → `unset`, `src/main.ts` → `auto`.
  - AC3: `.gitignore` covers `target/`, `node_modules/`, `dist/`, `src-tauri/gen/`, `coverage/`, `test-results/`, `playwright-report/` (and `*.key` so updater keys are never committed).
- **Follow-ups:** none. No commit made yet: commits wait for the user's go-ahead.

### M0-01 · Write Heimdall's AI_INSTRUCTIONS.md and CLAUDE.md · 2026-09-25

- **Agent:** documentation-engineer (executed by the coordinating agent)   **Reviewers:** pending (architect-reviewer)
- **Status:** done
- **Files touched:** `AI_INSTRUCTIONS.md`, `CLAUDE.md`
- **Evidence:**
  - AC1: `CLAUDE.md` is 3 lines and only points to `AI_INSTRUCTIONS.md`.
  - AC2: sections present: Overview, Stack, Layer rules, Icons, Design tokens, Security, Commands, Parallel development (worktrees), Swift reference, Feature development workflow, New Feature Checklist, Git & PRs.
  - AC3: the New Feature Checklist is adapted (Ctrl/Cmd+K, watcher areas, Playwright baselines).
  - AC4: Git & PRs copied verbatim from `$POIROT/AI_INSTRUCTIONS.md`.
  - AC5: the Swift-rules grep matches only line 91, inside the Swift reference section.
- **Follow-ups:** none.

### M0-04 · Add LICENSE, NOTICE and a scoped README · 2026-09-25

- **Agent:** technical-writer (executed by the coordinating agent)   **Reviewers:** pending (code-reviewer)
- **Status:** done
- **Files touched:** `LICENSE`, `NOTICE`, `README.md`
- **Evidence:**
  - AC1: LICENSE is MIT with `Copyright (c) 2026 amine-bourouba (Heimdall)` and `Copyright (c) 2026 Leonardo Cardoso (Poirot, …)`; the permission text is byte-identical to Poirot's (`diff` of the body is empty).
  - AC2: NOTICE names https://github.com/a7t-ai/poirot at `a29dd6ee0e0133ef26bd067c164c66b160e13d86`.
  - AC3: `grep -n -i "usage limit" README.md` returns nothing.
  - AC4: every README feature maps to MVP parity rows (B2–B18, B23–B37, C8–C12).
- **Follow-ups:** none.

### M0-05 · Exhaustive inventory of every ~/.claude path Poirot reads or writes · 2026-09-25

- **Agent:** legacy-modernizer (executed by the coordinating agent)   **Reviewers:** pending (qa-expert)
- **Status:** done
- **Files touched:** `docs/spec/claude-home-inventory.md`; planning corrections in `docs/parity-matrix.md` (new row G7), `docs/port-plan.md` (C-22, C-23), `docs/adr/0006-change-propagation-and-caching.md`
- **Evidence:**
  - AC1: §6 cross-references every parity row in sections E, F and G, including the new G7.
  - AC2: 110 Swift citations; an automated check confirmed every cited line exists at `a29dd6e`.
  - AC3: §5 lists every `homeDirectoryForCurrentUser`/`NSHomeDirectory` hit.
  - AC4: §7 lists the discrepancies, including `originalPath` versus `projectPath` and the unused `cwd`.
  - New findings: 13 watcher start sites in 8 files (brief said 4); history deletes remove lines by index (D-5).
- **Follow-ups:** none.

### M0-06 · Map every PoirotTests case to its Heimdall destination · 2026-09-25

- **Agent:** legacy-modernizer (executed by the coordinating agent)   **Reviewers:** pending (qa-expert)
- **Status:** done
- **Files touched:** `docs/spec/test-carryover.md`; task inputs updated for M1-10, M2-02, M2-07, M2-14, M3-02, M3-11, M3-31, M3-43 (they now name the Swift tests routed to them)
- **Evidence:**
  - AC1: `grep -rho "@Test" $POIROT/PoirotTests | wc -l` = 637; the map has 637 rows.
  - AC2: every destination is an output of some task (checked by script).
  - AC3: drops limited to ClaudeCodeProviderTests, UpdateCheckerTests, `UsageWindowTests.timeUntilReset` (Usage Limits leftover), mocks, and the stale `ScreenshotTests_Usage` skip. HighlightedTextTests has no AttributedString assertions, so all four move to `search/matcher.rs`.
  - AC4: AnalyticsCSVExporterTests maps to Rust with the P-18 reason.
- **Follow-ups:** none.

### M0-07 · Behaviour spec for universal search · 2026-09-25

- **Agent:** legacy-modernizer   **Reviewers:** pending (qa-expert)
- **Status:** done
- **Files touched:** `docs/spec/search-overlay.md`
- **Evidence:** all 16 categories with sources, fields and scoping; fuzzy and exact scoring with worked examples verified against HighlightedTextTests; empty-query state; both quick actions; all SearchAction variants; key bindings; 200 ms debounce; custom-folder gap. Every Swift citation resolves (automated check, 23 citations).
- **Follow-ups:** Poirot has 6 SearchAction variants, the contract has 4; the two quick actions need IPC variants when they ship (S17, S18, Later) → Rust data core.

### M0-08 · Behaviour spec for the session browser and conversation view · 2026-09-25

- **Agent:** legacy-modernizer   **Reviewers:** pending (qa-expert)
- **Status:** done
- **Files touched:** `docs/spec/session-views.md`
- **Evidence:** one section per Swift view; every threshold with its source line (page size 20, 80/50-line truncation, 5000-line diff cap, 4 s/10 s toasts, debug-log page 200, and about 25 more); the full toolbar table; buildable skeleton layouts. 109 Swift citations all resolve.
- **Follow-ups:** "Expand All" updates only tool blocks in Poirot; thinking and system blocks ignore it. Decide whether to copy the asymmetry (spec open question 5) → Feature screens (M2-20).

### M0-09 · Behaviour spec for the configuration screens, forms and MCP wizard · 2026-09-25

- **Agent:** legacy-modernizer   **Reviewers:** pending (qa-expert)
- **Status:** done
- **Files touched:** `docs/spec/config-screens.md`
- **Evidence:** a section per screen with read-only versus mutating actions; the full MCP wizard (3 steps, 14 catalog entries, validation, preview); the "Do not port" section with the 4 required items verified plus 6 more (multi-handler hook truncation on edit is the most severe). 163 Swift citations all resolve.
- **Follow-ups:** Swift folds user, local and project MCP sources into one `.user`; the contract keeps five. Keep the contract (richer) and note it in M1-26/M1-45 → Rust data core.

### M0-10 · Behaviour spec for analytics, exports and the context ring · 2026-09-25

- **Agent:** legacy-modernizer   **Reviewers:** pending (qa-expert)
- **Status:** done
- **Files touched:** `docs/spec/analytics.md`
- **Evidence:** exact range and reference-date formulas; every derived series and formatter; six charts; eight stat cards; the Monday-first heatmap algorithm; four CSV formats (always all-time); the PNG snapshot; the 200k/1M context-window rule; staleness rules. 65 Swift citations all resolve.
- **Follow-ups:** 9 open questions about Poirot's own inconsistencies (for example the "(UTC)" hour label on local-time buckets) → Feature screens (M3-31 to M3-34).

### M0-13 · Write the ~/.claude scrubbing tool · 2026-09-25

- **Agent:** test-automator (executed by the coordinating agent)   **Reviewers:** pending (qa-expert, security-auditor)
- **Status:** done
- **Files touched:** `scripts/scrub-claude-home.mjs`, `scripts/scrub-claude-home.test.mjs`
- **Evidence:** `node --test scripts/scrub-claude-home.test.mjs` → 10 pass, 0 fail: CRLF kept byte for byte; secrets replaced; home replaced in Windows, macOS, Linux and hyphen-encoded forms; same-length filler with valid JSON; credentials never copied; username, email and client names absent from the output.
- **Beyond the task:** project path segments below the home (for example client folder names) become stable pseudonyms `proj-N` across paths, folder names and config keys; folders whose project path cannot be determined are reported for manual review. Git branch names become filler.
- **Follow-ups:** none.

### M0-14 · Capture a scrubbed real Windows ~/.claude tree · 2026-09-25

- **Agent:** test-automator (executed by the coordinating agent)   **Reviewers:** pending (qa-expert)
- **Status:** done
- **Files touched:** `fixtures/windows/claude-home/**` (118 files), `fixtures/windows/claude.json`, `fixtures/windows/crlf/**`, `fixtures/windows/MANIFEST.md`
- **Evidence:**
  - AC1: MANIFEST records Claude Code 2.1.282, 2026-09-25, Windows build 10.0.26200.9445, counts, and LF endings.
  - AC2: transcripts are LF, so `crlf/` holds a CRLF copy of one 30-line transcript (content identical apart from line ends).
  - AC3: `cwd` values such as `C:\Users\fixture\Desktop\tech\proj-11` are present.
  - AC4: largest file 1 MB (capped); a grep for the real username, email and 13 real project names found nothing.
  - Real-data findings recorded in MANIFEST and fed into M1-14 and ADR 0003: mixed `C:`/`c:` drive case, `cwd` moving into subfolders, orphaned transcripts, synced skills, no `sessions-index.json`, persisted tool outputs. Assumption A-18 answered (LF).
- **Follow-ups:** none.

### M0-15 · Capture a scrubbed real Linux ~/.claude tree · 2026-09-25

- **Agent:** test-automator   **Reviewers:** —
- **Status:** blocked
- **Evidence:** WSL is installed (Ubuntu), but it cannot start: `HCS_E_SERVICE_NOT_AVAILABLE` (the Virtual Machine Platform feature is off, which needs admin rights and a reboot). No Linux machine is available.
- **Follow-ups:** the user enables WSL (`wsl --install --no-distribution` as admin, reboot), installs Claude Code in Ubuntu and runs a session; then run the scrub tool there → Testing. This blocks M1-50, M1-54, M1-57 and the M1 gate.

### M0-11 · Extract PoirotTests inline fixtures into a shared synthetic ~/.claude tree · 2026-09-25

- **Agent:** test-automator   **Reviewers:** pending (qa-expert)
- **Status:** done
- **Files touched:** `fixtures/README.md`, `fixtures/synthetic/**` (249 files: `cases/` 154 test-case folders, `claude-home/` 24 files, `claude.json`, `project-repo/`, `corrupt/`, `README.md`, `check_fixtures.py`)
- **Evidence:**
  - AC1: the README maps all 164 carried-over Services tests plus 35 decoding tests to existing paths or `inline` (checked by script against test-carryover.md).
  - AC2: `claude-home/` holds one valid instance of every inventory format.
  - AC3: `python fixtures/synthetic/check_fixtures.py` → all checks passed; an independent parse of `claude-home/` and `claude.json` found 0 bad records.
  - Deviation: 5 intentionally invalid per-test inputs stay inside their own `cases/` folder (listed in the README and the check script) instead of `corrupt/`.
- **Follow-ups:** none.

### M0-12 · Convert ScreenshotData.swift into a deterministic fixture tree · 2026-09-25

- **Agent:** test-automator   **Reviewers:** pending (qa-expert)
- **Status:** done
- **Files touched:** `fixtures/screenshot/**` (31 files)
- **Evidence:** the README maps all 60 ScreenshotData declarations; fixed reference date 2026-09-20T12:00:00Z; independent parse of 364 records found 0 bad; no home path other than `/Users/fixture`.
- **Follow-ups:** some mock values cannot come from disk (toasts, standalone tool blocks, illustrative token totals larger than the transcript sums); the README lists them → Testing (M2-24 uses component-level fixtures for those).

### M0-16 · Define and run the upstream Poirot sync check · 2026-09-25

- **Agent:** legacy-modernizer (executed by the coordinating agent)   **Reviewers:** pending (qa-expert)
- **Status:** done
- **Files touched:** `docs/spec/upstream-sync.md`
- **Evidence:** first run 2026-09-25 through a throwaway clone (so `$POIROT` stays untouched): upstream head is `a29dd6e`, 0 new commits.
- **Follow-ups:** next run 2026-10-25 → Legacy parity extraction.

### M1-02 · Create package.json with the full frontend toolchain · 2026-09-25

- **Agent:** dependency-manager (executed by the coordinating agent)   **Reviewers:** pending (architect-reviewer, security-auditor)
- **Status:** done
- **Files touched:** `package.json`, `package-lock.json`; task data M1-02, M2-01, M2-23, new task M2-31, M4-08, port-plan P-8
- **Evidence:**
  - `npm ci --include=dev` exits 0; `npm audit --include=dev --audit-level=high` → 0 vulnerabilities.
  - Deviations: `@lucide/vue` replaces the deprecated `lucide-vue-next`; `happy-dom` replaces jsdom (jsdom 30 needs Node 22.22+); WebdriverIO deferred to new task M2-31 because 9.32 pulls a high-severity `extract-zip` advisory.
  - Environment notes for this machine: the shell sets `NODE_ENV=production` (npm skips dev dependencies unless `--include=dev`) and `NODE_TLS_REJECT_UNAUTHORIZED=0` (TLS verification off).
- **Follow-ups:** consider Node 22.22+ or 24 LTS locally (a few transitive dev tools warn on 22.17) → Release.

### M1-04 · Scaffold the Vite + Vue 3 + TypeScript + Tailwind v4 frontend · 2026-09-25

- **Agent:** vue-expert (executed by the coordinating agent)   **Reviewers:** pending (architect-reviewer)
- **Status:** done
- **Files touched:** `index.html`, `src/main.ts`, `src/App.vue`, `src/env.d.ts`, `vite.config.ts`, `tsconfig.json`, `eslint.config.js`
- **Evidence:**
  - `npm run build` → `dist/index.html`; `npm run typecheck` exit 0; `npm run lint` clean.
  - Throwaway files proved `vue/no-v-html` and the `@tauri-apps/api/core` import ban both fire, and that `src/lib/markdown/MarkdownView.vue` is exempt from the `v-html` ban.
  - Port from `HEIMDALL_DEV_PORT` (default 1420), `strictPort: true`; Tailwind v4 plugin wired.
  - Deviation: one `tsconfig.json` covers the app and config files, so there is no `tsconfig.node.json`.
- **Follow-ups:** M2-06 must import `src/styles/main.css` in `src/main.ts` → Vue shell.

### M1-01 · Create the Cargo workspace, crate manifests and supply-chain policy · 2026-09-25

- **Agent:** dependency-manager (executed by the coordinating agent)   **Reviewers:** pending (architect-reviewer, security-auditor)
- **Status:** done
- **Files touched:** `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `deny.toml`, `crates/heimdall-core/Cargo.toml`, `crates/heimdall-core/src/lib.rs` (empty placeholder for M1-05), `src-tauri/Cargo.toml`
- **Evidence:**
  - `cargo metadata --format-version 1` succeeds; `cargo check -p heimdall-core` passes.
  - `cargo tree -p heimdall-core -e normal | grep -c tauri` → 0.
  - Dependencies as listed in the task; tauri-specta and specta pinned to `=2.0.0-rc.25`, specta-typescript to `=0.0.12`.
  - `rust-toolchain.toml` pins 1.98.1 (≥ 1.77.2).
  - `cargo deny check` (cargo-deny 0.20.2) → advisories ok, bans ok, licenses ok, sources ok. Getting there: `rust-version` raised from 1.85 to 1.98 so the MSRV-aware resolver stops choosing old crates (this pulled `time` 0.3.55 and `quick-xml` 0.42, fixing RUSTSEC-2026-0009, -0194, -0195); `unmaintained = "workspace"` (the unmaintained `unic-*`, `paste` and `proc-macro-error` crates sit deep in Tauri's tree); `allow-wildcard-paths` for the local path dependency; `tauri-plugin-fs` allowed only as a dependency of `tauri-plugin-dialog`, and `tauri-plugin-shell` stays banned.
  - Package and binary name `heimdall-desktop` (ADR 0011).
- **Follow-ups:** none.

### M1-03 · Scaffold the Tauri app crate with final identity, CSP and module stubs · 2026-09-25

- **Agent:** rust-engineer (executed by the coordinating agent)   **Reviewers:** pending (security-engineer, security-auditor, deployment-engineer)
- **Status:** done on Windows; Linux and macOS launch checks wait for CI (M1-58)
- **Files touched:** `src-tauri/src/main.rs`, `src-tauri/src/lib.rs`, `src-tauri/build.rs`, `src-tauri/tauri.conf.json`, `src-tauri/icons/**` (placeholder set from `tauri icon`, mobile folders removed), placeholders for every src-tauri module in ADR 0002
- **Evidence:**
  - `lib.rs` registers opener, dialog, clipboard-manager, store and autostart; setup calls `window`, `menu`, `watcher`, `tray` and `updater` init stubs; tauri-specta's builder supplies the invoke handler and events.
  - `tauri.conf.json`: productName Heimdall, identifier `io.github.amine-bourouba.heimdall`, version from `../package.json`, main window 1280×820 (min 900×600), `csp` and `devCsp` equal to the ADR 0010 strings (compared by script), `freezePrototype: true`, asset protocol off, `withGlobalTauri: false`.
  - `cargo clippy --workspace -- -D warnings` → exit 0; `cargo build -p heimdall-desktop` links.
  - `npx tauri dev` on Windows 11: process `heimdall-desktop` running with window title "Heimdall", responding; then stopped.
  - Production CSP has no `unsafe-*`; no fs or shell plugin in `Cargo.toml`.
- **Environment notes:** this machine has no MSVC build tools, so it builds with Rust's GNU toolchain plus a portable MinGW (`%LOCALAPPDATA%\heimdall-dev\mingw64`, no admin). The MinGW linker warns "multiple non-default manifests"; the app still runs. Release builds use MSVC in CI.
- **Follow-ups:** install the Visual Studio Build Tools (C++ workload) for MSVC parity locally → user.

### M1-05 · Declare every heimdall-core module and implement AppError · 2026-09-25

- **Agent:** rust-engineer (executed by the coordinating agent)   **Reviewers:** pending (architect-reviewer)
- **Status:** done
- **Files touched:** `crates/heimdall-core/src/lib.rs`, `crates/heimdall-core/src/error.rs`, and placeholders for every other module path in the task's Outputs
- **Evidence:**
  - `cargo test -p heimdall-core error::` → 4 passed. One test checks each of the six variants against `{"kind": …, "message": "m", "path": null}`; the others cover `with_path`, `from_io` (Windows sharing violations 32/33 → `busy`) and the `Loaded`/`WriteOutcome` shapes.
  - `AppError` derives `thiserror::Error`, `Serialize` and `specta::Type`; `Loaded<T>` has `value` and `skipped`.
  - `lib.rs` has `#![deny(clippy::print_stdout, clippy::print_stderr)]`.
  - `RUSTDOCFLAGS="-D warnings" cargo doc -p heimdall-core --no-deps` → exit 0.
- **Follow-ups:** none.

### M1-06 · Claude home resolution, path normalisation and comparison keys · 2026-09-25

- **Agent:** rust-engineer (executed by the coordinating agent)   **Reviewers:** pending
- **Status:** done
- **Files touched:** `crates/heimdall-core/src/paths.rs`
- **Evidence:**
  - `cargo test -p heimdall-core paths::` → 7 passed.
  - `ClaudeHome::for_user_home` and `ClaudeHome::new(root)` take explicit roots; only `from_env` reads the environment, and tests never touch the real home.
  - `CLAUDE_CONFIG_DIR`: verified against Claude Code 2.1.282 on this machine. With the variable set to an absolute folder, the CLI reads and writes there instead of `~/.claude`. A relative value is not honoured, so `from_env` accepts absolute values only (test `claude_home_resolution_honours_an_absolute_config_dir_only`).
  - Normalisation tests cover `C:\Users\a\x`, `c:/Users/a/x`, `\?\C:\Users\a\x`, mixed and doubled separators, trailing separators, `/Users/a/x/`, and UNC forms.
  - Comparison keys fold case for Windows and macOS and keep it for Linux. Every function takes `TargetOs`, so all variants run on any host.
  - `same_file` matches on segment boundaries, including `src/a.rs` against `C:\proj\src\a.rs`.
  - `is_unc` detects `\server\share` and `//server/share` but not `\?\C:\`.
  - `decode_project_dir_name` takes an injected `exists` function and is documented as the last-resort fallback.
- **Follow-ups:** none.

### M1-07 · Streaming JSONL reader with CRLF and truncation tolerance · 2026-09-25

- **Agent:** rust-engineer (executed by the coordinating agent)   **Reviewers:** pending
- **Status:** done
- **Files touched:** `crates/heimdall-core/src/jsonl.rs`
- **Evidence:**
  - `cargo test -p heimdall-core jsonl::` → 6 passed. The tests cover LF, CRLF, mixed line endings, a UTF-8 BOM, blank lines, an invalid line mid-file and a truncated last line, with the skipped counts asserted.
  - `Records` iterates a `BufReader`; `grep read_to_string crates/heimdall-core/src` finds nothing.
  - `read_head(path, max, qualifies)` reads in 8 KiB chunks and stops after N qualifying lines.
- **Follow-ups:** none.

### M1-08 · Serde models for messages, sessions and projects · 2026-09-25

- **Agent:** rust-engineer (executed by the coordinating agent)   **Reviewers:** pending
- **Status:** done
- **Files touched:** `crates/heimdall-core/src/models/{message,session,project,token_usage,context_usage,session_group}.rs`
- **Evidence:**
  - Test counts: message 19, session 33, project 4, token_usage 10, context_usage 4, session_group 8. There is one Rust test per Swift test in test-carryover.md for these files; session tests that had been merged were split back out to match one to one.
  - IPC structs use `rename_all = "camelCase"` and derive `specta::Type`. `ContentBlock` is tagged by `kind` (`text`, `thinking`, `toolUse`, `toolResult`).
  - The model sentinel is `message::SYNTHETIC_MODEL`.
  - Disk types use `#[serde(default)]` and ignore unknown fields.
- **Follow-ups:** none.

### M1-09 · Serde models for stats, history, todos, facets, debug log and file history · 2026-09-25

- **Agent:** rust-engineer (executed by the coordinating agent)   **Reviewers:** pending
- **Status:** done
- **Files touched:** `crates/heimdall-core/src/models/{stats_cache,history_entry,session_todo,session_facets,debug_log_entry,file_version}.rs`
- **Evidence:**
  - Test counts: stats_cache 12, history_entry 5, session_todo 8, session_facets 12, debug_log_entry 7, file_version 1.
  - `SessionFacets` reads snake_case keys and serialises camelCase (tested).
  - `TodoStatus` accepts `in_progress`, `pending` and `completed`. `list_from_json` skips items with unknown statuses without failing the file (tested).
  - The `costUSD` key is kept exactly.
- **Follow-ups:** none.

### M1-10 · Config models, catalogs and the redacting Secret type · 2026-09-25

- **Agent:** rust-engineer (executed by the coordinating agent)   **Reviewers:** pending (architect-reviewer, security-auditor)
- **Status:** done
- **Files touched:** `crates/heimdall-core/src/models/config/{scope,mcp,hook,sub_agent,command,skill,output_style,plan,memory_file,plugin,settings}.rs`, `crates/heimdall-core/src/models/catalog/{claude_models,mcp_catalog}.rs`, `crates/heimdall-core/src/models/secret.rs`, `docs/ipc-contract.md` (added `McpCatalogEntry`, `McpEnvKey`). Lint fixes in `models/session_group.rs` and `jsonl.rs`, which `cargo clippy --all-targets` flagged.
- **Evidence:**
  - Carried-over tests pass: `cargo test -p heimdall-core` → 210 passed.
    - MCPServerCatalogTests: 14 tests in `mcp_catalog`.
    - Catalog cases from SettingsWriterMCPTests: 4 tests in `mcp_catalog`.
    - MCPServerStatusTests: 7 tests in `mcp`.
    - PlanTests: 8 tests in `plan`.
    - MemoryFileTests: 12 tests in `memory_file`.
    - SubAgentTests: 7 tests in `sub_agent`.
    - ClaudeModelCatalogTests: 7 tests in `claude_models`.
    - Moved elsewhere, as test-carryover.md records: fuzzyMatch cases to M1-29, the plan delete case to M3-42, and SF Symbol cases to M3-14.
  - `Secret`: `Debug` and `Display` print `[redacted]`, and it has no `Serialize`. `McpServer` serialises `envKeys` and `headerKeys`, and skips the value maps.
  - Test `serialised_server_carries_env_key_names_but_never_values` builds a server from `env: {"API_KEY": "sk-123"}` and asserts that the JSON has no `sk-123`.
  - `HookHandler` keeps unknown keys in `extra` and sends only `extraKeys` over IPC (a test asserts that an http header value never reaches JSON). `handler_round_trips_every_key` covers `from_json` → `to_json`. `McpServerDefinition` keeps `type`, `url` and `headers`.
  - Catalogs match Swift entry for entry:
    - MCP catalog: 15 entries in Poirot's order, with ids, env keys and defaults as in Swift. The count is asserted.
    - Model catalog: 5 curated models, with `DEFAULT_MODEL_NAME` "Opus 5" and the same `friendly_name` algorithm.
  - `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- **Deviations:**
  - Icons stay Poirot's SF Symbol names (sub-agents, MCP catalog and its categories); M2-04's icon map converts them to Lucide.
  - `McpServer.status` is not serialised, because the contract sends status separately as `McpStatus`.
  - `McpServerSource` has the contract's five values, not Swift's three.
  - `HookEvent.icon` is not ported to Rust, because no DTO carries it; the hooks UI owns it (M2-04 map).
- **Follow-ups:** none.

### M1-11 · Typed preferences, editor and terminal enums · 2026-09-25

- **Agent:** rust-engineer (executed by the coordinating agent)   **Reviewers:** pending
- **Status:** done
- **Files touched:** `crates/heimdall-core/src/models/prefs.rs`
- **Evidence:**
  - `cargo test -p heimdall-core prefs` → 15 passed; `cargo clippy -p heimdall-core --all-targets -- -D warnings` → exit 0.
  - All 18 keys plus `schemaVersion` have the types and defaults in ADR 0007 (`defaults_follow_adr_0007` counts 19 keys). `claudeCodePath` defaults to null, which means a PATH search.
  - `#[serde(default)]` on the struct:
    - `missing_keys_take_defaults_and_unknown_keys_are_ignored` checks both.
    - `Prefs::from_stored` goes further: a value that no longer parses falls back for that key only, so a downgrade cannot wipe `customMemoryFolders` (`a_bad_value_falls_back_for_that_key_only`).
  - `PreferredEditor` is vscode, cursor, xcode or zed; `options(os)` drops Xcode off macOS.
  - `PreferredTerminal` has the ADR 0003 sets:
    - macOS: Poirot's six.
    - Windows: Windows Terminal, PowerShell, cmd.
    - Linux: GNOME Terminal, Konsole, kitty, Alacritty, system.
    - Defaults are tested per OS on any host.
  - Carried-over tests: all 6 PreferredEditorTests and all 4 PreferredTerminalTests (checked against the macOS set).
  - `ColorThemeId` has the 20 ids from PoirotTheme.swift.
- **Deviations:** VS Code is stored as `vscode` (ADR 0007), not Poirot's raw value `code`. The raw-value test asserts `vscode`; the CLI command is still `code`.
- **Follow-ups:** `update_prefs` validation (fontScale range and step, per-OS terminal and editor options) and the schema migration hook → M1-39.

### M1-12 · Full-parse tier, content blocks, system content and tool input · 2026-09-25

- **Agent:** rust-engineer (executed by the coordinating agent)   **Reviewers:** pending
- **Status:** done
- **Files touched:** `crates/heimdall-core/src/transcript/{full,blocks,searchable,system_content,tool_input}.rs`, `crates/heimdall-core/src/models/message.rs` (`ToolUse.input` kept in Rust for exports; `SystemSegment.children`), `docs/ipc-contract.md` (`SystemSegment.children`). Also deleted `fixtures/windows/grep.exe.stackdump`, which a crashed MSYS `grep` had left there; it is not fixture data.
- **Evidence:**
  - `cargo test -p heimdall-core transcript` → 47 passed; `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
  - Carried-over tests:
    - All 20 full-parse cases from TranscriptParserTests, plus `parse_searchableText_indexesContent`.
    - All 9 SystemContentParserTests.
    - All 7 JSONBeautifierTests.
  - System, summary, progress, file-history-snapshot and file-history-delta records are passed over without error; `<synthetic>` assistant messages are dropped (`parse_only_skipped_record_types_returns_nil`, `parse_synthetic_model_skipped`).
  - The sidechain filter matches TranscriptParser.swift:34: sidechain records are dropped unless the context is a sub-agent transcript (tested both ways).
  - Block ids are `<messageId>:<index>` and identical across two parses (`block_ids_are_message_id_and_index_and_stable`). Records with no uuid or message id use `record-<n>`, not a random UUID, so ids stay stable.
  - All 25 transcripts in `fixtures/windows` (LF tree and CRLF copy, main and sub-agent) parse with `skipped == 0` (`every_windows_fixture_transcript_parses_without_skipped_lines`).
- **Deviations:**
  - JSON pretty-printing uses serde_json's `"key": value`, where Foundation wrote `"key" : value`; the JSONBeautifier assertions drop that space.
  - Timestamps without fractional seconds also parse; Poirot's formatter required them.
  - A message missing a timestamp no longer drags `startedAt` to the distant past.
  - The searchable-text cap counts `char`s where Swift counted grapheme clusters.
  - System tags are classified as `systemReminder` (`system-reminder`), `commandMessage` (`command-name`, `command-message`, `command-args`) or `other`.
- **Follow-ups:** none.

### M1-13 · Header and summary tiers · 2026-09-25

- **Agent:** rust-engineer (executed by the coordinating agent)   **Reviewers:** pending
- **Status:** done
- **Files touched:** `crates/heimdall-core/src/transcript/{header,summary}.rs`, `crates/heimdall-core/src/transcript/full.rs` (usage counted once per message id), `docs/port-plan.md` (C-24, C-25, D5)
- **Evidence:**
  - `cargo test -p heimdall-core` → 288 passed; `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
  - Carried-over: all 11 parseSummary cases in `summary.rs`. PoirotTests has no header-tier cases; 3 new tests cover field extraction, the sidechain and synthetic filters, and the 50-line limit.
  - `summary_tokens_match_the_full_parse_for_every_fixture_transcript` compares `total_tokens` and the `TokenUsage` breakdown across every transcript under `fixtures/**/projects` (both sidechain modes). The first run failed on a real sub-agent transcript: the full parse counted a split message's usage twice (C-24). Both tiers now count once per message id; regression test `usage_counts_once_when_tool_results_split_a_message`.
  - `summarize` streams records and keeps only counters, the first title and preview, the seen-id set and the capped search text; it never builds a `Vec<Message>`.
  - The header tier reads through `jsonl::read_head` (8 KiB chunks, 50 JSON lines) and stops early once title, preview and start time are found.
- **Deviations:**
  - The header limit counts 50 JSON lines; Poirot counted every line, blank and malformed ones included.
  - `first_prompt` and the sub-agent fields are left for the caller (M1-14), which knows them from the file's location.
- **Follow-ups:** D5 (first-record vs last-record token usage) → user.

### M1-14 · Project discovery, sessions index, sub-agents and project path resolution · 2026-09-25

- **Agent:** rust-engineer (executed by the coordinating agent)   **Reviewers:** pending
- **Status:** done
- **Files touched:**
  - `crates/heimdall-core/src/projects/{mod,discovery,sessions_index,subagents,project_path}.rs`.
  - `crates/heimdall-core/src/models/session.rs`: new field `Session.cwds`.
  - `crates/heimdall-core/src/transcript/summary.rs`: now counts the `cwd` of every record.
- **Evidence:**
  - `cargo test -p heimdall-core` → 309 passed; `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
  - Carried-over: all 17 SessionLoaderTests. Discovery takes an injected folder probe, so the decoding case (`-Users-dev-projects-my-app` → `dev-projects-my-app`) gives the same answer on every OS.
  - Sessions index:
    - It is used when it has entries.
    - Entries whose file is missing are skipped (`index_entries_whose_file_is_missing_are_skipped`).
    - The project path comes from `entries[0].projectPath`, the top-level `projectPath` or the top-level `originalPath`.
  - Sub-agents are read from `<session>/subagents/agent-*.jsonl`, excluding `agent-acompact-*`. `agentType` and `description` come from `.meta.json`, and `parentSessionId` is set.
  - Path order:
    1. The `cwd` whose encoding matches the folder name, case-insensitively; the most frequent wins if several match.
    2. The most frequent `cwd`.
    3. The sessions-index path.
    4. `decode_project_dir_name`.
    - `pathSource` reports which one won.
    - The fixture's `C--Users-fixture-Desktop-tech` resolves to `C:\Users\fixture\Desktop\tech`, even though its most frequent `cwd` is a subfolder (455 records vs 27 + 20).
  - Orphaned `<uuid>.orphaned-…` transcripts are ignored. The `C--…` and `c--…` folders stay separate projects.
  - fixtures/windows:
    - Every project with transcripts reports `transcriptCwd`, with a drive-letter path whose encoding is its folder name, and 0 skipped lines (`windows_fixture_projects_resolve_from_transcript_cwd`).
    - The exception is `c--Users-fixture-Desktop-tech-proj-2`, which holds only memory files, has no `cwd` to read and so reports `decoded`. The criterion as written ("every project") cannot hold for it.
  - `grep -rn "replace('-', \"/\")\|replace(\"-\", \"/\")" crates/heimdall-core/src` → no matches.
- **Deviations:**
  - Projects are sorted by folder name; Poirot used directory order.
  - Index entries are not pre-sorted by `created`, because sessions are sorted by start time at the end anyway.
  - `cwd` counts are gathered during the summary pass, so no transcript is read twice.
- **Follow-ups:** none.

### D5 · Per-message token usage from the final streamed record · 2026-09-26

- **Agent:** rust-engineer (executed by the coordinating agent)   **Reviewers:** pending
- **Status:** done
- **Files touched:** `crates/heimdall-core/src/transcript/{full,summary}.rs`, `docs/port-plan.md` (D5 answered, C-25 updated)
- **Evidence:**
  - The user chose the last record. Both tiers keep the latest usage seen for each message id and sum those. In the full parse, the usage sits on the message's first group only, so a message split by tool results is still counted once.
  - `cargo test -p heimdall-core` → 309 passed; `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
  - `summary_tokens_match_the_full_parse_for_every_fixture_transcript` still passes, so the two tiers agree on every fixture transcript under the new rule.
  - Expectations changed, with the deviation noted on each test:
    - `parse_usage_deduplicated_per_group` → usage (200, 100), total 300 (Poirot: 150).
    - `parse_summary_sums_tokens_per_group` → 750 (Poirot: 600).
    - `usage_counts_once_when_tool_results_split_a_message` → first group carries (2, 209), total 211.
- **Follow-ups:** none.

### M1-15 · History reader with stable per-entry ids · 2026-09-26

- **Agent:** rust-engineer (executed by the coordinating agent)   **Reviewers:** pending
- **Status:** done
- **Files touched:**
  - `crates/heimdall-core/src/data/history.rs`.
  - `crates/heimdall-core/src/lib.rs`: `stable_hash`, the FNV-1a helper, moved out of the todo model to be shared.
  - `crates/heimdall-core/src/models/session_todo.rs`: uses `stable_hash`.
  - `docs/ipc-contract.md`: `HistoryEntryId` format.
- **Evidence:**
  - `cargo test -p heimdall-core -- history session_todo` → 30 passed; `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
  - Carried-over: all 12 HistoryLoaderTests (millisecond timestamps, `pastedContents`, null `pastedContents`, `entryCount`, `uniqueProjects`).
  - Entry ids are `<timestamp ms>-<FNV-1a of the raw line>`. `ids_follow_the_line_not_its_position` inserts a line above and the ids do not change. Identical lines get `-1`, `-2`… (`identical_lines_get_distinct_ids`). This fixes C-23's root cause for the delete in M3-46.
  - `page(offset, limit)` returns the contract's `HistoryPage` with `total` and `skipped`.
  - The Windows fixture history (28 lines, with a `sessionId` key Poirot ignores) parses with 0 skipped.
- **Deviations:**
  - A `pastedContents` value stored as an object keeps its `content`, or its JSON text; Poirot dropped the whole line.
  - Non-UTF-8 lines are skipped and counted instead of failing the file.
  - A missing file is an empty history; other I/O errors are reported.
- **Follow-ups:** none.

### M1-16 · Todo loader · 2026-09-26

- **Agent:** rust-engineer (executed by the coordinating agent)   **Reviewers:** pending
- **Status:** done
- **Files touched:** `crates/heimdall-core/src/data/todos.rs`
- **Evidence:**
  - `cargo test -p heimdall-core data::todos` → 14 passed; `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
  - Carried-over: all 13 TodoLoaderTests, including `<sessionId>-agent-<agentId>.json` with UUID session and agent ids, merging agent files, and ignoring unrelated files.
  - `load_all` groups files by the part before `-agent-` and leaves out sessions whose files are empty.
- **Deviations:**
  - Files are read in name order, so the merged order is the same on every OS; Poirot used directory order.
  - An item with an unknown status drops only that item (`TodoItem::list_from_json`, M1-09); Poirot dropped the whole file.
  - Unreadable files and bad items are counted in `skipped`.
- **Follow-ups:** none.

### M1-17 · Debug log loader · 2026-09-26

- **Agent:** rust-engineer (executed by the coordinating agent)   **Reviewers:** pending
- **Status:** done
- **Files touched:** `crates/heimdall-core/src/data/debug_log.rs`, `crates/heimdall-core/src/paths.rs` (`plain_file_name`, the guard for ids joined into file names)
- **Evidence:**
  - `cargo test -p heimdall-core -- debug_log paths::` → 39 passed; `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
  - Carried-over: all 22 DebugLogLoaderTests (parsing, unknown level → debug, brackets inside messages, `hasLog`, `allSessionIds`, and paging by offset and limit with `total`).
  - `page` streams the file line by line and keeps only the requested window. The contract's optional `level` filter is supported: `total` counts matching entries, and ids stay file positions (`a_level_filter_pages_matching_entries_and_keeps_file_ids`).
  - A session id that is not a plain file name, such as `../secrets`, is rejected before any path is built (`session_ids_cannot_leave_the_debug_folder`).
- **Deviations:**
  - Timestamps without fractional seconds also parse.
  - Invalid UTF-8 is replaced rather than ending the read.
  - `allSessionIds` is sorted.
  - The task text's "multi-line entries" means multiple lines, as in Poirot's `parse_multipleLines`. Poirot has no continuation lines, and neither does Heimdall.
- **Follow-ups:** none.

### M1-18 · Facets loader · 2026-09-26

- **Agent:** rust-engineer (executed by the coordinating agent)   **Reviewers:** pending
- **Status:** done
- **Files touched:** `crates/heimdall-core/src/data/facets.rs`
- **Evidence:**
  - `cargo test -p heimdall-core data::facets` → 11 passed; `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
  - Carried-over: all 10 FacetsLoaderTests.
  - `load_all` keys by the `session_id` inside each file, as Poirot does, and counts files that do not parse in `skipped`.
  - `load` rejects a session id that is not a plain file name.
- **Deviations:**
  - The model reads missing keys as defaults (M1-09), so a file with no `session_id` is keyed by its file name instead of being dropped.
- **Follow-ups:** none.

### M1-19 · File history from snapshot and delta records · 2026-09-26

- **Agent:** rust-engineer (executed by the coordinating agent)   **Reviewers:** pending
- **Status:** done
- **Files touched:** `crates/heimdall-core/src/data/file_history.rs`
- **Evidence:**
  - `cargo test -p heimdall-core file_history` → 8 passed; `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
  - Carried-over: all 5 FileHistoryLoaderTests. `sameFile` is now `paths::same_file`, tested with the same four cases.
  - Both `file-history-snapshot` and `file-history-delta` records are read. Backups with no file name, version 0 or no time are dropped.
  - File matching:
    - It uses `paths::same_file`.
    - In `windows_paths_merge_with_relative_tracked_names`, the snapshot key `C:\Users\me\proj\src\Main.rs` and the delta path `src\main.rs` merge on Windows (case-insensitive) but stay apart on Linux.
    - A repeated backup counts once.
  - On the Windows fixture, every version of sessions `54a79a0b…` and `ab8bf07e…` resolves to a backup file on disk, with 0 skipped lines.
  - Transcript lines are read into a small typed record, so the rest of each line is skipped rather than built into JSON values.
- **Deviations:**
  - Every version of a merged file carries the file's final (longest) name; Poirot kept the name each version was first seen under.
  - Transcript lookup visits project folders in name order.
  - Session and backup names must be plain file names.
- **Fixture privacy fix (same day):**
  - Tracked file paths in the Windows capture contained hyphen-encoded project folders (`c--Users-fixture-Desktop-tech-<real name>`) that the scrubber had not pseudonymised.
  - The scrubber now applies the folder pseudonyms inside paths (new test case), and the 126 occurrences in the fixture were rewritten to the existing `proj-10` and `proj-11` pseudonyms.
  - The history was rewritten before any push, so no commit contains them.
- **Follow-ups:** none.

### M1-20 · Stats cache staleness and local recomputation · 2026-09-26

- **Agent:** rust-engineer (executed by the coordinating agent)   **Reviewers:** pending
- **Status:** done
- **Files touched:** `crates/heimdall-core/src/data/{stats,stats_cache}.rs`, `crates/heimdall-core/src/projects/discovery.rs` (`is_uuid` made public), `docs/port-plan.md` (C-26)
- **Evidence:**
  - `cargo test -p heimdall-core -- data::stats` → 15 passed; `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
  - Carried-over: all 6 StatsComputerTests and the 5 loader cases of StatsCacheTests.
  - Staleness matches StatsCacheLoader.swift:36-51:
    - It is stale when the token series stops before activity, or either series ends before the latest session day (found from the newest main transcript's modification time).
    - A fresh cache is kept. A stale one is replaced by a recomputation, or kept when there is nothing to recompute.
  - Recomputation:
    - It reads only UUID-named transcripts and non-sidechain, non-synthetic records.
    - It fills per-day message, session and tool-call counts, per-model tokens and web searches, the hour histogram, the longest session, the first session date and the totals.
    - `costUSD` is 0, with the 200k/1M context window and 32k max output as in Poirot.
  - Days and hours use an injected time zone (`compute_in`); the app uses local time. `days_and_hours_follow_the_time_zone` checks a UTC 23:30 record landing on the next day at +02:00.
  - Transcript lines are read into a small typed record, and message content is only scanned to count `tool_use` blocks.
  - The Windows fixture recomputes with sorted days and non-zero output tokens.
- **Deviations:**
  - Usage counts once per message id, from its last record (D5 applied to analytics; C-26). A message that appears in two transcripts (a resumed session) is counted once. `a_message_split_over_records_counts_its_last_usage_once` shows what Poirot would have reported. The carried-over tests have one record per message, so their numbers are unchanged.
  - Session files are matched case-insensitively, as Swift's `UUID(uuidString:)` did.
- **Follow-ups:** none.

### M1-22 · Safe JSON write primitive · 2026-09-26

- **Agent:** rust-engineer (executed by the coordinating agent)   **Reviewers:** pending (security-auditor, architect-reviewer)
- **Status:** done
- **Files touched:** `crates/heimdall-core/src/config/write/{json_doc,atomic}.rs`, `Cargo.toml` (serde_json `float_roundtrip`)
- **Evidence:**
  - `cargo test -p heimdall-core config::write` → 11 passed; the full crate → 410 passed; `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
  - A corrupt target (`fixtures/synthetic/corrupt/claude.json`) returns `AppError::Parse` with the path. The file stays byte-identical, and no `.heimdall.bak` or temp file appears.
  - Round trip of `fixtures/windows/claude.json`:
    - Unmodified, it renders byte for byte and `save` reports `unchanged`.
    - Changing `numStartups` changes exactly that one line.
    - This needed serde_json's `float_roundtrip` feature: the default parser rounded `1105.2999999999997` to a neighbouring float, which printed as `…995`.
  - `<file>.heimdall.bak` (the ADR 0009 name) is written before the first mutating save and never replaced.
  - Two-space, four-space, tab, compact, CRLF, BOM and no-trailing-newline files all render byte for byte and keep their style after an edit.
  - Unix mode 0600 survives (a `cfg(unix)` test, which runs in Linux CI).
  - Windows:
    - A handle held without `FILE_SHARE_DELETE` makes the rename retry with backoff for 2 s, then try `ReplaceFileW` once, then return `AppError::Busy` with the path. The original stays intact and no temp file remains.
    - A handle released after 150 ms lets the write through.
  - The temp file sits in the target's folder and is removed on every failure; the `cfg(unix)` rename-over-a-folder test checks this.
  - Also from ADR 0009:
    - Rule 8: `save` refuses with `Busy` when the file changed since it was read.
    - A missing file is created (two spaces, trailing newline) only when the caller allows it.
    - A top level that is not an object cannot be edited.
- **Follow-ups:** the settings, MCP and file writers (M3-40 to M3-42) build on `JsonDoc`.

### M1-23 · Frontmatter parser · 2026-09-26

- **Agent:** rust-engineer (executed by the coordinating agent)   **Reviewers:** pending
- **Status:** done
- **Files touched:** `crates/heimdall-core/src/config/read/frontmatter.rs`
- **Evidence:**
  - `cargo test -p heimdall-core frontmatter` → 14 passed; `cargo clippy -p heimdall-core --all-targets -- -D warnings` → exit 0.
  - Carried-over: all 12 FrontmatterParserTests, including `>` and `|` block scalars, a block followed by a key, a block at the end, and a colon inside a value.
  - CRLF: `crlf_files_parse_like_lf_files` gives the same metadata as the LF version.
- **Deviations:**
  - Poirot split lines on `\r` as well as `\n`, so a CRLF file ended a block scalar after its first continuation line; lines here split on `\n` with `\r` trimmed.
  - Metadata is a sorted map.
- **Follow-ups:** none.

### M1-24 · Loaders for commands, skills, plans, output styles and agents · 2026-09-26

- **Agent:** rust-engineer (executed by the coordinating agent)   **Reviewers:** pending
- **Status:** done
- **Files touched:**
  - `crates/heimdall-core/src/config/read/{mod,commands,skills,plans,output_styles,agents}.rs`.
  - `crates/heimdall-core/src/models/config/scope.rs`: `ConfigKind`, `ProjectContext` and `ConfigRef`, in the contract's shapes.
  - `crates/heimdall-core/src/models/config/{command,skill,output_style,plan,sub_agent}.rs`: a `config_ref` field.
- **Evidence:**
  - `cargo test -p heimdall-core` → 431 passed; `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
  - Tests on `fixtures/synthetic` cover global and project scope for commands (`/review`, `/deploy`), skills (`pdf-tools`, `db-migrate`) and output styles (`concise`, `terse`). Plans and agents are global-only, as in Poirot.
  - Output styles skip `README.md`. `refactor-auth-module` becomes "Refactor Auth Module". The four built-in agents come first, then custom `reviewer` with its tools, colour and model.
  - Each item carries a `ConfigRef` (kind, `ProjectContext` for project items, slug). `config::read::item_path` is the only way back to a file: it accepts a plain-file-name slug only and resolves it under the kind's folder (`refs_resolve_only_inside_their_folder`). Item `file_path` stays display-only.
  - Items are sorted case-insensitively by name. Files that are not UTF-8 are counted in `skipped`.
- **Deviations:**
  - An agent `memory` value Poirot does not know (the fixture's `project`) is left unset, as Poirot's enum did.
  - `skills/synced/<id>/<name>/` is not read, as in Poirot (parity E12).
- **Follow-ups:** Claude Code's synced-skills layout (fixtures/windows MANIFEST fact 4) is outside parity; record it as a Later candidate if the user wants those skills listed → product decision.

### M1-25 · Settings and hooks readers · 2026-09-26

- **Agent:** rust-engineer (executed by the coordinating agent)   **Reviewers:** pending
- **Status:** done
- **Files touched:** `crates/heimdall-core/src/config/read/{settings,hooks}.rs`
- **Evidence:**
  - `cargo test -p heimdall-core -- config::read::settings config::read::hooks` → 6 passed; `cargo clippy -p heimdall-core --all-targets -- -D warnings` → exit 0.
  - The global hooks (`PreToolUse` with two handlers, `SessionStart` with a null matcher) and the project hook (`PostToolUse`, matcher `Bash`) load from the synthetic fixtures with their scope and group index.
  - http and command handlers keep `url`, `timeout` and `statusMessage`. The fixture's `websocket` handler is dropped, as Poirot's enum did.
  - Unknown handler keys (`async`, `env`) are kept in `extra` and named in `extraKeys`.
  - `project_model` matches `loadProjectModel`: `claude-sonnet-4-6` for the fixture repo, `None` when missing.
- **Deviations:**
  - Hook events are kept as text, so an event newer than Heimdall still appears; Poirot dropped unknown events.
  - A settings file that does not parse is a `Parse` error from `settings::load` and counts in `skipped` for hooks, instead of silently reading as empty.
- **Follow-ups:** none.

### M1-26 · MCP server (five sources) and plugin readers · 2026-09-26

- **Agent:** rust-engineer (executed by the coordinating agent)   **Reviewers:** pending (security-auditor)
- **Status:** done
- **Files touched:** `crates/heimdall-core/src/config/read/{mcp,plugins}.rs`
- **Evidence:**
  - `cargo test -p heimdall-core` → 449 passed; `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
  - Carried-over: all 8 ClaudeConfigLoaderMCPTests, decoding into the M1-10 models (env and header values are `Secret`s).
  - `five_sources_follow_poirots_precedence` covers all five sources:
    - User servers load from `~/.claude.json`.
    - A local `projects.<path>.mcpServers` entry overrides the user one of the same name.
    - `.mcp.json` adds `team-tool` but does not override `github`.
    - An enabled plugin's server appears as `plugin:context-mode:context-mode` ("Context Mode"); a disabled plugin's is hidden.
    - `claude.ai Gmail` from the auth cache appears as a cloud integration with `needsAuth`.
  - Tool permissions (`mcp__<server>__<tool>` in `settings.json`) group by server; a server without any entry is a wildcard.
  - `serialised_servers_never_carry_secret_values` loads the synthetic fixtures (three sentinel secrets) and finds no `SENTINEL-SECRET` in the JSON; `envKeys` lists the names.
  - Plugins list one entry per installed version, with `enabledPlugins` from the global settings.
- **Deviations:**
  - Local servers are found with `paths::same_path`. On Windows, Claude Code keys projects as `C:/Users/…` while paths elsewhere use `\` and either drive-letter case, and Poirot's exact string match would miss them (`windows_project_keys_match_whatever_the_separator`).
  - Sources are the contract's five values (`local` and `project` kept apart); Poirot folded them into `user`.
  - A config file that does not parse counts in `skipped`.
- **Follow-ups:** none.

### M1-27 · Memory and models readers · 2026-09-26

- **Agent:** rust-engineer (executed by the coordinating agent)   **Reviewers:** pending
- **Status:** done
- **Files touched:**
  - `crates/heimdall-core/src/config/read/{memory,models}.rs`.
  - `docs/ipc-contract.md`: `ModelsView` now carries the catalog entries with their descriptions and strengths, plus default and project display names.
  - `docs/port-plan.md`: C-27.
  - `docs/tasks/M3-40-*.md`: a criterion for C-27.
- **Evidence:**
  - `cargo test -p heimdall-core -- config::read::memory config::read::models` → 11 passed; `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
  - Carried-over: all 7 MemoryLoaderTests (`MEMORY.md` first, then names, `.txt` ignored, `projectId`, custom folders with `custom:<path>` and the folder label, counts, missing folders). The logic part of ClaudeModelCatalogTests already passes in `models::catalog` (M1-10).
  - Custom folders come in as a parameter (`sources(projects_dir, custom_folders)`, `load_folder`); core reads no preferences.
  - `MemorySource` ids are `project:<ProjectId>` and `folder:<index>`, as in the contract. A project id that is not a plain file name is rejected.
  - The Windows fixture's project memory folders list and load.
- **Deviations:**
  - The default model is the `model` in the global `settings.json`, shown as a display name. Poirot always showed "Opus 5" (C-27).
  - Catalog display names written by Poirot (`Sonnet 5`) are recognised as they are; ids and aliases go through `friendly_name`.
- **Follow-ups:** the model write must store an id or alias (C-27) → M3-40.

### M1-28 · MCP process status on sysinfo · 2026-09-26

- **Agent:** rust-engineer (executed by the coordinating agent)   **Reviewers:** pending
- **Status:** done
- **Files touched:** `crates/heimdall-core/src/mcp_status.rs`
- **Evidence:**
  - `cargo test -p heimdall-core mcp_status` → 19 passed; `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
  - Carried-over: all 14 MCPServerStatusCheckerTests, run against an injected process list and auth-cache set.
  - Pattern building ports MCPServerStatusChecker.swift:160-214 (`patterns_follow_the_runner_rules`), and matching runs on each process's joined command line. `running_command_lines()` reads it through `sysinfo`; `the_real_process_list_is_readable` passes on Windows.
  - Windows shape: `npx.cmd -y @modelcontextprotocol/server-filesystem` matches `node.exe …\npm-cache\_npx\…\node_modules\.bin\..\@ModelContextProtocol\server-filesystem\…` (case-insensitive on Windows) and is unknown without that process.
  - HTTP and SSE status is URL well-formedness only (`url::Url::parse`), labelled P-5 in the module and at the check.
  - The "any user stdio server connected means the idle ones failed" heuristic is kept (`a_connected_stdio_server_marks_the_idle_ones_failed`).
- **Deviations:**
  - `npx.cmd`, `npx.exe` and `uvx.exe` count as the npx and uvx runners.
  - Pattern parts are matched literally: Poirot passed arguments to `pgrep` unescaped, so `server.js` also matched `serverXjs` (`literal_parts_do_not_act_as_regex`).
- **Follow-ups:** none.

### M1-29 · Fuzzy and exact matcher with UTF-16 highlight ranges · 2026-09-26

- **Agent:** rust-engineer (executed by the coordinating agent)   **Reviewers:** pending
- **Status:** done
- **Files touched:** `crates/heimdall-core/src/search/{matcher,mod}.rs`
- **Evidence:**
  - `cargo test -p heimdall-core search::` → 11 passed; `cargo clippy -p heimdall-core --all-targets -- -D warnings` → exit 0.
  - Carried-over: the 4 matching assertions of HighlightedTextTests, and the 6 `fuzzyMatch` cases deferred from PlanTests and MemoryFileTests in M1-10.
  - The worked examples in `docs/spec/search-overlay.md` reproduce exactly:
    - fuzzy `swordfish`/`ssh` scores 37 with ranges `[0,1),[7,8),[8,9)`;
    - exact `ssh config` gives `[0,3)`;
    - a one-character query scores 10.
  - Ranges are UTF-16: in `日本語 🎉 SSH tunnel`, "SSH" is `[7,10)` (the emoji counts 2), "🎉" is `[4,6)` and "本語" is `[1,3)`.
  - Exact mode highlights every non-overlapping occurrence; fuzzy highlights each matched character, as the spec's §2.4 describes.
- **Deviations:** characters are compared per Unicode scalar after lower-casing each one, so ranges map to the original text. Poirot compared grapheme clusters; the two differ only inside multi-scalar emoji and combining sequences.
- **Follow-ups:** none.

### M1-21 · Tray snapshot aggregation · 2026-09-26

- **Agent:** rust-engineer (executed by the coordinating agent)   **Reviewers:** pending
- **Status:** done
- **Files touched:** `crates/heimdall-core/src/data/tray.rs`
- **Evidence:**
  - `cargo test -p heimdall-core data::tray` → 12 passed; full crate → 502 passed; `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
  - Carried-over: all 7 MenuBarStateTests (defaults, sort by most recent, capped at 10, project info carried, empty projects, blank and whitespace queries return everything unfiltered).
  - Recent sessions are sorted by `startedAt` descending, capped at 10, then optionally fuzzy-filtered on title or project name with `search::matcher` (`filter_matches_project_name_or_session_title`). The filter only narrows that capped top 10, as Poirot's `filteredSessions` does over `recentSessions`: an 11th, older session matching the query is not pulled back in (`filter_only_narrows_the_capped_top_ten`).
  - `TraySnapshot` and `RecentSession` match the contract shapes exactly (`projectId`, `projectName`, `session`, plus `totalSessions`, `totalMessages`, `todayMessages`, `recent`).
- **Deviations:**
  - `todayMessages` is not in Poirot's `MenuBarState`, which showed only total sessions, total messages, total tokens and cost; the contract added it for the tray header. It is today's entry in `stats.dailyActivity`'s message count, 0 without one; `snapshot_in` takes an injected time zone so "today" is testable (same pattern as M1-20).
  - `TraySnapshot` leaves out total tokens and cost, which Poirot's header also showed; not in the contract, so left for a follow-up if the tray design wants them.
- **Follow-ups:** none for this task; M3-30 wires `get_tray_snapshot` to this function with the live projects, stats and query.

### M1-30 · Search providers for all 16 categories · 2026-09-26

- **Agent:** rust-engineer (executed by the coordinating agent)   **Reviewers:** pending
- **Status:** done
- **Files touched:**
  - `crates/heimdall-core/src/search/{mod,providers}.rs`.
  - `docs/ipc-contract.md` unchanged — the new types (`SearchHit`, `SearchAction`, `SearchCategory`, `SearchResults`, `NavScreen`, `MemoryFileLocator`) already matched the contract's shapes exactly; verified field by field while writing the module.
- **Evidence:**
  - `cargo test -p heimdall-core search::` → 32 passed; full crate → 523 passed; `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
  - One test per category, all on `fixtures/synthetic` (the fixture's own "auth bug in login.py" story gives every category a real hit to find): sessions, debug logs, facets (AI summaries), file changes, todos, history, commands, skills, plans, memory (content fallback), MCP servers, plugins, output styles, hooks, models (curated catalog only), sub-agents (built-in + custom).
  - Caps: `groups_follow_the_fixed_category_order_and_cap_at_five` checks the 16-category order is always `SearchCategory::ALL`'s order and every group has at most 5 hits. `file_history_scan_is_capped_and_deterministic` checks the file-history folder scan stops at 50 even with 60 present, and is sorted (Poirot's own directory order is filesystem-native and not guaranteed).
  - Highlights always come from matching the title directly against the query, never re-derived from whichever field produced the score (spec §2.4) — built into `highlights()`, used identically by every category.
  - Ties keep provider append order (a stable sort), matching Swift's stable `sort`.
  - Quick actions ("Toggle Menu Bar Icon", "Add MCP Server") are correctly left out, as the spec documents they wait on M3-58/M3-48.
- **Deviations:**
  - `Plan.content.prefix(200)` (Swift Characters/grapheme clusters) becomes 200 Unicode scalars.
  - `subtitle` is typed `Option<String>` per the contract but every provider always populates it (possibly with an empty string), matching Poirot's non-optional field.
  - File-history session folders are sorted before the 50-cap, for determinism across OSes.
- **Follow-ups:** none; `search.rs` (M1-46) assembles `SearchInput` from the live loaders and calls `providers::search`.

### M1-31 · Cross-platform PATH search for claude and editor CLIs · 2026-09-26

- **Agent:** rust-engineer (executed by the coordinating agent)   **Reviewers:** pending
- **Status:** done
- **Files touched:** `crates/heimdall-core/src/os_integration/cli_locate.rs`
- **Evidence:**
  - `cargo test -p heimdall-core cli_locate` → 12 passed; full crate → 535 passed; `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
  - No Swift test existed for this (Poirot only ever probed four hardcoded macOS paths); this is new Rust-only coverage.
  - `find_on_path`/`find_claude_cli`/`find_editor_cli` take `PATH`, `PATHEXT`, `home` and an `exists` check as plain values, so every OS's layout is tested from any host (Windows PATHEXT resolution, VS Code's `code.cmd` shim, Linux and macOS fallback directories, empty `PATH` segments, PATH taking precedence over every fallback directory).
  - Fallback directories match the ADR 0003 table exactly (`fallback_dirs_match_the_adr_0003_table`): Windows `%APPDATA%\npm`; Linux `~/.local/bin`, `~/.npm-global/bin`; macOS `/opt/homebrew/bin`, `/usr/local/bin`, `~/.local/bin`. Editor CLIs get no fallback directories, only `PATH`.
- **Deviations:** none; this replaces Poirot's four hardcoded macOS-only paths outright, so there is no literal behaviour to preserve beyond the ADR's stated fallback list.
- **Follow-ups:** `os_integration::launch` (M1-32) will call `find_editor_cli` when a URL scheme fails; `app.rs` (M1-40) will call `find_claude_cli` for `CliStatus`.

### M1-32 · Validated argv builders for terminals and editors · 2026-09-26

- **Agent:** rust-engineer (executed by the coordinating agent)   **Reviewers:** pending (security-engineer, security-auditor)
- **Status:** done
- **Files touched:** `crates/heimdall-core/src/os_integration/launch.rs`
- **Evidence:**
  - `cargo test -p heimdall-core os_integration::launch` → 21 passed; full crate → 556 passed; `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
  - No builder returns a shell string: `Launch` is `Argv { program, args, cwd }` or `Url(String)`; every terminal and editor test matches on the `Argv`'s fields directly, never a joined string.
  - `validate_launch_path`/`validate_cwd` reject a relative path, a leading `-`, a UNC path, and a target the caller's own filesystem check says is the wrong kind (file vs directory); every builder calls one of them internally, so an unsafe input never reaches argv construction.
  - Session ids: `valid_session_id` accepts the contract's UUID and `agent-` forms and rejects uppercase hex, a bare `agent-`, a space inside the suffix, and a 129-character suffix; `resume_command_text` refuses to build `claude --resume <id>` from anything else.
  - Cross-OS table tests: all three Windows terminals, all five Linux terminals, and all six macOS terminals (`open -a <App> <cwd>`) match the ADR 0003 table exactly; a terminal not offered on the given OS is rejected. `windows_terminal_never_lets_a_semicolon_split_the_command` and `dangerous_characters_survive_as_one_intact_argv_element_on_every_os` push a cwd containing spaces and `;&|%^"` through every OS's terminals and check it arrives as one untouched argv element.
  - Editors: `vscode://file/…`, `cursor://file/…` and `zed://file/…` build with percent-encoding and a normalised Windows drive path (`vscode://file/C:/Users/…`); Xcode has no URL scheme, matching Poirot. `editor_cli_argv` reproduces each editor's Poirot argv shape (`--goto` for vscode/cursor, bare `path:line` for zed, `--line <n> <path>` for xed) and refuses to name a `.cmd` file as the program.
- **Deviations:**
  - Poirot's AppleScript "run this command" path is dropped outright (port-plan C-5; it had no callers). No command is ever injected into a terminal; the resume command goes to the clipboard instead.
  - The exact URL scheme and percent-encoding are new (Poirot never used URL schemes for editors, only bundle-id app-opening and CLI shims) and are exactly what port-plan assumption **A-17** leaves open; M3-54 will confirm or correct them against the real editors.
  - macOS `open -a` app names are a judgment call pending the same confirmation: `Terminal`, `iTerm` (not "iTerm2"), `Warp`, `Ghostty`, `kitty` (lower-case), `Alacritty`.
- **Follow-ups:** M3-54 (editor URL scheme verification, already tracked as A-17) may change `editor_url`'s scheme or encoding; `src-tauri/src/platform/*.rs` (M3-52) is the first real caller, spawning `Argv` and opening `Url`.

### M1-33 · External URL allowlist · 2026-09-26

- **Agent:** rust-engineer (executed by the coordinating agent)   **Reviewers:** pending (security-auditor)
- **Status:** done
- **Files touched:** `crates/heimdall-core/src/os_integration/links.rs`
- **Evidence:**
  - `cargo test -p heimdall-core os_integration::links` → 7 passed; full crate → 563 passed; `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
  - Corpus test rejects `file:`, `javascript:`, `data:`, a custom scheme (`vscode:`), `ftp:`, `tel:`; rejects three backslash-smuggled UNC-host shapes; rejects embedded credentials (`user:pass@` and bare `user@`); rejects an over-long URL at the 2048-character cap and accepts one just under it; rejects an empty and an unparsable string.
  - Accepts ordinary `http`, `https` and `mailto` links, including a case-varied scheme (`HTTPS://Example.COM`) and a `mailto:` address whose own `@` sign does not trip the credentials check (mailto has no authority component to hold one).
  - The backslash check runs on the raw string, before `url::Url::parse`, because the WHATWG URL spec normalises `\` to `/` for `http`/`https` — parsing first would already have resolved a UNC-shaped string into an ordinary host, hiding exactly the mismatch this control exists to catch.
- **Deviations:** the 2048-character length cap is Heimdall's own choice; ADR 0010 says "over-long" without a number.
- **Follow-ups:** `open_external_url` (M2-11) and the update/toast call sites (M2-12, M4-04) call this before ever reaching the opener plugin.

### M1-34 · Changed-path to data-area classifier · 2026-09-26

- **Agent:** rust-engineer (executed by the coordinating agent)   **Reviewers:** pending
- **Status:** done
- **Files touched:** `crates/heimdall-core/src/classify.rs`
- **Evidence:**
  - `cargo test -p heimdall-core classify` → 10 passed; full crate → 571 passed; `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
  - A table test covers every area `classify` can produce (20 of the contract's 21 — `prefs` is emitted by `prefs.rs`, M1-39, never from a path) from both a Windows-shaped and a Unix-shaped Claude home, plus: `settings.json` classifying as both `settings` and `hooks`; `~/.claude.json` classifying as `mcp` even though it sits outside the home root; `skills/synced/` and a session's `tool-results/` folder correctly producing nothing; unrelated paths (including a sibling folder that only shares a name prefix, `.claude-extra`) producing nothing; Windows and macOS comparing the root case-insensitively while keeping the matched remainder's own case; and each `Area` variant's exact contract spelling (`debugLogs`, `fileHistory`, `mcp`, …).
  - ADR 0006 states the *mechanism* (one classifier, per-path) but not the literal path-to-area table; this task had to design that table from the Claude home layout established across M1-06 through M1-28, which is why it is documented in full above rather than cited to a Swift line.
- **Deviations / design decisions (none are a Swift port; ADR 0006 has no Poirot equivalent):**
  - A changed path can classify to more than one area. Only two do: the bare `~/.claude` root (`all`, a rare, drastic event) and `settings.json` (`settings` and `hooks`, since one file holds both and each has its own store to notify).
  - `todos/<sessionId>-agent-<id>.json` classifies to the *parent* session's id (`data::todos::session_id_of`), matching how the loader already merges agent files under one session's todo list.
  - `projects/<name>` (the bare project folder) classifies to `projects` with a null key, per the contract's own key table, even though a per-project key would also have been a defensible reading.
- **Follow-ups:** the watcher (M1-47) calls `classify` per changed path and is the one place custom memory folders and a project's own `.claude/` config get their events — ADR 0006 does not list either as a watched root, so those stay outside `classify`'s scope by design.

### M1-35 · Line diff port · 2026-09-26

- **Agent:** rust-engineer (executed by the coordinating agent)   **Reviewers:** pending
- **Status:** done
- **Files touched:** `crates/heimdall-core/src/diff.rs`
- **Evidence:**
  - `cargo test -p heimdall-core diff::` → 17 passed; full crate → 589 passed; `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
  - Carried-over: all 13 LineDiffTests. `sequentialIds` had no wire-format equivalent (`DiffLine` carries no id; the contract drops it, and the frontend can key rows by array index) and was ported as an order/length check instead.
  - Inputs over 5000 lines are truncated on each side independently and `truncated: true` is set (`inputs_over_the_cap_are_truncated_and_flagged`); `maxLineNumber` is the largest line number on either side (`max_line_number_is_the_largest_line_seen_on_either_side`).
  - `a_replacement_reports_the_removal_before_the_insertion` locks in the tie-break Poirot's own reconstruction loop used (a pending removal is checked before a pending insertion at the same position), reproduced here by the DP backtrack's own tie rule.
  - `kind_names_match_the_contract` confirms `context`/`add`/`remove` — the contract's names, not Poirot's `context`/`added`/`removed`.
- **Deviations:**
  - Swift used its standard library's `CollectionDifference` (a Myers diff); this ports the same *behaviour* (a minimal line-level diff) from a textbook LCS dynamic program instead, since Rust's standard library has no equivalent. Verified by hand-tracing the DP table against every carried-over test, including the two ambiguous-tie cases (`insertionInMiddle`, `deletionInMiddle`); a genuinely adversarial input could in principle produce a different, equally minimal diff than Swift's.
  - **ponytail:** the DP table is `O(old.len() × new.len())` time and space — bounded by the 5000-line cap to at most ~100 MB, reached only by two capped files sharing no line at all, never by a real edit. Upgrade path: Myers' linear-space (divide-and-conquer) algorithm, if this ever proves to matter.
  - `truncated`/`maxLineNumber` are new fields the contract adds; Poirot silently truncated with no flag and left the gutter-width computation to the view.
- **Follow-ups:** none.

### M1-36 · Session Markdown and patch export · 2026-09-26

- **Agent:** rust-engineer (executed by the coordinating agent)   **Reviewers:** pending
- **Status:** done
- **Files touched:** `crates/heimdall-core/src/export/{session_markdown,patch}.rs`
- **Evidence:**
  - `cargo test -p heimdall-core export::` → 16 passed; full crate → 605 passed; `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
  - Poirot has no `SessionExporterTests` at all (checked: no such file exists), so every test here is new coverage written directly against `Services/Export/SessionExporter.swift`, run against a real, varied session from `fixtures/screenshot` (`conv-001.jsonl`: text, thinking, a Bash call, an Edit call with real diff data, tool results).
  - Each of the four toggles (`timestamps`, `toolResults`, `thinking`, `tokenUsage`) is tested on and off against that real session. Bash and Edit tool blocks render their own shapes (a fenced `bash` block; a `` ```diff `` block from the edit's own old/new strings); a non-Bash, non-Edit tool lists its file path once and its remaining input key-sorted (a BTreeMap, where Poirot's raw Swift dictionary had no defined order at all — a deliberate improvement, not a literal port). `tool_block_to_markdown` (used elsewhere by name, e.g. a "copy tool block" action) shares the same rendering.
  - `to_patch` builds hunks with the exact imperative walk `SessionExporter.buildHunks` used, index for index, including its "no changes found → one hunk of pure context" fallback. `the_patch_applies_cleanly_with_git_apply` and a second case with both an added and a removed line actually shell out to `git apply --check` in a throwaway repo and check its exit code, per the acceptance criterion.
  - Fixed while writing the patch test: a file's own trailing newline is not a content line. Diffing the raw strings (as a naive `split('\n')`, matching Poirot's own `components(separatedBy:)`, would) added a spurious trailing context line both sides "agree" on, which was enough for `git apply` to reject the hunk's line count against the real file on disk. `to_patch` strips one trailing newline from each side before diffing; `session_markdown.rs`'s own diff rendering (which Poirot's `LineDiffTests` already covers byte-for-byte) is untouched.
- **Deviations:**
  - The header's long date (Poirot: locale- and timezone-dependent `DateFormatter(.long, .short)`, with no fixed rendering even in Swift) becomes a fixed English format ("September 26, 2026 at 2:30 PM") in an injected time zone, the same pattern M1-20/M1-21 used for testability.
  - A message with no timestamp (Heimdall's `Option<DateTime<Utc>>`, versus Poirot's always-present `Date` defaulting to `.distantPast`) leaves the `*HH:MM:SS*` line out rather than printing a fake time.
  - Tool input truncation (500 characters) and tool-result truncation (2000 characters) count Unicode scalars; Poirot counted Characters (grapheme clusters), the same difference already noted for other 200/500-character truncations in this port.
- **Follow-ups:** none.

### M1-37 · IPC contract frozen as compiled stubs and generated bindings · 2026-09-26

- **Agent:** rust-engineer (executed by the coordinating agent)   **Reviewers:** pending (architect-reviewer, security-auditor)
- **Status:** done
- **Files touched:** `src-tauri/src/commands/{mod,app,prefs,sessions,session_data,history,config_read,config_write,mcp,analytics,tray,search,export,launch,update}.rs`, `src-tauri/src/events.rs`, `src-tauri/src/lib.rs`, `src-tauri/src/bin/generate_bindings.rs` (new), `src/ipc/bindings.ts` (new, generated), `scripts/check-bindings.mjs` (new); plus every heimdall-core file needed to define the contract's remaining wire types: `crates/heimdall-core/src/app_shell.rs` (new — `AppInfo`, `CliStatus`, `SidebarCounts`, `UpdateStatus`, `AppAction`), `error.rs` (`AppError::not_implemented`), `classify.rs` (`Area`/`DataChange` gain `Deserialize`), `export/mod.rs` (`ExportDestination`, `CsvKind`, `PngKind`), `os_integration/launch.rs` (`EditorTarget`, `RevealTarget`), `search/mod.rs` (`NavTarget`, `Deserialize` on `NavScreen`), `models/{message,session_todo,file_version,prefs,stats_cache}.rs` and `models/config/{scope,plugin,memory_file,mcp,hook,sub_agent}.rs` (the remaining `*Summary`/`*Detail`/`*Input` types the contract's §5 sketch names).
- **Evidence:**
  - Every one of the 65 commands in `docs/ipc-contract.md` §6 exists with the contract's exact argument and return types, registered through `tauri_specta::collect_commands!` in `commands/mod.rs`, each returning `Err(AppError::not_implemented("<command>"))` (a new helper wrapping `AppError::InvalidInput`, chosen since none of the six kinds mean "not implemented yet" and this keeps every stub grep-able and one line).
  - The 4 events (`data-changed`, `watcher-status`, `menu-action`, `navigate`) are typed structs in `events.rs` deriving `tauri_specta::Event`; its derive kebab-cases the struct name by default (`DataChanged` → `"data-changed"`), which is exactly the contract's spelling, so no `#[tauri_specta(event_name = …)]` override was needed.
  - `src/ipc/bindings.ts` (954 lines) is generated and committed. Spot-checked against the contract table line by line: every command's camelCase wrapper, argument names and nested tagged-union shapes (`ProjectContext`, `SearchAction`, `NavTarget`, …) match; `TodoStatus`'s genuine serialize/deserialize asymmetry (`inProgress` vs `in_progress`, since the same struct both parses the on-disk file and serves IPC) correctly cascades into `TodoItem_Serialize`/`TodoList_Serialize`/`TodoGroup_Serialize` phase-split aliases, and `get_session_todos`/`list_all_todos` correctly resolve to the `_Serialize` phase — this is tauri-specta's documented "phase-specific types" behaviour, not a bug (traced in full before accepting it).
  - `node scripts/check-bindings.mjs` regenerates via a dedicated helper binary (`cargo run -p heimdall-desktop --bin generate_bindings`, itself just `commands::builder().export(...)`) and byte-compares the result; verified both ways — a clean run prints "up to date" (exit 0), and a manual one-line edit to the committed file is detected and reported (exit 1) before being regenerated back to the true committed content.
  - `commands::tests::registered_commands_match_the_contract_table` (in `commands/mod.rs`) exports the bindings to a temp file inside the test and asserts every one of the 65 contract command names (as its literal Tauri invoke-channel string) appears in the output — the same list the test's `CONTRACT_COMMAND_NAMES` constant carries, transcribed from `docs/ipc-contract.md` §6 in table order.
  - No command takes a raw filesystem path (checked argument by argument against §6: every argument is an id, a `ConfigRef`/`ProjectContext` tagged value, or a primitive — reviewer should still re-check `open_in_editor`/`reveal_in_file_manager`'s `EditorTarget`/`RevealTarget` payloads, which name ids and a `BlockId`, never a path).
  - **tauri-specta version and maintenance status (recorded per the acceptance criterion):** pinned `tauri-specta = "=2.0.0-rc.25"` (workspace `Cargo.toml`, set in M1-01) is, as of today, both the exact release checked against crates.io's version API (released 2026-05-08) and the newest release on crates.io — no update is available. It remains a release candidate; upstream's own open issue [specta-rs/tauri-specta#247](https://github.com/specta-rs/tauri-specta/issues/247) ("Question: Release candidate status for tauri-specta v2") asks the maintainers directly whether the rc series is production-safe and when 2.0 stable ships, with no committed date as of this check — the `=` exact-version pin already in place is upstream's own recommended mitigation for exactly this risk. No open tauri-specta issue matches a problem this task actually hit.
- **Deviations:**
  - `get_config_item`'s and `delete_config_item`'s contract argument `ref: ConfigRef` is spelled `configRef` on the wire: `ref` is a reserved word and cannot be a Rust parameter name. Recorded inline in `ipc-contract.md`'s status line and in both table rows.
  - `Record<string, McpStatus>` (`get_mcp_statuses`) generates as `Record<string, McpServerStatus>`: Heimdall already had a matching enum under that name (Poirot's `MCPServerStatus`, M1-10) with the exact six contract variants, and reusing it was simpler than adding a same-shape type alias purely to match the sketch's placeholder name. `ipc-contract.md` updated to match.
  - Every `u64`/`i64` field on the wire (token and message counts, `save_png`'s `bytes`) needed `Builder::dangerously_cast_bigints_to_number()`: Specta's default BigInt guard refuses to export any `u64`/`i64` field at all (precision-loss protection), and every such field in this app is a small counter nowhere near 2^53 — documented as a `ponytail:`-style deliberate simplification at the call site.
  - `commands::builder()` (and every command it registers) instantiates `tauri::Wry` concretely, so any `cargo test` binary for the `heimdall-desktop` crate links the full Windows WRY/WebView2 runtime — and on this machine's GNU/MinGW toolchain, that binary (built to `target/debug/deps/`) crashes at process start with `STATUS_ENTRYPOINT_NOT_FOUND`, because `tauri-build`'s `WebView2Loader.dll` copy step only targets `target/debug/` (where ordinary `[[bin]]` outputs land), never `target/debug/deps/` (where test binaries land) — confirmed by copying the DLL into `deps/` by hand, which changed the failure from "DLL not found" to "entrypoint not found" (a second, deeper mismatch), while a plain `[[bin]]` in `target/debug/` runs cleanly with the same DLL. This is a known, previously reported class of issue for the GNU-toolchain path on Windows (not specific to tauri-specta): [tauri-apps/tauri#1960](https://github.com/tauri-apps/tauri/issues/1960), [#13024](https://github.com/tauri-apps/tauri/issues/13024), [#8336](https://github.com/tauri-apps/tauri/issues/8336), [tauri-apps/wry#1268](https://github.com/tauri-apps/wry/issues/1268). Worked around by adding `src-tauri/src/bin/generate_bindings.rs`, a tiny binary with no window or event loop that only calls `commands::builder().export(...)` — it runs from `target/debug/` and both `scripts/check-bindings.mjs` and this task's own verification runs go through it. `commands::tests::registered_commands_match_the_contract_table` is correct and compiles (`cargo check`/`cargo clippy --workspace --all-targets -- -D warnings` both pass), and is expected to run cleanly wherever this DLL-placement quirk does not apply (an MSVC toolchain, or CI); it could not be executed locally on this machine for that reason. `cargo test -p heimdall-core` (the only crate this session's work has been able to execute directly) stays green at 605 passed throughout.
- **Follow-ups:**
  - M1-50+ (cross-OS CI) should confirm `registered_commands_match_the_contract_table` and `node scripts/check-bindings.mjs` both run on a real runner (Windows MSVC and/or Linux/macOS), since neither could be executed end-to-end on this dev machine.
  - Every command's real body (the "Implemented by" column) still returns `not implemented`; M1-39 through M4-04 replace them one task at a time, each regenerating `bindings.ts` as its argument usage changes (dropping the `#![allow(unused_variables)]` module attribute each stub file currently carries).
  - `src/ipc/client.ts` (ADR 0004 §2: "the webview imports IPC only through src/ipc/client.ts") does not exist yet; whichever task first calls a command from the frontend should create it as a thin wrapper over `bindings.ts`.

### M1-38 · App state and the (mtime, size)-validated cache · 2026-09-26

- **Agent:** rust-engineer (executed by the coordinating agent)   **Reviewers:** pending
- **Status:** done
- **Files touched:** `src-tauri/src/state.rs`
- **Evidence:**
  - `AppState { claude_home, cache }` and `Cache::{get, put, invalidate_area, clear}` implemented; `Cache` is path-keyed and type-erased (`Box<dyn Any + Send + Sync>`) so one cache can hold whichever value type each loader reads (a `Project` list, a session's messages, …), each entry stamped with the (mtime, size) it was read at and the `Area` it belongs to (ADR 0006 decision 4).
  - Poirot's own `AppState.swift`'s cache (`sessionCache`, cited at line ~303) is never invalidated at all — its `clearCache()` has no callers outside `AppStateCacheTests` (ADR 0006's Context section calls this out as a known Poirot bug) — so it has no staleness concept to port. `AppStateCacheTests`' three cases (`cacheSession_storesAndRetrieves`, `cachedSession_returnsNilForUnknownId`, `clearCache_removesAllEntries`) are ported as `cache_session_stores_and_retrieves`, `cached_session_returns_none_for_an_unknown_path` and `clear_removes_all_entries`. The staleness cases (`a_changed_size_at_the_same_mtime_is_a_miss`, `a_changed_mtime_at_the_same_size_is_a_miss`, `a_stale_entry_stays_in_the_map_until_overwritten_or_invalidated`) and the area case (`invalidate_area_drops_only_matching_entries`) are new, since Heimdall's design (decisions 2 and 4) has no Swift equivalent.
  - The cache never touches disk itself: `get`/`put` take the caller's own `(mtime, size)` reading, so a command handler reads metadata once (via `blocking`, off the event loop) and passes it straight through — keeping the freshness check itself a pure, disk-free data structure, trivial to test deterministically (fabricated `SystemTime`s, no real file writes or timing).
  - `blocking()` wraps `tauri::async_runtime::spawn_blocking`, per contract §1 ("Commands are async or run disk work through `tauri::async_runtime::spawn_blocking`"); `blocking_runs_the_closure_and_returns_its_value` proves the round trip via `tauri::async_runtime::block_on`. Every future loader (M1-39 onward) is expected to route its disk reads through this helper — reviewable at each of those call sites, not provable from this task alone.
  - `HEIMDALL_CLAUDE_HOME`: reading it lives entirely behind `#[cfg(debug_assertions)]` (a separate, always-`None` function under `#[cfg(not(debug_assertions))]` — the release binary never even contains the code path that reads the variable, closing off an ADR 0010-style risk of a shipped app being redirected by an attacker-controlled environment). `the_override_only_takes_effect_in_a_debug_build` branches on `cfg!(debug_assertions)` (which reflects whichever profile actually compiled the running test binary) so the *same* test proves both arms depending on how it is invoked: `cargo test` (debug-assertions on) proves the override works, `cargo test --release` (debug-assertions off by default) proves it is ignored.
  - `cargo check -p heimdall-desktop` and `cargo clippy --workspace --all-targets -- -D warnings` both exit 0. `cargo test -p heimdall-core` stays green at 605 passed (untouched by this task).
- **Deviations:**
  - `HEIMDALL_CLAUDE_HOME`, when set, is resolved exactly like `CLAUDE_CONFIG_DIR` (an absolute directory containing `.claude.json` alongside its contents) rather than a "user home" to append `.claude` onto — simpler, and consistent with the one override Heimdall already has.
  - As in M1-37, `state.rs`'s own tests could not be executed on this machine: any test binary for the `heimdall-desktop` crate links the full Wry/WebView2 runtime, which crashes at process start here (documented in the M1-37 entry, with upstream citations). Verified instead by `cargo check`/`cargo clippy --all-targets -- -D warnings`, which compile every test function and would catch a type or logic error in them.
  - Module-level `#![allow(dead_code)]`: nothing outside this module's own tests calls `AppState`, `Cache` or `blocking` yet — `AppState` is not yet `.manage()`d by the Tauri builder. Intentional, matching the same placeholder-adjacent pattern M1-37's stub command files already use, and expected to become unnecessary once M1-39's commands construct and manage an `AppState`.
- **Follow-ups:**
  - M1-39 (prefs storage + commands) is the first task that needs to actually construct an `AppState` and `.manage()` it in `lib.rs`'s `run()`.
  - M1-47 (recursive watcher) is `invalidate_area`'s real caller, one call per `DataChange` in the debounced batch.
  - M1-50+ (cross-OS CI) should also run `cargo test -p heimdall-desktop --release` at least once, since that is the only way this session could find to actually exercise the release-mode arm of `the_override_only_takes_effect_in_a_debug_build`.

### M1-39 · Preferences storage and folder-picker commands · 2026-09-26

- **Agent:** rust-engineer (executed by the coordinating agent)   **Reviewers:** pending
- **Status:** done
- **Files touched:** `src-tauri/src/prefs.rs`, `src-tauri/src/commands/prefs.rs`, `src-tauri/Cargo.toml` (a `tauri` dev-dependency entry adding the `test` feature, unioned with the main dependency — enables `tauri::test`'s `MockRuntime` for `cargo test` only)
- **Evidence:**
  - `Prefs` is stored key by key in the store plugin's `prefs.json` (`app.store()`, `BaseDirectory::AppData`): `load` collects every key into one flat JSON object and hands it to `Prefs::from_stored` (already built in M1-11, unknown/bad keys already fall back per key); `save` clears the store and re-`set`s every field from `serde_json::to_value(&prefs)`, so no key from an older schema ever lingers.
  - A migration hook (`migrate(stored) -> stored`, called before `Prefs::from_stored`) exists ahead of `Prefs::from_stored`'s own call; v1 (today's only `schemaVersion`) is the identity, tested by `migrate_v1_is_the_identity`.
  - `update_prefs` (`update()`): merges only the patch's `Some` fields onto the loaded `Prefs` (`update_merges_only_the_patched_fields`), validates `fontScale` in `[0.75, 1.5]` on a `0.05` step (`update_rejects_a_font_scale_outside_the_range`, `update_rejects_a_font_scale_off_the_005_step`, `update_accepts_every_valid_step` walks every valid step), and validates `textEditor`/`preferredTerminal` against `PreferredEditor`/`PreferredTerminal::options(os)` (`update_rejects_xcode_off_macos`, skipped on macOS itself since the restriction does not apply there). `colorTheme`, `sessionLayout` and `globalSearchMode` need no extra runtime check: their wire type is a fixed Rust enum, so deserialisation itself already rejects anything outside the valid set. A rejected patch never reaches `save` (`update_leaves_the_store_untouched_when_validation_fails`).
  - `add_memory_folder`/`remove_memory_folder`/`pick_config_project` open the native folder picker from Rust (`tauri_plugin_dialog`'s `blocking_pick_folder`, called from inside `state::blocking`) and never take a path argument from the webview; `MemorySourcesStoreTests`' four cases port directly: `add_appendsAndPersists` → `add_memory_folder_appends_and_persists`, `add_ignoresDuplicates` → `add_memory_folder_ignores_duplicates` (compared via `paths::same_path`, matching Poirot's `standardizedFileURL` de-dup), `remove_dropsFolder` → `remove_memory_folder_drops_the_folder_at_its_index`, `loadFolders_readsPersistedPaths` → covered by the save/load round trip tests generally, since Heimdall has one `Prefs` object rather than a side channel read directly from `UserDefaults`. `remove_memory_folder` resolves the contract's `folder:<index>` `MemorySourceId` form and rejects anything else or an out-of-range index (`remove_memory_folder_rejects_an_unresolvable_id`) — Poirot's `remove` took a URL directly and had no such id scheme to validate.
  - `the_store_path_never_lands_under_the_claude_home` resolves the real store path via `tauri_plugin_store::resolve_store_path` against a mock app and asserts it never contains a `.claude` segment — the concrete form the "nothing is written under `~/.claude`" criterion takes here, since the store plugin (not this code) owns the actual file write.
  - `cargo check -p heimdall-desktop` and `cargo clippy --workspace --all-targets -- -D warnings` both exit 0 (empirically confirmed along the way: a `tauri::AppHandle` command parameter compiles fine under `#[specta::specta]` and — checked directly in the regenerated `bindings.ts` — never appears in the generated wrapper's argument list, so `get_prefs(app: AppHandle)` still generates as `getPrefs: () => …`). `node scripts/check-bindings.mjs` reports "up to date" (none of these commands' wire-visible signatures changed from M1-37's stubs). `cargo test -p heimdall-core` stays green at 605 passed.
- **Deviations:**
  - Poirot's `MemorySourcesStore` is its own `UserDefaults`-backed class with a single `customMemoryFolders` array; Heimdall folds the same data into the one `Prefs` object the store plugin already persists, so there is no separate store or defaults suite to construct.
  - As in M1-37 and M1-38, this file's own tests (now written against `tauri::test::mock_builder`/`MockRuntime`, the officially recommended way to unit-test Tauri command logic without a real window) could not be executed on this machine — `cargo test -p heimdall-desktop --lib` still crashes at process start with the same `STATUS_ENTRYPOINT_NOT_FOUND` documented in M1-37, confirming (by directly trying it) that using `MockRuntime` in one test does not avoid the crash: every test in the `heimdall-desktop` crate links the same whole compiled library, including `lib.rs::run()`'s concrete use of real `tauri::Wry`. Verified instead by `cargo check`/`cargo clippy --all-targets -- -D warnings`.
- **Follow-ups:**
  - `lib.rs`'s `run()` does not yet construct or `.manage()` an `AppState`; this task's commands only need an `AppHandle` (for the store and dialog plugins), so it was not needed yet. The first task that needs `state::Cache` (a real loader, M1-40+) should add `.manage(AppState::new()?)` to `run()`.
  - M2-05 (settings screen) and M3-16 are ADR 0007's other named consumers of these commands.

### M1-40 · App info, sidebar counts and CLI detection commands · 2026-09-26

- **Agent:** rust-engineer (executed by the coordinating agent)   **Reviewers:** pending
- **Status:** done
- **Files touched:** `src-tauri/src/commands/app.rs`, `crates/heimdall-core/src/app_shell.rs` (`sidebar_counts`, `cli_status`, both real, tested functions — not stubs), `crates/heimdall-core/src/paths.rs` (`user_home()`, a thin wrapper around `dirs::home_dir()` so src-tauri never needs its own `dirs` dependency)
- **Evidence:**
  - The counting/detection formulas (`sidebar_counts`, `cli_status`) live in heimdall-core, not src-tauri: neither needs any Tauri type, both are plain functions over a `ClaudeHome` (plus, for `cli_status`, the same injected-dependency pattern M1-31 already used: `home_dir`, `path_var`, `pathext`, `os`, `exists`). This makes them directly runnable under `cargo test -p heimdall-core` — unlike every other src-tauri-side task this session, these tests actually execute rather than only compile-check.
  - `sidebar_counts_matches_fixtures_synthetic` cross-checks the formula against each loader called directly, then against concrete counts read off `fixtures/synthetic` itself (1 command, 1 skill, 1 output style — `README.md` excluded, 1 plan, 5 sub-agents — 4 built-in plus `agents/reviewer.md`, 2 plugins, 3 MCP servers — `claude.json`'s top-level `github`/`sentry` plus the project-scoped `local-db` under `projects["/Users/fixture/src/app"]`, which `mcp::load` includes even with no project filter, per its own M1-26 behaviour). `AppState.computeSidebarCounts` has no dedicated Swift test suite (checked: no `AppStateTests.swift` exists), so every case here is new.
  - `sidebar_counts_scoped_to_a_project_adds_that_projects_own_items` resolves a `ProjectScope` at `fixtures/synthetic/project-repo` and confirms only `commands`/`outputStyles` grow (that fixture's `.claude/` adds one of each) while every count with no per-project half in Poirot (`todos`, `history`, `plans`, `subAgents`, `models`) is unchanged.
  - `detect_claude_cli`'s two halves (`cli_status`): a PATH-search hit (`cli_status_reports_the_found_path_via_path_search`), a total miss (`cli_status_reports_none_when_the_cli_is_nowhere_found`), and an empty `projects/` folder correctly reporting `sessionsExist: false` (`cli_status_reports_no_sessions_for_an_empty_projects_dir`, using a real `tempfile::tempdir`).
  - `cargo test -p heimdall-core` → 610 passed (5 new: the two `sidebar_counts` cases and three `cli_status` cases). `cargo clippy --workspace --all-targets -- -D warnings` → exit 0. `node scripts/check-bindings.mjs` → up to date (none of these three commands' wire-visible signatures changed from M1-37's stubs).
- **Deviations:**
  - Poirot's `detectCLI` (`OnboardingView.swift:281-297`) checks four hardcoded macOS/Unix paths (`/usr/local/bin/claude`, `/opt/homebrew/bin/claude`, …). This reuses the cross-platform PATH search M1-31 already built (`find_claude_cli`, which already implements ADR 0003's fallback-directory table across all three OSes) instead of literally porting a macOS-only list — the task's own Dependencies section names M1-31, confirming this was the intended replacement, not a re-port.
  - `get_sidebar_counts`'s `ProjectContext::Project` resolution reads that project's real on-disk path via `projects::discovery::load_project` (one project, not a full `discover` scan); `ProjectContext::PickedFolder` reads `Prefs.configProjectPath` through `prefs::load`, which is why `resolve_project_root` — the one AppHandle-dependent piece of this task — stays in `commands/app.rs` rather than moving to heimdall-core with the rest.
  - `total_memory_file_count` (feeding `memory`) counts every project's `memory/` folder with no project scoping, matching Poirot's own `totalMemoryFileCount` exactly; it does not yet include custom memory folders (`Prefs.customMemoryFolders`), which have no fixed root to scan from inside a pure heimdall-core function — `list_memory_sources` (M1-44) is where those are enumerated with prefs already in hand.
  - As in every src-tauri task this session, `commands/app.rs` itself has no executable local test (same `STATUS_ENTRYPOINT_NOT_FOUND` documented in M1-37); verified by `cargo check -p heimdall-desktop`/`cargo clippy --all-targets -- -D warnings`. Moving the real logic into heimdall-core (this task's main design decision) is precisely what let the *substance* of this task be genuinely tested despite that constraint.
- **Follow-ups:** none beyond the memory/custom-folders note above, already tracked against M1-44.

### M1-41 · Session read commands · 2026-09-26

- **Agent:** rust-engineer (executed by the coordinating agent)   **Reviewers:** pending
- **Status:** done
- **Files touched:** `src-tauri/src/commands/sessions.rs`, `src-tauri/src/commands/app.rs` (refactored to read `AppState.claude_home` instead of re-resolving it per call), `src-tauri/src/lib.rs` (`.manage(state::AppState::new()?)` — the first task to actually wire `AppState` into the running app), `crates/heimdall-core/src/transcript/full.rs` (`parse_bounded`, new), `crates/heimdall-core/src/data/{file_history,todos}.rs` (`has_any`, new, cheap existence checks for the session-summary `has_*` flags)
- **Evidence:**
  - Every command's logic lives in a plain `*_inner(state: &AppState, …)` function; the `#[tauri::command]` body is a one-line `blocking(move || …_inner(&app.state::<AppState>(), …)).await`. The intent was to let these tests run without any Tauri app at all (a plain `AppState` built straight from a fixture path needs no mock or real app) — tried directly against the fixture (list/get/page every one of the four projects and the seven `-Users-fixture-Dev-poirot` sessions by their real ids: `conv-001`, `tool-001`, `think-001`, `alltools-001`, `diff-001`, `sysctx-001`, `write-001`) but still could not execute: `cargo test -p heimdall-desktop --lib commands::sessions` crashes at process start with the same `STATUS_ENTRYPOINT_NOT_FOUND` documented in M1-37, confirming (checked directly, again) that this really is whole-crate linkage (`lib.rs::run()`'s concrete `tauri::Wry`) and not something any individual test's own dependencies can avoid. Verified instead by `cargo check -p heimdall-desktop` and `cargo clippy --workspace --all-targets -- -D warnings`, both exit 0, which compile every test function in this file.
  - `transcript::full::parse_bounded`'s own tests, in heimdall-core, have no such problem and do run: `cargo test -p heimdall-core` → 613 passed, up from 610 (the 3 new `parse_bounded` cases below).
  - `notFound`/`invalidInput`: an unknown `projectId` or `sessionId` returns `notFound` (`list_sessions_rejects_an_unknown_project_as_not_found`, `get_session_rejects_an_unknown_session_as_not_found`, `get_session_messages_rejects_an_unknown_session_as_not_found`); a `projectId` shaped to escape its folder (`"../etc"`) returns `invalidInput` before ever touching the filesystem (`list_sessions_rejects_a_path_escaping_project_id_as_invalid_input`).
  - `get_session_messages`'s fast path is exercised by three tests in `commands::sessions::tests` (compile-checked only, per the constraint above — not executed): `get_session_messages_serves_a_first_page_without_a_warm_cache` calls it with nothing cached and expects a correct 2-message page straight away; `get_session_messages_pages_a_cached_full_parse` warms the cache via `get_session` first and expects a later page to match a full-page fetch exactly; `get_session_caches_the_full_parse_for_a_later_call` expects `get_session` itself to leave a usable cache entry behind. The underlying mechanism these rely on — `transcript::full::parse_bounded` — is what actually got executed and proven: its own three tests, in heimdall-core, confirm it stops well short of a 1000-turn transcript when only 4 messages are asked for (`parse_bounded_stops_short_of_a_huge_transcript`, and the prefix it does read matches the full parse exactly), reaches EOF and matches the full parse exactly for a small one, and correctly widens its own read bound (quadrupling, re-reading from the start) when the first bound comes up short.
  - The cache stores `(Session, u32)` (the parsed session plus its `skipped` count) keyed by transcript path, tagged with `Area::Transcript`, exactly matching M1-38's `Cache` contract; `get_session` populates it (always, since it needs the whole file anyway for `ContextUsage`/`toolNames`), and `get_session_messages` both reads it first and populates it itself when its own bounded scan happens to reach EOF.
  - `AppState.claude_home` was, until this task, a field nothing read (`AppState::new()` set it, nothing else touched it) — every command in M1-38 through M1-40 called the free `state::resolve_claude_home()` function fresh instead. Fixed as part of this task (both `app.rs` and `sessions.rs` now read `state.claude_home`) so the home is resolved once at startup and every command shares the same value, rather than each one separately re-reading `HEIMDALL_CLAUDE_HOME`/`CLAUDE_CONFIG_DIR` and re-touching `dirs::home_dir()`. `Cache::invalidate_area`/`clear` keep a scoped `#[allow(dead_code)]` (M1-47, the watcher, is their first real caller); everything else in `state.rs` no longer needs the module-wide allow M1-38 left it with.
- **Deviations:**
  - `get_session`/`get_session_messages`/`list_sessions` validate `sessionId` with `paths::plain_file_name` (no separator, not `.`/`..` — the same check every other session-id-as-filename loader in this port already uses: debug logs, todos, file history), not the launch mechanism's stricter UUID/`agent-` shape (`os_integration::launch::valid_session_id`). Real Claude Code ids always satisfy the stricter form, but this task's own required test fixture (`fixtures/screenshot`, generated from `ScreenshotData.swift`) uses short names like `conv-001` that do not — and the actual security property ADR 0010 cares about here (no path escape) is exactly what `plain_file_name` already guarantees.
  - `get_session_messages`'s partial-read design (documented at its `ponytail:` comment): when the bounded scan does not reach EOF, this falls back to a full parse for an exact `total` rather than exposing a lower-bound as the wire's `total` field — trading away the speed win for a session whose first page alone does not cover the whole file, in exchange for never returning a number that could shrink on a later call. No transcript in either fixture tree is large enough to exercise this fallback branch through the command layer itself, so it is covered indirectly, through `parse_bounded`'s own direct tests in heimdall-core.
  - `has_memory` (used by `list_projects`) and the `has_debug_log`/`has_file_history`/`has_todos` flags (`list_sessions`/`get_session`) are all new, cheap existence checks (a `read_dir` or a stat, never a full parse) — Poirot's Swift equivalents load the full lists and check non-emptiness inline; extracting `file_history::has_any`/`todos::has_any` as named functions (matching `debug_log::has_log`, already there) keeps that pattern consistent instead of duplicating the check inline in the command layer.
- **Follow-ups:**
  - M1-47 (recursive watcher) is `Cache::invalidate_area`'s first real caller.
  - `list_recent_sessions`, `get_session`, `get_session_messages` and `list_sessions` all call `discover`/`load_project` fresh on every request (summary-tier, so cheap, but still a full projects-folder scan); if this ever measures as a bottleneck, the natural next step is caching `Project`/`Vec<Project>` the same way sessions' full parses are cached now.

### M1-42 · Session data commands · 2026-09-26

- **Agent:** rust-engineer (executed by the coordinating agent)   **Reviewers:** pending
- **Status:** done
- **Files touched:** `src-tauri/src/commands/session_data.rs`, `src-tauri/src/commands/sessions.rs` (`project_dir` made `pub(super)` for reuse), `crates/heimdall-core/src/data/todos.rs` (`load_lists_for_session`, `load_all_lists`, new — per-agent-separated todo lists, alongside the existing merged `load_for_session`/`load_all`), `crates/heimdall-core/src/models/file_version.rs` (`file_history`, new — `Vec<FileHistoryEntry>` to the contract's `FileHistory` shape)
- **Evidence:**
  - All 8 commands follow the same `*_inner(state: &AppState, …)` pattern M1-41 established, and hit the same wall confirmed there: `cargo check -p heimdall-desktop`/`cargo clippy --workspace --all-targets -- -D warnings` both exit 0 (compiling every test in this file), but `cargo test -p heimdall-desktop --lib` cannot execute on this machine (`STATUS_ENTRYPOINT_NOT_FOUND`, per M1-37). The two genuinely new pieces of logic this task adds to heimdall-core itself — `todos::load_lists_for_session`/`load_all_lists` and `file_version::file_history` — have no such problem: `cargo test -p heimdall-core` → **619 passed**, up from 613 (6 new: 4 for the todos split, 2 for the `FileHistory` conversion).
  - Before writing `session_data.rs`'s own tests against `fixtures/synthetic`, checked every assumption they rely on by actually running the real loaders against that fixture (a throwaway `src-tauri/src/bin/debug_check.rs`, built and run via `cargo run —` like `generate_bindings`, then deleted — not committed): confirmed the fixture project's real name is `app` (not guessed), and its debug log has exactly 3 entries, one each of `Debug`/`Warn`/`Error`. Both numbers are hardcoded into the tests below rather than left as an unverified guess.
  - `get_session_todos`/`list_all_todos` split each session's todos one `TodoList` per file (`agent_id: None` for the plain file, `Some(id)` for each `-agent-<id>` file) — Poirot's `TodoLoader().loadAllTodos()` merges every file for a session into one flat list and has no project grouping at all; `list_all_todos_groups_the_fixture_session_under_its_real_project` confirms the fixture's session groups under project `-Users-fixture-src-app` (name `app`), cross-referencing every project's own session ids (from `discover`) against the todos directory's session ids — a session with todos but no matching project anywhere falls into a `null`-project group (contract-typed as `projectId: ProjectId | null`), though no fixture data exercises that specific path.
  - `get_file_history`/`get_file_version`/`diff_file_versions` read the real two backups in `fixtures/synthetic/claude-home/file-history/11111111.../3f2504e5f2a14f9f@v1` and `@v2`; `diff_file_versions_diffs_the_two_real_backups` diffs them directly. `diff_lines_truncates_at_5000_lines`/`diff_lines_small_input_is_not_truncated` cover the acceptance criterion's named case (`diff::diff`, M1-35, already enforces the cap — this command is a one-line pass-through, so the test is really confirming the pass-through does not lose the flag).
  - `get_debug_log_page`'s `level` filter parses the contract's lowercase strings (`"debug"`/`"warn"`/`"error"`) into `LogLevel`, rejecting anything else as `invalidInput` (`get_debug_log_page_rejects_an_unknown_level_as_invalid_input`) — the wire type is a plain `string | null`, not a typed enum, specifically because the identifier table gives no fixed id shape for it and the contract sketch (§5) itself spells `level` as `string`, so this validation could not be pushed onto the type system the way `PrefsPatch`'s enums were in M1-39.
  - `get_session_facets`/`get_file_version` both distinguish "no such thing" (`notFound`/`None`) correctly: an unknown session's facets file is `None` (not an error, matching `facets::load`'s existing contract), an unknown file-history version id is `notFound` (a real error, since the caller asked for something specific by id that should exist).
- **Deviations:**
  - `get_file_history` validates `projectId` (via `sessions::project_dir`, now `pub(super)`) before calling `file_history::load`, even though that loader resolves the transcript by searching every project folder for `sessionId` regardless of the `projectId` given — it never actually needed `projectId` to find the right file. Kept the validation anyway (an unknown `projectId` still returns `notFound`) since the contract names both as arguments and a caller passing a bogus project id deserves an error, not a silent success from the "wrong" project.
  - `todos::load_all_lists` calls `json_files` twice (once to collect the set of session ids, once per id inside `load_lists_for_session`) rather than restructuring the single-pass grouping `load_all` already uses — todos directories are small, and the simpler two-pass version reuses `load_lists_for_session` directly instead of duplicating its per-file agent-id logic.
- **Follow-ups:** none.

### M1-43 · History command · 2026-09-26

- **Agent:** rust-engineer (executed by the coordinating agent)   **Reviewers:** pending
- **Status:** done
- **Files touched:** `src-tauri/src/commands/history.rs`
- **Evidence:**
  - `list_history` is a one-line wrapper over `data::history::page` (M1-15), already fully tested against the real HistoryLoaderTests carry-over; this task's own job is just the command-layer plumbing (`AppState.claude_home`, `state::blocking`), following the same `list_history_inner(state: &AppState, …)` pattern as every prior session command.
  - Three tests against `fixtures/synthetic`'s real `history.jsonl`: paging returns the same first entry and the same `total` whether asked for the whole file or a 1-entry page (`list_history_pages_the_real_fixture_file`); an offset past the end of the file is an empty page, not an error (`list_history_offset_past_the_end_is_an_empty_page_not_an_error`); a missing history file (a fresh `tempfile::tempdir`, no `history.jsonl` at all) is an empty page with `total: 0`, `skipped: 0` (`list_history_on_a_missing_file_is_an_empty_page`) — the acceptance criterion's named "empty history file" case.
  - `cargo check -p heimdall-desktop`/`cargo clippy --workspace --all-targets -- -D warnings` exit 0 (this file's tests could not be executed locally, per the `STATUS_ENTRYPOINT_NOT_FOUND` constraint documented since M1-37 — no new logic landed in heimdall-core this task, so there is nothing to verify by direct execution the way M1-41/M1-42 could for their own new heimdall-core functions). `cargo test -p heimdall-core` stays green (619 passed, untouched by this task). `node scripts/check-bindings.mjs` → up to date.
- **Deviations:** none — the smallest task in this session's run so far, exactly matching its Goal.
- **Follow-ups:** `delete_history_entry`/`clear_history_older_than` stay `not implemented` until M3-46.
