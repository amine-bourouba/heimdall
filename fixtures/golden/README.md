# fixtures/golden/

The Swift reference app's own computed outputs for every fixture tree (task M1-52), produced by a
golden-output harness run for real against that app and committed here as a frozen regression
fixture. The harness and its CI workflow are retired (M4-15, M4-16, ADR 0001 point 4) — this
directory is their only remaining trace. Rust tests (M1-53) compare their own output against these
files byte-for-byte, with every tolerated difference listed in `KNOWN_DIFFERENCES.md`.

## Provenance

| Field | Value |
|---|---|
| Upstream Swift reference app commit | `a29dd6ee0e0133ef26bd067c164c66b160e13d86` (pinned, ADR 0001) |
| Workflow run | [golden-oracle.yml #36325558912](https://github.com/amine-bourouba/heimdall/actions/runs/36325558912) |
| Heimdall commit checked out for that run | `36323cbb47143a3e23a47fd6793dbfdc6a9fd863` |
| Run date | 2026-09-27 (started 14:20:31 UTC, finished 14:24:20 UTC) |
| Runner | GitHub-hosted `macos-latest`, Xcode `Xcode_26.0.app` |
| Outcome | Success — all 13 tests passed against all four fixture sets; 145 files uploaded and committed here unchanged |

This was not the first attempt: five earlier runs on the same day failed and were fixed in place
(missing `@testable import` of the reference app's own module and the wrong test framework; a struct access-level mismatch; an
`iso(_:)` overload ambiguity and a Foundation API change; `xcodebuild` not passing environment
variables through to the test process; an artifact-upload-unsafe filename). Each fix is recorded in
`docs/progress-log.md`'s "Golden oracle real-run closure" entry, cross-referencing the exact run
that caught it. This directory holds only the sixth run's real, passing output.

## Contents — one folder per fixture set

| Folder | Files | Source fixture | Notes |
|---|---|---|---|
| `synthetic/` | 14 | `fixtures/synthetic/claude-home/` | Hand-built deterministic tree |
| `screenshot/` | 72 | `fixtures/screenshot/claude-home/` | Converted from `ScreenshotData.swift`; the largest set, one session per screenshot scenario |
| `windows/` | 48 | `fixtures/windows/claude-home/` | Real, scrubbed Windows capture (M0-14) |
| `linux/` | 11 | `fixtures/linux/claude-home/` | Real, scrubbed Linux capture (M0-15) |

Each folder's own internal layout (`projects.json`, `sessions/<projectId>/...`, `stats.json`,
`history.json`, `todos.json`, `facets.json`, `debug/`, `file-history/`, `diffs/`) was documented in
the now-retired harness's own README ("Output layout" section, recoverable from git history) — this
directory is that harness's output, committed verbatim, not restructured.

## Known non-reproducible fields

Two fields in this committed output are **not** meaningful to compare byte-for-byte against a
fresh run of the (now-retired) harness, let alone against Rust's own output, and M1-53's comparison
must exclude or normalize both (the retired harness's own README documented this in depth; recoverable
from git history):

- `<set>/stats.json`'s `computed.lastComputedDate` — `StatsComputer.compute` calls `Date()`
  directly; this is the wall-clock time the harness happened to run, not fixture-derived data.
- `<set>/sessions/.../export.md`'s `**Date:**` header line — locale/timezone-dependent formatting.
  The retired workflow pinned `TZ=UTC`, `LANG`/`LC_ALL=en_US.UTF-8` for the run that produced this
  commit, but whether `LANG`/`LC_ALL` actually pins `Locale.current` in an unconfigured CI process
  was never independently confirmed.

## Updating this directory

The harness and workflow that produced this directory are retired (M4-15, M4-16) — this tree is now
a frozen regression fixture. It is not expected to be refreshed again; if MVP parity work ever needs
a fresh run, recover the harness and workflow from git history first.
