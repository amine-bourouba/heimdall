# M2-26 · Malicious transcript end-to-end test

> Milestone M2 · Workstream: Testing · Scope: MVP

## Goal

Opening a transcript full of attack payloads causes no dialog, no navigation, no off-origin request and no new window.

## Inputs

- `docs/adr/0010-webview-trust-boundary.md`
- `fixtures/security/xss-payloads.json`

## Outputs

- `fixtures/security/malicious-claude-home/**` — create
- `e2e/security/malicious-transcript.spec.ts` — create

## Acceptance criteria

- [ ] Payloads sit in user text, assistant text, tool output, thinking, a plan and a memory file.
- [ ] The spec fails on any `dialog` event, URL change, request to a non-app origin, or new page; it runs mocked in WebKit and against the real app on Linux and Windows.

## Dependencies

- M2-12 — Per-OS window chrome and navigation guard
- M2-20 — Virtualised conversation view
- M2-23 — Playwright and real-app E2E harness
- M2-25 — XSS corpus through renderMarkdown

## Owning agent

test-automator

## Reviewer agent

qa-expert, security-auditor

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
