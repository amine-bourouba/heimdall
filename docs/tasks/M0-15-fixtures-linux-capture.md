# M0-15 · Capture a scrubbed real Linux ~/.claude tree

> Milestone M0 · Workstream: Testing · Scope: Infra

## Goal

A scrubbed copy of a real Linux Claude Code `~/.claude` tree and `.claude.json` is committed as a fixture set.

## Inputs

- `scripts/scrub-claude-home.mjs`
- `fixtures/README.md`

## Outputs

- `fixtures/linux/claude-home/**` — create
- `fixtures/linux/claude.json` — create
- `fixtures/linux/MANIFEST.md` — create

## Acceptance criteria

- [ ] MANIFEST.md records the Claude Code version, distro and version, capture date, file counts, and whether the capture came from native Linux or WSL2.
- [ ] Project folder names follow Linux encoding (for example `-home-fixture-src-app`).
- [ ] No committed file is larger than 5 MB, and a manual grep for the real username finds nothing.

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

WSL2 is acceptable if no native Linux machine is available; record which one was used.

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
