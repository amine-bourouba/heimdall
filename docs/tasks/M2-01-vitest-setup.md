# M2-01 · Vitest configuration and fixture-backed DTO snapshots

> Milestone M2 · Workstream: Testing · Scope: MVP

## Goal

Vitest runs with coverage thresholds and an IPC mock seam, and the DTO fixtures it consumes are generated from the real Rust loaders.

## Inputs

- `docs/ipc-contract.md`
- `fixtures/screenshot/`
- `docs/port-plan.md` — §8 Coverage gates

## Outputs

- `vitest.config.ts` — create
- `src/test/setup.ts` — create
- `crates/heimdall-core/examples/export_dto_fixtures.rs` — create
- `fixtures/dto/**` — create: generated

## Acceptance criteria

- [ ] `npx vitest run` passes with a sample spec.
- [ ] Coverage thresholds: 80% lines for `src/stores/**`, `src/composables/**` and `src/features/**/store.ts`.
- [ ] `cargo run -p heimdall-core --example export_dto_fixtures` regenerates `fixtures/dto/` byte-identically (drift check wired in M2-29).
- [ ] `src/test/setup.ts` installs `mockIPC` from `@tauri-apps/api/mocks`; the test environment is happy-dom.

## Dependencies

- M0-12 — Convert ScreenshotData.swift into a deterministic fixture tree
- M1-12 — Transcript parser: full-parse tier, blocks, system content, tool input
- M1-13 — Transcript parser: header and summary tiers
- M1-14 — Project and session discovery with cwd-first project paths
- M1-15 — Prompt history reader
- M1-16 — Todo loader
- M1-17 — Debug log loader
- M1-18 — Session facets loader
- M1-19 — File history loader with cross-OS file matching
- M1-24 — Loaders for commands, skills, plans, output styles and agents
- M1-25 — Settings and hooks readers
- M1-26 — MCP server (five sources) and plugin readers
- M1-27 — Memory and models readers
- M1-37 — Freeze the IPC contract as compiled stubs and generated TypeScript bindings

## Owning agent

test-automator

## Reviewer agent

qa-expert

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
