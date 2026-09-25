# M3-54 · Verify editor URL schemes per OS

> Milestone M3 · Workstream: Testing · Scope: Later

## Goal

Recorded evidence shows which editor URL schemes (vscode, cursor, zed) open a file at a line on each OS, including paths with spaces and non-ASCII characters.

## Inputs

- `docs/adr/0003-platform-abstraction.md` — Launchers

## Outputs

- `docs/spikes/editor-url-schemes.md` — create

## Acceptance criteria

- [ ] A table of editor × OS × result, with the fallback argv where no scheme works.

## Dependencies

- None

## Owning agent

test-automator

## Reviewer agent

qa-expert

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
