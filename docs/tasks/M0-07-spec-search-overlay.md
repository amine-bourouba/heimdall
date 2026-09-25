# M0-07 · Behaviour spec for universal search

> Milestone M0 · Workstream: Legacy parity extraction · Scope: Infra

## Goal

A spec describes every behaviour of Poirot's universal search precisely enough to implement it in Rust (matching, grouping) and Vue (overlay, keyboard) without reading Swift.

## Inputs

- `$POIROT/Poirot/Sources/Views/Search/SearchOverlayView.swift`
- `$POIROT/Poirot/Sources/Utilities/HighlightedText.swift`
- `$POIROT/Poirot/Sources/Models/Provider/NavigationItem.swift`

## Outputs

- `docs/spec/search-overlay.md` — create

## Acceptance criteria

- [ ] Lists the 16 categories in Swift order with their data source, matched fields and caps (file changes: 50 dirs; 5 results per group).
- [ ] Defines fuzzy and exact ('Match Word') matching with worked examples, including highlight ranges.
- [ ] Documents the empty-query state (all nav screens plus the 5 most recent sessions), both quick actions, every SearchAction variant and every key binding.
- [ ] Records that custom memory folders are not searched (assumption: intentional gap).

## Dependencies

- M0-05 — Exhaustive inventory of every ~/.claude path Poirot reads or writes

## Owning agent

legacy-modernizer (consults swift-expert, who only reads the Swift app)

## Reviewer agent

qa-expert

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
