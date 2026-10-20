# M4-02 · Generate and store the updater signing key

> Milestone M4 · Workstream: Release and packaging · Scope: MVP

## Goal

A Tauri updater key pair exists; the private key and password are GitHub Actions secrets with an offline backup, and rotation is documented.

## Inputs

- `docs/adr/0011-app-identifier-and-release-repo.md`
- `docs/adr/0010-webview-trust-boundary.md` — Updates and signing

## Outputs

- `docs/release/updater-key.md` — create

## Acceptance criteria

- [ ] Secrets `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` exist in the repository settings (screenshot or `gh secret list` output in the progress log, values never shown).
- [ ] updater-key.md records the public key, two offline backup locations, the rotation procedure (ship the new public key in a release signed by the old key) and the compromise response.

## Dependencies

- None

## Owning agent

deployment-engineer

## Reviewer agent

architect-reviewer, security-auditor

## Size

S

## Context

Manual task done with the user; the key is generated on a trusted machine, never in CI.

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
