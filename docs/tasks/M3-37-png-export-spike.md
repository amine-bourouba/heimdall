# M3-37 · Spike: PNG export approach (open risk R-7)

> Milestone M3 · Workstream: Feature screens · Scope: Later

## Goal

A written comparison, backed by prototypes on WebView2, WebKitGTK and WKWebView, recommends how to produce the analytics PNG and the diff copy-as-image, closing open risk R-7 once architect-reviewer accepts it.

## Inputs

- `$POIROT/Poirot/Sources/Views/Analytics/Export/AnalyticsImageExporter.swift`
- `$POIROT/Poirot/Sources/Views/Analytics/Export/AnalyticsSnapshotView.swift`
- `$POIROT/Poirot/Sources/Views/Session/EditDiffView.swift:220-250`
- `docs/adr/0010-webview-trust-boundary.md` — CSP

## Outputs

- `docs/spikes/png-export.md` — create

## Acceptance criteria

- [ ] Compares in-webview DOM-to-image rendering against a Rust-side render on fidelity, CSP compatibility (no `unsafe-inline`), speed and dependency cost on all three engines.
- [ ] Names the chosen library or approach and, if it is the Rust-side option, rewrites the Outputs of M3-39 before that task starts.

## Dependencies

- M3-33 — The six analytics charts

## Owning agent

frontend-developer (with typescript-pro for typed IPC)

## Reviewer agent

architect-reviewer, security-auditor

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
