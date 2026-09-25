# ADR 0004: IPC contract and type generation

- **Status:** Accepted (generator chosen by user answer 5)
- **Date:** 2026-09-25

## Context

The command surface is wide (sessions, session data, history, config reads and writes, MCP, search, export, launch, analytics, tray, updates). Rust and Vue will be built in parallel by different agents, so their shared types must not drift.

## Decision

1. **Transport.** Commands for requests and mutations; four events (`data-changed`, `watcher-status`, `menu-action`, `navigate`). `ipc::Channel` only if a measured load misses a port-plan §9 budget.
2. **Rust types are the single source of truth.** Types derive `specta::Type`, use `#[serde(rename_all = "camelCase")]`, and tag unions by `kind`.
3. **tauri-specta generates both the TypeScript types and the typed command wrappers** into `src/ipc/bindings.ts`. The file is committed. CI regenerates it and fails on any difference (`scripts/check-bindings.mjs`, M1-37, M1-58).
4. **Contract first.** `docs/ipc-contract.md` defines every command and event now. Task M1-37 turns it into compiled stubs plus generated bindings before Rust implementation and Vue feature work proceed in parallel. After that, the generated file is authoritative and the document changes in the same pull request as any shape change.
5. **No IPC versioning:** UI and backend ship in one binary. The preferences file carries `schemaVersion` instead (ADR 0007).
6. **Tolerant parsing:** Claude Code's formats are parsed permissively everywhere (unknown fields ignored, optional fields optional, unknown record types skipped).
7. **Maintenance check.** tauri-specta 2 has shipped as release candidates for a long time. M1-37 pins an exact version and records its release date and open Tauri 2 compatibility issues in the progress log. If it stalls, the fallback is ts-rs for types plus hand-written wrappers, which needs a superseding ADR (risk R-10).

## Consequences

- Vue never hand-writes a command name or payload type.
- A shape change is visible in review as a diff to `src/ipc/bindings.ts`, and requires architect-reviewer and security-auditor (ADR 0010).
- `heimdall-core` depends on `specta` (not on tauri) so its model types can be exported.

## Related

`docs/ipc-contract.md` · ADR 0002, 0005, 0006, 0010 · Tasks M1-37, M1-58, M2-03
