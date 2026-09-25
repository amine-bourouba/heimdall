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
