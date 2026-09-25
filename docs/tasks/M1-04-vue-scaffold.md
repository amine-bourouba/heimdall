# M1-04 · Scaffold the Vite + Vue 3 + TypeScript + Tailwind v4 frontend

> Milestone M1 · Workstream: Vue shell and design system · Scope: Infra

## Goal

A minimal frontend builds to `dist/`, type-checks and lints, with the lint rules that enforce the webview trust boundary already in place.

## Inputs

- `docs/adr/0002-layer-boundaries.md`
- `docs/adr/0010-webview-trust-boundary.md` — Rendering rule 1
- `docs/port-plan.md` — §5.8 dev identifiers, ports and worktrees

## Outputs

- `index.html` — create
- `src/main.ts` — create
- `src/App.vue` — create
- `src/env.d.ts` — create
- `vite.config.ts` — create
- `tsconfig.json` — create
- `tsconfig.node.json` — create
- `eslint.config.js` — create

## Acceptance criteria

- [ ] `npm run build` produces `dist/index.html`; `npm run typecheck` (vue-tsc) and `npm run lint` pass.
- [ ] ESLint sets `vue/no-v-html: error` with a single override allowing it in `src/lib/markdown/MarkdownView.vue`.
- [ ] ESLint `no-restricted-imports` forbids `@tauri-apps/api/core` outside `src/ipc/`.
- [ ] The Vite dev server port reads `HEIMDALL_DEV_PORT` (default 1420) with `strictPort: true`.
- [ ] The Tailwind v4 Vite plugin is wired.

## Dependencies

- M1-02 — Create package.json with the full frontend toolchain

## Owning agent

vue-expert

## Reviewer agent

architect-reviewer

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
