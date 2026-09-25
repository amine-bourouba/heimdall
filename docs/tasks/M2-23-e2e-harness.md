# M2-23 · Playwright and real-app E2E harness

> Milestone M2 · Workstream: Testing · Scope: MVP

## Goal

Playwright runs visual and UI tests against the built frontend with fixture-backed IPC mocks on all OSes, and real-app smoke tests run on Windows (Playwright over CDP to WebView2) and Linux (tauri-driver with WebdriverIO).

## Inputs

- `docs/port-plan.md` — decision P-8
- `fixtures/dto/`
- `fixtures/screenshot/`

## Outputs

- `playwright.config.ts` — create
- `e2e/support/mockIpc.ts` — create
- `e2e/support/fixtures.ts` — create
- `e2e/support/tauriCdp.ts` — create
- `e2e/smoke/wdio.conf.mjs` — create
- `e2e/smoke/app.smoke.mjs` — create
- `e2e/README.md` — create

## Acceptance criteria

- [ ] A mocked smoke spec passes in Playwright WebKit and Chromium.
- [ ] The Windows real-app smoke launches a debug build with `HEIMDALL_CLAUDE_HOME=fixtures/screenshot/claude-home` and asserts the sidebar renders.
- [ ] The Linux real-app smoke does the same through tauri-driver under `xvfb-run`.
- [ ] e2e/README.md records whether macOS WebDriver support exists for the pinned Tauri version (assumption A-10) and the fallback used.

## Dependencies

- M1-38 — App state and the (mtime, size)-validated cache
- M2-01 — Vitest configuration and fixture-backed DTO snapshots
- M2-09 — App shell: split view, sidebar with live counts, toolbar, overlays
- M2-31 — Add the WebDriver client for Linux real-app smoke tests

## Owning agent

test-automator

## Reviewer agent

qa-expert

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
