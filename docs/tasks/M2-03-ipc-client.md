# M2-03 · Typed IPC client wrapper and test helpers

> Milestone M2 · Workstream: Vue shell and design system · Scope: MVP

## Goal

All webview IPC goes through one wrapper that unwraps `Result`s into a typed `IpcError` and exposes typed `data-changed` and `watcher-status` subscriptions.

## Inputs

- `docs/ipc-contract.md`
- `src/ipc/bindings.ts`
- `docs/adr/0005-error-model.md`

## Outputs

- `src/ipc/client.ts` — create
- `src/ipc/testing.ts` — create
- `src/ipc/client.spec.ts` — create

## Acceptance criteria

- [ ] Spec covers success, each AppError kind, and `onDataChanged(areas, handler)` filtering.
- [ ] `testing.ts` mocks commands with data from `fixtures/dto/`.

## Dependencies

- M1-37 — Freeze the IPC contract as compiled stubs and generated TypeScript bindings
- M2-01 — Vitest configuration and fixture-backed DTO snapshots

## Owning agent

typescript-pro

## Reviewer agent

architect-reviewer

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
