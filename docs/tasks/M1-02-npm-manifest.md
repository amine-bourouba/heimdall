# M1-02 · Create package.json with the full frontend toolchain

> Milestone M1 · Workstream: Release and packaging · Scope: Infra

## Goal

`package.json` pins every frontend dependency needed through M3, and `npm ci` reproduces it on all three OSes.

## Inputs

- `docs/port-plan.md` — §4 locked stack, decision P-8
- `docs/adr/0010-webview-trust-boundary.md` — Supply chain

## Outputs

- `package.json` — create
- `package-lock.json` — create

## Acceptance criteria

- [ ] Runtime dependencies: vue 3, vue-router 4, pinia, @tauri-apps/api 2, @tauri-apps/plugin-clipboard-manager, @tauri-apps/plugin-autostart, markdown-it, dompurify, shiki, @shikijs/transformers, echarts, vue-echarts, @lucide/vue (the maintained successor of lucide-vue-next), @tanstack/vue-virtual.
- [ ] Dev dependencies: @tauri-apps/cli 2, vite, @vitejs/plugin-vue, typescript, vue-tsc, tailwindcss 4 with @tailwindcss/vite, vitest, @vitest/coverage-v8, @vue/test-utils, happy-dom (jsdom 30 needs Node 22.22+), eslint, eslint-plugin-vue, typescript-eslint, @playwright/test, @types/markdown-it, @types/node. WebdriverIO is deferred to M2-31 because its current release pulls a high-severity advisory.
- [ ] Scripts exist: dev, build, tauri, lint, typecheck, test, test:e2e, bindings:check.
- [ ] `engines.node` pins the current Node LTS major; `version` uses the plan's semver calendar format (P-10).
- [ ] `npm ci` exits 0 on Windows, Linux and macOS, and `npm audit --audit-level=high` exits 0.

## Dependencies

- M0-03 — Initialise the Heimdall repository and its ignore/attribute rules

## Owning agent

dependency-manager

## Reviewer agent

architect-reviewer, security-auditor

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
