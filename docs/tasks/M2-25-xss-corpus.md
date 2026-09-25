# M2-25 · XSS corpus through renderMarkdown

> Milestone M2 · Workstream: Testing · Scope: MVP

## Goal

A Vitest corpus proves `renderMarkdown()` neutralises every listed XSS vector.

## Inputs

- `docs/adr/0010-webview-trust-boundary.md` — Acceptance tests

## Outputs

- `tests/security/xss-corpus.spec.ts` — create
- `fixtures/security/xss-payloads.json` — create

## Acceptance criteria

- [ ] Payloads include script tags, `onerror`, `<svg onload>`, `javascript:`/`data:`/`file:` links, `<iframe>`, remote images, style attributes, meta refresh, markdown link smuggling, entity-obfuscated schemes, and HTML inside fenced code.
- [ ] Assertions walk the resulting DOM: no script, no event-handler attribute, no disallowed URI, no iframe, no loaded image.

## Dependencies

- M2-01 — Vitest configuration and fixture-backed DTO snapshots
- M2-10 — renderMarkdown, Shiki highlighting and the single v-html component

## Owning agent

test-automator

## Reviewer agent

qa-expert, security-auditor

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
