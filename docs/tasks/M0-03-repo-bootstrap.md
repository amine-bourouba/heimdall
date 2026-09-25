# M0-03 · Initialise the Heimdall repository and its ignore/attribute rules

> Milestone M0 · Workstream: Release and packaging · Scope: Infra

## Goal

Heimdall is its own git repository whose attribute rules keep fixture bytes, including CRLF line endings, exactly as captured.

## Inputs

- `docs/adr/0001-repo-layout-and-swift-reference.md`
- fixtures layout in docs/port-plan.md

## Outputs

- `.gitignore` — create
- `.gitattributes` — create
- `.editorconfig` — create

## Acceptance criteria

- [ ] `git rev-parse --show-toplevel` run inside the project prints the Heimdall root (`C:/Users/pc/Desktop/tech/Heimdall`).
- [ ] `.gitattributes` contains `* text=auto eol=lf` and `fixtures/** -text`; `git check-attr text -- fixtures/windows/x.jsonl` prints `unset`.
- [ ] `.gitignore` covers `target/`, `node_modules/`, `dist/`, `src-tauri/gen/`, `coverage/`, `test-results/`, `playwright-report/`.

## Dependencies

- None

## Owning agent

build-engineer

## Reviewer agent

architect-reviewer

## Size

S

## Context

The user moved the folder out of the unrelated hydra repository on 2026-09-25 (decision D4); it is not inside any git repository yet, so this task runs `git init` there.

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
