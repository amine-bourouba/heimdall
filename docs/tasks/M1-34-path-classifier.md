# M1-34 · Changed-path to data-area classifier

> Milestone M1 · Workstream: Rust data core · Scope: MVP

## Goal

`classify(path)` maps any changed path under the watched roots to a data area and optional key, as ADR 0006 defines.

## Inputs

- `docs/adr/0006-change-propagation-and-caching.md`
- `docs/ipc-contract.md` — §7 Events (Area)

## Outputs

- `crates/heimdall-core/src/classify.rs` — fill the placeholder created by M1-05

## Acceptance criteria

- [ ] A table test covers every Area in the contract with Windows and Unix path forms, plus unrelated paths returning `None`.

## Dependencies

- M1-06 — Claude home resolution, path normalisation and comparison keys

## Owning agent

rust-engineer

## Reviewer agent

architect-reviewer

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
