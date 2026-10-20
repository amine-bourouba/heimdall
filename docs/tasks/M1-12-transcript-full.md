# M1-12 · Transcript parser: full-parse tier, blocks, system content, tool input

> Milestone M1 · Workstream: Rust data core · Scope: MVP

## Goal

The full-parse tier turns a transcript into messages and content blocks exactly as TranscriptParser.parse does, skipping unknown record types without error.

## Inputs

- `$SWIFT_REF/Poirot/Sources/Services/TranscriptParser.swift:1-139, 355-554`
- `$SWIFT_REF/Poirot/Sources/Utilities/SystemContentParser.swift`
- `$SWIFT_REF/Poirot/Sources/Utilities/JSONBeautifier.swift`
- `$SWIFT_REF/Poirot/Sources/Models/Message.swift`
- `$SWIFT_REF/PoirotTests/Services/TranscriptParserTests.swift`
- `$SWIFT_REF/PoirotTests/Utilities/SystemContentParserTests.swift`
- `$SWIFT_REF/PoirotTests/Utilities/JSONBeautifierTests.swift`
- `docs/spec/session-views.md`
- `fixtures/synthetic/`
- `fixtures/windows/`
- `docs/adr/0005-error-model.md`

## Outputs

- `crates/heimdall-core/src/transcript/full.rs` — fill the placeholder created by M1-05
- `crates/heimdall-core/src/transcript/blocks.rs` — fill the placeholder created by M1-05
- `crates/heimdall-core/src/transcript/searchable.rs` — fill the placeholder created by M1-05
- `crates/heimdall-core/src/transcript/system_content.rs` — fill the placeholder created by M1-05
- `crates/heimdall-core/src/transcript/tool_input.rs` — fill the placeholder created by M1-05

## Acceptance criteria

- [ ] Carried-over full-parse, SystemContentParser and JSONBeautifier tests pass.
- [ ] Records of type system, summary, file-history-snapshot and file-history-delta are skipped or routed without failing the parse; `<synthetic>` assistant messages are filtered (TranscriptParser.swift:38).
- [ ] The sidechain filter matches TranscriptParser.swift:34.
- [ ] Block ids are `<messageId>:<index>` and stay stable across re-parses of an unchanged file.
- [ ] Parsing every transcript in fixtures/windows reports zero skipped lines.

## Dependencies

- M0-08 — Behaviour spec for the session browser and conversation view
- M0-11 — Extract PoirotTests inline fixtures into a shared synthetic ~/.claude tree
- M0-14 — Capture a scrubbed real Windows ~/.claude tree
- M1-07 — Streaming JSONL reader with CRLF and truncation tolerance
- M1-08 — Serde models for messages, sessions and projects

## Owning agent

rust-engineer

## Reviewer agent

architect-reviewer

## Size

L

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
