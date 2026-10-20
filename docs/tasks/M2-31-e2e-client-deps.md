# M2-31 · Add the WebDriver client for Linux real-app smoke tests

> Milestone M2 · Workstream: Release and packaging · Scope: MVP

## Goal

WebdriverIO, the WebDriver client that drives tauri-driver on Linux (decision P-8), is added without failing the supply-chain gate.

## Inputs

- `docs/port-plan.md` — decision P-8
- `docs/adr/0010-webview-trust-boundary.md` — Supply chain

## Outputs

- `package.json` — modify
- `package-lock.json` — modify

## Acceptance criteria

- [ ] `npm audit --include=dev --audit-level=high` exits 0 after the addition (in September 2026 WebdriverIO 9.32 pulled a vulnerable extract-zip; use a patched release or an npm override and record which).
- [ ] `npx wdio --version` runs.

## Dependencies

- M1-02 — Create package.json with the full frontend toolchain

## Owning agent

dependency-manager

## Reviewer agent

architect-reviewer, security-auditor

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
