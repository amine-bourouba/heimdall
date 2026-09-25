# M1-29 · Fuzzy and exact matcher with UTF-16 highlight ranges

> Milestone M1 · Workstream: Rust data core · Scope: MVP

## Goal

A matcher implements fuzzy and exact ('Match Word') matching with the Swift semantics and returns highlight ranges in UTF-16 code units for the webview.

## Inputs

- `$POIROT/Poirot/Sources/Utilities/HighlightedText.swift`
- `$POIROT/Poirot/Sources/Views/Search/SearchOverlayView.swift`
- `$POIROT/PoirotTests/Utilities/HighlightedTextTests.swift (matching assertions only)`
- `docs/spec/search-overlay.md`

## Outputs

- `crates/heimdall-core/src/search/matcher.rs` — fill the placeholder created by M1-05
- `crates/heimdall-core/src/search/mod.rs` — fill the placeholder created by M1-05

## Acceptance criteria

- [ ] Matching assertions from HighlightedTextTests pass.
- [ ] Ranges are UTF-16 offsets; a test with emoji and CJK text proves it.

## Dependencies

- M1-05 — Declare every heimdall-core module and implement AppError
- M0-07 — Behaviour spec for universal search

## Owning agent

rust-engineer

## Reviewer agent

architect-reviewer

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
