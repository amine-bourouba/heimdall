# M0-14 · Capture a scrubbed real Windows ~/.claude tree

> Milestone M0 · Workstream: Testing · Scope: Infra

## Goal

A scrubbed copy of a real Windows 11 Claude Code `%USERPROFILE%\.claude` tree and `.claude.json` is committed as a fixture set.

## Inputs

- `scripts/scrub-claude-home.mjs`
- `fixtures/README.md`

## Outputs

- `fixtures/windows/claude-home/**` — create
- `fixtures/windows/claude.json` — create
- `fixtures/windows/MANIFEST.md` — create
- `fixtures/windows/crlf/**` — create only if the captured transcripts are LF

## Acceptance criteria

- [ ] MANIFEST.md records the `claude --version` output, capture date, Windows build, file counts, and whether transcripts are CRLF or LF.
- [ ] If the captured transcripts are LF, `fixtures/windows/crlf/` holds a CRLF-converted copy of one transcript so CRLF handling is still tested.
- [ ] At least one transcript records a `cwd` with a drive letter and backslashes.
- [ ] No committed file is larger than 5 MB, and a manual grep for the real username over `fixtures/windows` finds nothing.

## Dependencies

- M0-11 — Extract PoirotTests inline fixtures into a shared synthetic ~/.claude tree
- M0-13 — Write the ~/.claude scrubbing tool

## Owning agent

test-automator

## Reviewer agent

qa-expert

## Size

S

## Context

Manual capture on the user's Windows 11 machine, which already runs Claude Code.

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
