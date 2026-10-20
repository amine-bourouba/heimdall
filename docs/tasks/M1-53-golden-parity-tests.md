# M1-53 · Rust versus Swift golden parity tests

> Milestone M1 · Workstream: Testing · Scope: MVP

## Goal

Rust outputs equal the Swift golden outputs on every fixture set, and any accepted difference is listed and justified.

## Inputs

- `fixtures/golden/`
- `docs/ipc-contract.md`

## Outputs

- `crates/heimdall-core/tests/golden_parity.rs` — create
- `fixtures/golden/KNOWN_DIFFERENCES.md` — create

## Acceptance criteria

- [ ] Compares session counts, token totals, costUSD, daily activity, context usage, summaries, export Markdown and line diffs.
- [ ] KNOWN_DIFFERENCES.md lists each tolerated difference with its reason (for example project paths from `cwd` instead of decoding) and reviewer sign-off.

## Dependencies

- M1-12 — Transcript parser: full-parse tier, blocks, system content, tool input
- M1-13 — Transcript parser: header and summary tiers
- M1-14 — Project and session discovery with cwd-first project paths
- M1-15 — Prompt history reader
- M1-16 — Todo loader
- M1-17 — Debug log loader
- M1-18 — Session facets loader
- M1-19 — File history loader with cross-OS file matching
- M1-20 — Stats cache reader, staleness rules and the full StatsComputer port
- M1-35 — Line diff port
- M1-36 — Session Markdown and patch export
- M1-52 — Commit the golden outputs

## Owning agent

test-automator

## Reviewer agent

qa-expert

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
