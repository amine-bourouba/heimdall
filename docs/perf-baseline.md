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
