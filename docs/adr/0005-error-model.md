# ADR 0005: Error model

- **Status:** Accepted
- **Date:** 2026-09-25

## Context

Poirot swallows almost every failure: `try?` on reads and writes throughout `SettingsWriter.swift`, silent deletes (`AppState.swift:392, 446`), and `print` on load failure (`ContentView.swift:341`). Unparsable transcript lines are skipped with no count. A user cannot tell a missing file from a corrupt one, and a failed write looks like success.

## Decision

1. **One error type.** `AppError` in `heimdall-core/src/error.rs` is a tagged union: `notFound`, `io`, `parse`, `permission`, `busy` (Windows sharing violation after retries), `invalidInput`. Each carries a `message` and an optional `path`. It derives `thiserror::Error`, `Serialize` and `specta::Type`. Every command returns `Result<T, AppError>`.
2. **Skip and count.** Loaders skip unparsable lines and files instead of failing, and return `Loaded<T> { value, skipped }`. The UI may show "N lines could not be read" when `skipped > 0`.
3. **Writes never fail silently.** Every mutating command returns `WriteOutcome` (`written`, `cancelled`, `unchanged`) or an `AppError` (ADR 0009).
4. **Where errors surface.** Read failures show inline in the affected view; write failures and watcher degradation show as error toasts; `invalidInput` indicates a programming bug and is also reported in development builds' console.
5. **No logging framework in the MVP** (port-plan P-17). `heimdall-core` denies `print_stdout` and `print_stderr` lints. When logging is added later, it records keys, paths and counts only, never values from Claude Code files (ADR 0010, Secrets).
6. **No panics across IPC.** Commands convert every failure to `AppError`.

## Consequences

- Every silent `try?`/`print` site in Poirot has a named replacement behaviour.
- The UI needs empty, error and partial states for every data view; feature tasks include them in their specs.

## Related

`docs/ipc-contract.md` §3 · ADR 0009, 0010 · Tasks M1-05, M1-07, M1-22, M2-03
