# M0-13 · Write the ~/.claude scrubbing tool

> Milestone M0 · Workstream: Testing · Scope: Infra

## Goal

A dependency-free Node script copies a real `~/.claude` tree and `~/.claude.json` into a fixture folder with usernames, paths, transcript text and secrets replaced, keeping structure, line endings and JSON shapes.

## Inputs

- `docs/spec/claude-home-inventory.md`
- `docs/adr/0010-webview-trust-boundary.md` — section Secrets

## Outputs

- `scripts/scrub-claude-home.mjs` — create
- `scripts/scrub-claude-home.test.mjs` — create

## Acceptance criteria

- [ ] `node --test scripts/scrub-claude-home.test.mjs` passes.
- [ ] Tests prove: CRLF line ends survive byte for byte; every string under a key matching /token|key|secret|password|authorization|env|headers/i is replaced; the home prefix is replaced in `C:\Users\<name>`, `/home/<name>` and `/Users/<name>` forms and inside hyphen-encoded project folder names; message text becomes same-length filler and JSON stays valid.
- [ ] Files named `*credentials*` are never copied.
- [ ] A test greps the output tree for the input username and finds nothing.

## Dependencies

- M0-03 — Initialise the Heimdall repository and its ignore/attribute rules
- M0-05 — Exhaustive inventory of every ~/.claude path the Swift reference app reads or writes

## Owning agent

test-automator

## Reviewer agent

qa-expert, security-auditor

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
