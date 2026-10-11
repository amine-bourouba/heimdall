# Performance baseline (task M1-57)

Real measurements from `cargo run --release -p heimdall-core --example bench_load -- <claude_home>`
against the budgets in `docs/port-plan.md` §9. Reference hardware for every number below is the
maintainer's Windows 11 machine (the only machine available in this environment) — there is no
Linux or macOS machine here, so those columns record "fixture data, run on this same Windows
hardware," never a real measurement on that OS's own hardware. Per-OS numbers await M1-58's CI
matrix, which can run this same example on `windows-latest`, `ubuntu-22.04` and `macos-latest` for
real.

## Method

- `cargo run --release -p heimdall-core --example bench_load -- <claude_home>` for each of
  `fixtures/windows/claude-home` and `fixtures/linux/claude-home`.
- The same binary always also generates a fresh 30-projects x 500-sessions tree (15,000 small,
  real-shaped session files) and a single ~50 MB transcript in a temp directory, and benchmarks
  both — these, not the small real fixtures, are what actually exercise the budgets at their
  stated scale.
- "Cold" and "warm" are both same-process, back-to-back calls to `discovery::discover` on the same
  directory — see the caveat below on what this does and doesn't measure.
- Each command run once per `<claude_home>` argument; the generated-tree and 50 MB numbers were
  run twice across those two invocations to sanity-check variance (all within ~10% of each other).

## Important caveat: "warm" here is not the app's real cache

heimdall-core never depends on tauri (ADR 0002) and has no cache of its own. The actual
mtime/size-validated `Cache` that backs a real "cached reload" in the shipped app lives in
`src-tauri/src/state.rs` (task M1-38) and is not exercised by this benchmark at all. "Warm" below
means only "the same process called `discover()` again immediately," which can benefit from the
OS's own filesystem cache but re-does every bit of parsing and allocation from scratch — this is
why cold and warm are so close to each other in every row: the bottleneck this benchmark is
actually measuring is CPU-bound parsing work, not disk I/O, and re-reading the same bytes from the
OS page cache doesn't skip re-parsing them. A real "cached reload" benchmark for the shipped app's
own cache belongs in `src-tauri`, once that layer exists to benchmark, not here.

## Results

| Budget | Target | Real fixture (Windows) | Real fixture (Linux data, Windows HW) | Generated 30x500 |
|---|---|---|---|---|
| List projects and sessions, cold | ≤ 800 ms | 84–131 ms (4 projects, 21 sessions) | 3.2 ms (1 project, 1 session) | **3.42–3.45 s — FAIL** |
| Same, warm (same-process repeat) | ≤ 100 ms | 76–99 ms | 2.2 ms | **3.26–3.44 s — FAIL** |
| Full parse of the largest available transcript | (informative; the 50 MB row below is the real budget check) | 7.3 ms (994,732 bytes, 34 messages) | 1.9 ms (234,201 bytes, 16 messages) | — |
| Full parse of a 50 MB transcript | ≤ 2 s | — | — | **1.03 s (52,428,960 bytes, 196,940 messages) — PASS** |

The two real fixtures are far too small (21 and 1 sessions respectively) to say anything about the
30x500-scale budget on their own — that's exactly why the acceptance criterion asks for a
generated tree specifically. Their numbers above are recorded for completeness and because they
did pass trivially, not as evidence for the budget itself.

## What the FAIL means, and what it doesn't

The 30x500 project-listing budget fails by roughly **4.3x cold and 34x warm** against the current
`heimdall-core` implementation. This is real, reproducible (two separate runs, ~3.4–3.5 s each
time) — not a benchmark artifact. A plausible cause, **not independently profiled or confirmed** —
`projects::discovery::discover_with` processes project directories through a single sequential
`.map()` iterator (`crates/heimdall-core/src/projects/discovery.rs`), with no parallelism; 15,000
individual small-file opens and JSON-line parses done one at a time on one thread is consistent
with several seconds of wall-clock time on this hardware. Confirming this — and deciding whether
parallelizing project/session loading is worth the complexity it would add — is explicitly left to
a follow-up task, not attempted here: this task's own job is to measure and record against the
budget, not to optimize the loader.

The 50 MB full-parse budget **passes with real margin** (1.03 s against a 2 s budget, for a
196,940-message transcript) — `transcript::full::parse`'s own performance is not a concern at this
scale on this hardware.

## Follow-ups

- A task should profile `projects::discovery::discover_with` at the 30x500 scale (or larger) and
  decide on a remediation — likely parallel project loading (each project directory is already
  independent work) — before this budget can honestly be called met.
- M1-58 (cross-OS CI) should re-run `bench_load` on `windows-latest`, `ubuntu-22.04` and
  `macos-latest` and fill in real per-OS numbers here, replacing the "fixture data, same Windows
  hardware" column for Linux with an actual Linux measurement.
- Once `src-tauri`'s own `Cache` (M1-38) has a benchmarkable surface, a *real* cold-vs-cached-reload
  benchmark belongs there, not in this heimdall-core-only example.

## Frontend performance budgets (task M2-28)

Real measurements from this same Windows 11 machine (the only one available here, same caveat as
above). `npx playwright test --project=perf` for the scroll row; `npm run build && node
scripts/check-bundle-size.mjs` for the bundle rows. No macOS or Linux measurement exists yet —
CI is the next chance to produce one, the same position every other cross-OS number in this file
is already in.

### Method

- **Transcript scrolling**: `scripts/gen-large-transcript.mjs` generates 8,334 synthetic messages
  (10,000 content blocks: plain text plus a `Bash` tool_use/tool_result pair on every 5th
  assistant turn), loaded into `SessionDetail.vue`'s real virtualizer (`@tanstack/vue-virtual`,
  `estimateSize: 160`) through the same mocked-IPC harness every other `e2e/` spec uses — not a
  committed fixture (too large and synthetic to hand-build; regenerated fresh per test run).
  `e2e/perf/transcript-scroll.spec.ts` drives a 4-second continuous `scrollTop` increment (60px
  per `requestAnimationFrame` tick) and records the wall-clock delta between consecutive frames.
- **Bundle size**: `npm run build` (with `vite.config.ts`'s now-enabled `build.manifest: true`),
  then `node scripts/check-bundle-size.mjs` reads `dist/.vite/manifest.json` to separate the
  initial-route chunk from each lazy route's own chunk (Shiki's own ~150 per-language dynamic
  imports, some individually over 150 KB gzip on their own, are not routes and are correctly
  excluded — confirmed directly against a real build before trusting the script's own number).

### Results

| Budget | Target | Measured (Windows, Chromium) |
|---|---|---|
| Transcript scrolling at 10,000+ blocks, median frame | ≤ 16.7 ms | **16.70 ms — PASS** (241 samples) |
| Same, p95 | ≤ 33 ms | **16.80 ms — PASS** |
| Initial route bundle (`index-*.js`) | ≤ 250 KB gzip | **54.00 KB — PASS** |
| Largest lazy route (`SessionsPage.vue`) | ≤ 150 KB gzip | **120.11 KB — PASS** |
| Every other lazy route (stub pages: Todos/History/Plans/Memory/Analytics/Settings/Help/About/Hooks/Mcp/Plugins/Models) | ≤ 150 KB gzip | **0.23–0.24 KB each — PASS** (not yet built out; see below) |
| ECharts confined to the analytics chunk | — | **PASS** — nothing in `src/` imports `echarts`/`vue-echarts` yet at all |

### What this does and doesn't prove

The median and p95 frame times came back nearly identical (16.70 ms vs. 16.80 ms) and sit right at
the 60 Hz vsync interval (1000/60 ≈ 16.667 ms) — this is what a scroll that's **not** bottlenecked
at all looks like: every sampled frame landed on the display's own refresh cadence, with no
variance worth reporting. That's a real, reproducible result (re-run twice, identical both times),
not a fluke — but it is also the *easy* case: the generated transcript's own synthetic content
(short filler text, a two-line Bash result) is far lighter than a real transcript full of long
tool outputs, large diffs and expanded code blocks might be. This budget is genuinely met for the
fixture tested, not proven immune to a heavier real transcript.

Every lazy-route chunk except `SessionsPage.vue` measures under 250 **bytes** gzip, not kilobytes —
these are still M2-06-era placeholder screens (`<p>X is not built yet.</p>`, confirmed directly in
`fixtures/README.md`'s and this session's own prior task entries), so this budget is trivially met
today and will need re-measuring once each of those M3 screens has real content. `SessionsPage.vue`
is the one real, fully-built screen among them, and it already uses 80% of its own 150 KB budget —
worth watching as session-related features keep growing, not a current violation.

### Follow-ups

- Re-run both checks on `windows-latest`, `ubuntu-22.04` and `macos-latest` once CI exists for the
  frontend (M2-29) — no Linux or macOS number exists yet.
- Re-run the bundle-size check after each M3 screen (Todos, History, Settings, etc.) gets its real
  implementation — today's near-zero numbers for those routes are a placeholder artifact, not a
  real measurement of their eventual weight.
- `SessionsPage.vue`'s own 120 KB (80% of its 150 KB budget) is worth a second look if any future
  task adds a substantial new dependency to the sessions feature specifically.
- The scroll budget was measured against synthetic, lightweight message content; a heavier
  transcript (long tool outputs, large diffs, many expanded blocks) is a plausible future case this
  measurement does not cover.
