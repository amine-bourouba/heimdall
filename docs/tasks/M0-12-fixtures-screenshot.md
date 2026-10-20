# M0-12 · Convert ScreenshotData.swift into a deterministic fixture tree

> Milestone M0 · Workstream: Testing · Scope: Infra

## Goal

The deterministic data behind the Swift reference app's screenshot tests exists as a `~/.claude` tree that the real Rust loaders can read, so Playwright and Vitest render the same data.

## Inputs

- `$SWIFT_REF/PoirotTests/Mocks/ScreenshotData.swift`
- `$SWIFT_REF/PoirotTests/Screenshots/Helpers/ScreenshotTestHelpers.swift`
- `docs/spec/claude-home-inventory.md`

## Outputs

- `fixtures/screenshot/README.md` — create
- `fixtures/screenshot/claude-home/**` — create
- `fixtures/screenshot/claude.json` — create

## Acceptance criteria

- [ ] README maps each ScreenshotData factory to the files it became.
- [ ] All timestamps are fixed ISO-8601 values; nothing depends on the current date.
- [ ] Every `.json` parses and every `.jsonl` line parses.

## Dependencies

- M0-03 — Initialise the Heimdall repository and its ignore/attribute rules
- M0-05 — Exhaustive inventory of every ~/.claude path the Swift reference app reads or writes

## Owning agent

test-automator

## Reviewer agent

qa-expert

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
