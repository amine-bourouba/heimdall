# M1-08 · Serde models for messages, sessions and projects

> Milestone M1 · Workstream: Rust data core · Scope: MVP

## Goal

Transcript models mirror Message.swift, Session.swift, Project.swift, ContextUsage.swift and SessionGroup.swift, and every carried-over model test passes in Rust.

## Inputs

- `$POIROT/Poirot/Sources/Models/Message.swift`
- `$POIROT/Poirot/Sources/Models/Session.swift`
- `$POIROT/Poirot/Sources/Models/Project.swift`
- `$POIROT/Poirot/Sources/Models/ContextUsage.swift`
- `$POIROT/Poirot/Sources/Models/SessionGroup.swift`
- `$POIROT/PoirotTests/Models/MessageTests.swift`
- `$POIROT/PoirotTests/Models/SessionTests.swift`
- `$POIROT/PoirotTests/Models/SessionXMLStrippingTests.swift`
- `$POIROT/PoirotTests/Models/TokenUsageTests.swift`
- `$POIROT/PoirotTests/Models/ContextUsageTests.swift`
- `$POIROT/PoirotTests/Models/ProjectTests.swift`
- `$POIROT/PoirotTests/Models/ProjectSortingTests.swift`
- `$POIROT/PoirotTests/Models/SessionGroupTests.swift`
- `docs/spec/test-carryover.md`
- `docs/ipc-contract.md` — §5 Types

## Outputs

- `crates/heimdall-core/src/models/message.rs` — fill the placeholder created by M1-05
- `crates/heimdall-core/src/models/session.rs` — fill the placeholder created by M1-05
- `crates/heimdall-core/src/models/project.rs` — fill the placeholder created by M1-05
- `crates/heimdall-core/src/models/token_usage.rs` — fill the placeholder created by M1-05
- `crates/heimdall-core/src/models/context_usage.rs` — fill the placeholder created by M1-05
- `crates/heimdall-core/src/models/session_group.rs` — fill the placeholder created by M1-05

## Acceptance criteria

- [ ] `cargo test -p heimdall-core models::` passes with one Rust test per Swift test listed for these files in test-carryover.md.
- [ ] Every IPC-facing struct uses `#[serde(rename_all = "camelCase")]` and derives `specta::Type`; `ContentBlock` is tagged by `kind` with variants text, thinking, toolUse, toolResult.
- [ ] The `<synthetic>` model sentinel is one named constant.
- [ ] Disk-format deserialisation ignores unknown fields and treats every optional field as optional.

## Dependencies

- M1-05 — Declare every heimdall-core module and implement AppError
- M0-06 — Map every PoirotTests case to its Heimdall destination

## Owning agent

rust-engineer

## Reviewer agent

architect-reviewer

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
