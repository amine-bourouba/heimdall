# ADR 0009: Safe writes to Claude Code's files

- **Status:** Accepted (key order preserved per user answer 6)
- **Date:** 2026-09-25
- **Severity:** highest in the port

## Context

The Swift reference app can wipe a user's Claude Code configuration. When `~/.claude.json` or a `settings.json` fails to parse, several `SettingsWriter` paths fall back to an empty dictionary and write that near-empty object back: `addMCPServer` (`SettingsWriter.swift:123-128`), `loadSettingsDict` (`:359-364`, used by plugin toggles, permission stripping and default model), `setProjectModel` (`:65-68`), and `loadSettingsDictFor` (`:336-338`, used by hook save, delete and import). A project-scope write with no project path silently targets the global file (`:318-326`). `updateMCPServer` drops `type`, `url` and `headers` (`:150-177`), and hook serialisation drops unknown handler fields (`:296-314`). All writes use sorted keys, which rewrites the whole file, and every failure is swallowed by `try?`. On Windows, replacing a file that Claude Code holds open without `FILE_SHARE_DELETE` can fail with a sharing violation.

## Decision

One primitive in `heimdall-core/src/config/write/` (M1-22) is the only way to change a Claude Code file.

1. **Fail closed.** Read and parse the target first. If parsing fails, return `AppError::Parse` and write nothing. A missing file may be created only by operations that allow it (for example a project's first `.claude/settings.json`).
2. **One-time backup.** Before the first mutating write to a file, copy it to `<file>.heimdall.bak` if no such backup exists. Never overwrite an existing backup.
3. **Generic document, minimal change.** Load as `serde_json::Value` with `preserve_order`, mutate only the keys the operation owns, and write everything else back unchanged, so unknown keys and key order survive. Detect and keep indentation and the trailing newline [A-3].
4. **Atomic replace.** Write a temp file in the same folder, flush it, then rename over the target. Preserve Unix file mode. On Windows, retry sharing violations with backoff for about 2 s, fall back to `ReplaceFileW`, then return `AppError::Busy` with the path. The original stays intact on every failure path.
5. **No silent scope fallback.** A project-scope write without a resolvable project returns `invalidInput`.
6. **Field preservation.** MCP updates keep fields the form does not edit (`type`, `url`, `headers`); hook handlers keep unknown keys.
7. **Code-execution writes need a native confirmation** showing the exact values (hook commands and URLs, MCP command, arguments and environment keys, hook imports). See ADR 0010, rule 6.
8. **The same rules apply to `history.jsonl` rewrites and to deletes**, with a check that the file has not changed since it was read, and containment checks that keep every path inside the Claude home or the resolved project.

## Consequences

- A corrupt file can no longer be replaced by an empty one; the user sees a clear parse error instead.
- Diffs of Claude Code's files after a Heimdall edit are minimal.
- All write features are Later scope (port-plan P-2); the primitive and its tests land in M1 because of the severity.

## Related

Parity matrix section F · ADR 0005, 0010 · Tasks M1-22, M3-40 to M3-46
