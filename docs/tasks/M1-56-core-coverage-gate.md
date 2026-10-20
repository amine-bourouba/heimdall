# M1-56 · Core coverage measurement and 80% gate

> Milestone M1 · Workstream: Testing · Scope: MVP

## Goal

A script measures heimdall-core line coverage and fails below 80%.

## Inputs

- `docs/port-plan.md` — §8 Coverage gates

## Outputs

- `scripts/coverage-core.sh` — create

## Acceptance criteria

- [ ] `scripts/coverage-core.sh` runs `cargo llvm-cov -p heimdall-core --fail-under-lines 80`; the progress-log entry records the first measured figure.

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
- M1-21 — Tray snapshot aggregation
- M1-22 — Safe JSON write primitive for Claude Code's files
- M1-23 — Frontmatter parser
- M1-24 — Loaders for commands, skills, plans, output styles and agents
- M1-25 — Settings and hooks readers
- M1-26 — MCP server (five sources) and plugin readers
- M1-27 — Memory and models readers
- M1-28 — MCP process status on sysinfo
- M1-29 — Fuzzy and exact matcher with UTF-16 highlight ranges
- M1-30 — Search providers for all 16 categories
- M1-31 — Cross-platform PATH search for claude and editor CLIs
- M1-33 — External URL allowlist
- M1-34 — Changed-path to data-area classifier
- M1-35 — Line diff port
- M1-36 — Session Markdown and patch export

## Owning agent

test-automator

## Reviewer agent

qa-expert

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
