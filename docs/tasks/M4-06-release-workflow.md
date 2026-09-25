# M4-06 · Multi-platform release workflow

> Milestone M4 · Workstream: Release and packaging · Scope: MVP

## Goal

Pushing a version tag builds, signs updater artifacts, checksums and drafts a GitHub Release for Windows x64, Linux x64 and macOS arm64 with tauri-action.

## Inputs

- `docs/adr/0010-webview-trust-boundary.md` — Updates and signing, Supply chain
- `docs/adr/0011-app-identifier-and-release-repo.md`
- `docs/port-plan.md` — §10 Release

## Outputs

- `.github/workflows/release.yml` — create

## Acceptance criteria

- [ ] Matrix: windows-latest (`x86_64-pc-windows-msvc`), ubuntu-22.04 (pinned, never ubuntu-latest), macos-latest (`aarch64-apple-darwin`).
- [ ] Uses `TAURI_SIGNING_PRIVATE_KEY` and its password; uses no Apple certificate or notarisation secrets and no Windows signCommand (unsigned builds, user decision 7).
- [ ] Uploads `latest.json` and a `SHA256SUMS` file covering every artifact.
- [ ] Actions are pinned by SHA; build jobs have `contents: read`; only the publish job has `contents: write`; every job sets `timeout-minutes`.
- [ ] Rust and npm caches are enabled (`save-if` limited to main).

## Dependencies

- M1-58 — Cross-OS CI for the Rust workspace
- M4-01 — Per-OS bundle configuration
- M4-02 — Generate and store the updater signing key
- M4-03 — Updater plugin wiring
- M3-20 — MVP feature gate review

## Owning agent

deployment-engineer

## Reviewer agent

architect-reviewer, security-auditor

## Size

L

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
