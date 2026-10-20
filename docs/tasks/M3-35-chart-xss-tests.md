# M3-35 · XSS tests for chart formatters

> Milestone M3 · Workstream: Testing · Scope: Later

## Goal

Every chart formatter and tooltip neutralises the XSS corpus when payloads arrive as model names, project names or dates.

## Inputs

- `fixtures/security/xss-payloads.json`
- `src/features/analytics/charts/formatters.ts`

## Outputs

- `tests/security/chart-formatters.spec.ts` — create

## Acceptance criteria

- [ ] No payload survives as markup in any formatter output.

## Dependencies

- M2-25 — XSS corpus through renderMarkdown
- M3-33 — The six analytics charts

## Owning agent

test-automator

## Reviewer agent

qa-expert, security-auditor

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
