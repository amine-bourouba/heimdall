# M1-22 · Safe JSON write primitive for Claude Code's files

> Milestone M1 · Workstream: Rust data core · Scope: Later

## Goal

One write primitive refuses to write when the target fails to parse, keeps a one-time `.bak`, preserves unknown keys, key order, formatting and Unix mode, and replaces the file atomically with Windows sharing-violation retries.

## Inputs

- `$POIROT/Poirot/Sources/Services/Config/SettingsWriter.swift (all; sites that fall back to an empty document: :65-68, :123-128, :336-338, :359-364, reached from :17, :27, :37, :51 and the hook functions; :155-158 and :182-185 return early instead)`
- `docs/adr/0009-safe-writes-to-claude-code-files.md`
- `docs/adr/0005-error-model.md`
- `fixtures/windows/claude.json`
- `fixtures/synthetic/corrupt/`

## Outputs

- `crates/heimdall-core/src/config/write/json_doc.rs` — fill the placeholder created by M1-05
- `crates/heimdall-core/src/config/write/atomic.rs` — fill the placeholder created by M1-05

## Acceptance criteria

- [ ] A corrupt target returns `AppError::Parse`, leaves the file byte-identical, and creates no `.bak`.
- [ ] Round-trip of fixtures/windows/claude.json after mutating one key changes only that key's bytes (unknown keys and order preserved; `serde_json` `preserve_order`).
- [ ] `.bak` is created on the first mutating write and never overwritten afterwards.
- [ ] Indentation (2, 4 spaces or tab) and trailing newline are detected and kept.
- [ ] Unix file mode is preserved (cfg(unix) test with 0600).
- [ ] On Windows, a handle held without FILE_SHARE_DELETE causes bounded retries with backoff (about 2 s in total, ReplaceFileW fallback) and then `AppError::Busy` with the path; the original stays intact (cfg(windows) test).
- [ ] The temp file is created in the target's directory and removed on failure.

## Dependencies

- M0-11 — Extract PoirotTests inline fixtures into a shared synthetic ~/.claude tree
- M0-14 — Capture a scrubbed real Windows ~/.claude tree
- M1-05 — Declare every heimdall-core module and implement AppError
- M1-06 — Claude home resolution, path normalisation and comparison keys

## Owning agent

rust-engineer

## Reviewer agent

security-auditor, architect-reviewer

## Size

L

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
