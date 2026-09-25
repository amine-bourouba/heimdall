# M0-01 · Write Heimdall's AI_INSTRUCTIONS.md and CLAUDE.md

> Milestone M0 · Workstream: Docs · Scope: Infra

## Goal

Heimdall's `AI_INSTRUCTIONS.md` states the port's stack, layer, icon, token, IPC, security, test and worktree rules so an agent can work without reading Poirot's Swift-era rules.

## Inputs

- `$POIROT/AI_INSTRUCTIONS.md` — keep its New Feature Checklist and Git & PRs sections, replace everything Swift-specific
- `$POIROT/CLAUDE.md` — pointer pattern to copy
- `docs/port-plan.md`
- `docs/adr/0002-layer-boundaries.md`
- `docs/adr/0003-platform-abstraction.md`
- `docs/adr/0004-ipc-contract-and-type-generation.md`
- `docs/adr/0007-preferences-storage-and-migration.md`
- `docs/adr/0010-webview-trust-boundary.md`
- `docs/adr/0011-app-identifier-and-release-repo.md`
- `docs/ipc-contract.md`

## Outputs

- `AI_INSTRUCTIONS.md` — create
- `CLAUDE.md` — create

## Acceptance criteria

- [ ] `CLAUDE.md` is at most 5 lines and only points to `AI_INSTRUCTIONS.md`.
- [ ] `AI_INSTRUCTIONS.md` has these sections: Overview, Stack, Layer rules (heimdall-core has no tauri dependency; Vue renders only; IPC types come only from `src/ipc/bindings.ts`), Icons (Lucide only through `src/components/AppIcon.vue`, every effect gated by reduced motion), Design tokens (CSS custom properties only, source `src/styles/tokens.css`), Security (summary of ADR 0010; `renderMarkdown()` is the only path into `v-html`), Commands (cargo test/clippy, npm run lint/typecheck/test, npx playwright test), Parallel development (per-worktree identifier `io.github.amine-bourouba.heimdall.dev<N>` and Vite port `1420 + N`), Swift reference ($POIROT is read-only, pinned at a29dd6e), New Feature Checklist, Git & PRs.
- [ ] The New Feature Checklist is adapted, not dropped: universal search is Ctrl/Cmd+K, live updates mean registering a watcher area (ADR 0006), screenshots mean Playwright baselines.
- [ ] The Git & PRs section is copied verbatim from $POIROT/AI_INSTRUCTIONS.md.
- [ ] `grep -n -E "SF Symbol|symbolEffect|mockups.html|SwiftLint|XcodeGen|Swift Testing" AI_INSTRUCTIONS.md` matches nothing outside the Swift-reference section.

## Dependencies

- None

## Owning agent

documentation-engineer

## Reviewer agent

architect-reviewer

## Size

S

## Context

This is the plan's first task. Take the app identifier and Linux name from ADR 0011 (settled on 2026-09-25).

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
