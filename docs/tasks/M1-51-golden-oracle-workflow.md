# M1-51 · macOS workflow that runs the golden oracle

> Milestone M1 · Workstream: Release and packaging · Scope: MVP

## Goal

A manually dispatched macOS workflow checks out upstream the Swift reference app at the pinned SHA, runs the golden harness, and uploads the JSON (and optional reference screenshots) as artifacts.

## Inputs

- `tools/golden-oracle/README.md`
- `docs/adr/0001-repo-layout-and-swift-reference.md`
- `docs/adr/0010-webview-trust-boundary.md` — Supply chain

## Outputs

- `.github/workflows/golden-oracle.yml` — create

## Acceptance criteria

- [ ] Checks out https://github.com/a7t-ai/poirot at a29dd6ee0e0133ef26bd067c164c66b160e13d86, installs XcodeGen, copies the harness into PoirotTests, runs `xcodegen generate` and `xcodebuild test -only-testing PoirotTests/GoldenOracleTests CODE_SIGNING_ALLOWED=NO`.
- [ ] An optional second job runs the ScreenshotTests suites in record mode and uploads the PNGs as a reference artifact (never committed).
- [ ] Actions are pinned by SHA, `permissions: contents: read`, and every job sets `timeout-minutes`.

## Dependencies

- M1-50 — Swift golden-output harness for the reference app

## Owning agent

deployment-engineer

## Reviewer agent

architect-reviewer, security-auditor

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
