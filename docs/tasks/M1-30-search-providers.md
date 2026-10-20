# M1-30 · Search providers for all 16 categories

> Milestone M1 · Workstream: Rust data core · Scope: MVP

## Goal

Search returns grouped results for all 16 categories in Swift order, at most 5 per group, from the core loaders.

## Inputs

- `$SWIFT_REF/Poirot/Sources/Views/Search/SearchOverlayView.swift:5-64, 1038-1100`
- `docs/spec/search-overlay.md`

## Outputs

- `crates/heimdall-core/src/search/providers.rs` — fill the placeholder created by M1-05

## Acceptance criteria

- [ ] A test per category on fixtures/synthetic returns the expected top result.
- [ ] Caps: 5 per group; file-history scan capped at 50 session folders.
- [ ] Memory search covers project memory only and models search covers the curated catalog only (parity; assumptions A-6).

## Dependencies

- M1-14 — Project and session discovery with cwd-first project paths
- M1-15 — Prompt history reader
- M1-16 — Todo loader
- M1-17 — Debug log loader
- M1-18 — Session facets loader
- M1-19 — File history loader with cross-OS file matching
- M1-24 — Loaders for commands, skills, plans, output styles and agents
- M1-25 — Settings and hooks readers
- M1-26 — MCP server (five sources) and plugin readers
- M1-27 — Memory and models readers
- M1-29 — Fuzzy and exact matcher with UTF-16 highlight ranges

## Owning agent

rust-engineer

## Reviewer agent

architect-reviewer

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
