# M2-29 · Frontend, security and E2E jobs in CI

> Milestone M2 · Workstream: Release and packaging · Scope: MVP

## Goal

CI also runs lint, type-check, Vitest with coverage thresholds, npm audit, the security lint, drift checks, bundle size and the Playwright and real-app smoke suites.

## Inputs

- `.github/workflows/ci.yml`
- `scripts/lint-security-config.mjs`
- `scripts/check-bundle-size.mjs`
- `e2e/README.md`

## Outputs

- `.github/workflows/ci.yml` — modify

## Acceptance criteria

- [ ] New jobs: `npm ci`, lint, typecheck, `vitest --coverage`, `npm audit --audit-level=high`, security lint, bindings drift, DTO fixture drift, bundle size.
- [ ] Playwright mocked suites on ubuntu-22.04 (xvfb) with committed baselines; real-app smoke on ubuntu-22.04 (tauri-driver) and windows-latest (CDP).
- [ ] Every added action is SHA-pinned; setup-node uses `cache: npm`; every job sets `timeout-minutes`.

## Dependencies

- M1-58 — Cross-OS CI for the Rust workspace
- M2-01 — Vitest configuration and fixture-backed DTO snapshots
- M2-23 — Playwright and real-app E2E harness
- M2-27 — Security configuration lint
- M2-28 — Performance budget checks

## Owning agent

deployment-engineer

## Reviewer agent

architect-reviewer, security-auditor, qa-expert

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
