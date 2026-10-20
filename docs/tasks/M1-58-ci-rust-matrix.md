# M1-58 · Cross-OS CI for the Rust workspace

> Milestone M1 · Workstream: Release and packaging · Scope: MVP

## Goal

CI runs formatting, lints, tests, supply-chain checks, the bindings-drift check and the coverage gate on Windows, Ubuntu 22.04 and macOS for every push and pull request.

## Inputs

- `docs/adr/0010-webview-trust-boundary.md` — Supply chain
- `docs/port-plan.md` — §8 Testing, §10 CI
- `scripts/check-bindings.mjs`
- `scripts/coverage-core.sh`

## Outputs

- `.github/workflows/ci.yml` — create
- `.github/dependabot.yml` — create

## Acceptance criteria

- [ ] Matrix: windows-latest, ubuntu-22.04, macos-latest running `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`.
- [ ] Ubuntu installs `libwebkit2gtk-4.1-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev` and runs app tests under `xvfb-run`.
- [ ] Ubuntu job also runs `cargo deny check`, `node scripts/check-bindings.mjs`, `scripts/coverage-core.sh`, and fails if `cargo tree -p heimdall-core` mentions tauri.
- [ ] Swatinem/rust-cache with `save-if` on main only; actions pinned by SHA; `permissions: contents: read`; `timeout-minutes` on every job.
- [ ] Dependabot covers cargo, npm and github-actions weekly.

## Dependencies

- M1-01 — Create the Cargo workspace, crate manifests and supply-chain policy
- M1-02 — Create package.json with the full frontend toolchain
- M1-37 — Freeze the IPC contract as compiled stubs and generated TypeScript bindings
- M1-56 — Core coverage measurement and 80% gate

## Owning agent

deployment-engineer

## Reviewer agent

architect-reviewer, security-auditor, qa-expert

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
