# M1-54 · Cross-OS fixture conformance tests

> Milestone M1 · Workstream: Testing · Scope: MVP

## Goal

Every loader reads the synthetic, screenshot, Windows and Linux fixture trees with no unexpected skipped lines and no decode heuristic used where a `cwd` exists.

## Inputs

- `fixtures/`
- `docs/adr/0003-platform-abstraction.md`

## Outputs

- `crates/heimdall-core/tests/fixtures_conformance.rs` — create

## Acceptance criteria

- [ ] CRLF and LF variants of the same transcript parse to identical results.
- [ ] Windows projects report `pathSource = transcriptCwd`.
- [ ] The test runs on all three CI OSes.

## Dependencies

- M0-11 — Extract PoirotTests inline fixtures into a shared synthetic ~/.claude tree
- M0-12 — Convert ScreenshotData.swift into a deterministic fixture tree
- M0-14 — Capture a scrubbed real Windows ~/.claude tree
- M0-15 — Capture a scrubbed real Linux ~/.claude tree
- M1-14 — Project and session discovery with cwd-first project paths
- M1-15 — Prompt history reader
- M1-16 — Todo loader
- M1-17 — Debug log loader
- M1-18 — Session facets loader
- M1-19 — File history loader with cross-OS file matching
- M1-20 — Stats cache reader, staleness rules and the full StatsComputer port
- M1-24 — Loaders for commands, skills, plans, output styles and agents
- M1-25 — Settings and hooks readers
- M1-26 — MCP server (five sources) and plugin readers
- M1-27 — Memory and models readers

## Owning agent

test-automator

## Reviewer agent

qa-expert

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
