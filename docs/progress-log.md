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
