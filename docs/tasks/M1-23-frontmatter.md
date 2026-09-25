# M1-23 · Frontmatter parser

> Milestone M1 · Workstream: Rust data core · Scope: MVP

## Goal

A frontmatter parser splits Markdown config files into key/value metadata and body exactly as FrontmatterParser does.

## Inputs

- `$POIROT/Poirot/Sources/Services/Config/FrontmatterParser.swift`
- `$POIROT/PoirotTests/Services/FrontmatterParserTests.swift`

## Outputs

- `crates/heimdall-core/src/config/read/frontmatter.rs` — fill the placeholder created by M1-05

## Acceptance criteria

- [ ] Carried-over FrontmatterParserTests pass, including CRLF input.

## Dependencies

- M1-05 — Declare every heimdall-core module and implement AppError

## Owning agent

rust-engineer

## Reviewer agent

architect-reviewer

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
