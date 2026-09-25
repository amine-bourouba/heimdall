# Windows fixture set

A scrubbed copy of a real Claude Code home from a Windows 11 machine (task M0-14).

| Field | Value |
|---|---|
| Captured | 2026-09-25 |
| Claude Code | 2.1.282 (`claude --version`) |
| OS | Windows 11 Home, build 10.0.26200.9445 |
| Scrubbed with | `node scripts/scrub-claude-home.mjs --home %USERPROFILE%\.claude --config %USERPROFILE%\.claude.json --out fixtures/windows --user <name> --max-bytes 1000000` |
| Username in paths | `fixture` |
| Project path segments | replaced by stable pseudonyms `proj-1` … `proj-11` (generic names such as `Desktop` and `tech` kept) |
| Transcript line endings | **LF** (24 of 24 `.jsonl` files). Claude Code on Windows writes LF, which answers port-plan assumption A-18. |
| CRLF variant | `crlf/claude-home/projects/c--Users-fixture-Desktop-tech-proj-11/7f33ce70-8d2c-4c0c-adc5-64f1aee23b5e.jsonl` is the same transcript converted to CRLF (30 lines); a conformance test must parse both identically |

## Contents

| Path under `claude-home/` | Files | Notes |
|---|---|---|
| `projects/` | 55 | 4 project folders: 9 top-level transcripts (main and orphaned), 14 sub-agent transcripts with 14 `.meta.json` files, 17 project-memory files (`<folder>/memory/*.md`), and 1 persisted tool output (`<session>/tool-results/<id>.txt`) |
| `file-history/` | 26 | backups referenced by transcript snapshot and delta records |
| `agents/` | 20 | first 20 of 165 agent files (sorted), trimmed for size |
| `skills/` | 14 | `SKILL.md` files only, under the `skills/synced/<id>/` layout; scripts and staging folders removed |
| `history.jsonl`, `settings.json`, `plugins/installed_plugins.json` | 3 | |
| `../claude.json` | 1 | stands in for `~/.claude.json` |

Not present at capture time: `todos/`, `plans/`, `commands/`, `output-styles/`, `usage-data/facets/`, `debug/` logs, `stats-cache.json`, `mcp-needs-auth-cache.json`, and any `sessions-index.json`. The synthetic set (`fixtures/synthetic/`) covers those formats.

Three transcripts were cut to the 1 MB cap at a line boundary (listed by the scrub run); every remaining line parses.

`claude.json` was re-scrubbed on 2026-09-26 after the scrubber learned to replace identifying object keys: account and organisation UUID keys are now stable fake UUIDs, and `githubRepoPaths` keys are `fixture/repo-<n>`. The `skills/synced/<id>` folder, named after the same two ids, was renamed to the matching fakes. The same day, hyphen-encoded project folders inside paths (for example `...\.claude\projects\c--Users-fixture-Desktop-tech-<name>\memory\...` in file-history records) were given the pseudonyms their plain path segments already had. Nothing else changed.

## Format facts this capture revealed

These affect the Rust loaders (task M1-14 and ADR 0003):

1. **Drive-letter case varies.** The same project appears with `cwd` `C:\Users\fixture\Desktop\tech\proj-11` in some records and `c:\Users\...` in others, and project folders exist in both `C--Users-...` and `c--Users-...` casings. Path comparison on Windows must be case-insensitive; folder names stay distinct ids.
2. **`cwd` changes within a session.** Records carry the shell's current folder, which can be a subfolder of the project (`...\proj-11\proj-4\proj-6`). The project path is the `cwd` whose encoding matches the folder name, not simply the first `cwd` seen.
3. **Orphaned transcripts.** Claude Code leaves files such as `<uuid>.orphaned-<ms>-<hex>.jsonl`. Their stem is not a UUID, so Poirot's fallback scan ignores them.
4. **Synced skills.** Skills can live under `skills/synced/<id>/<name>/SKILL.md`, which Poirot does not read (it looks only at `skills/<name>/SKILL.md`).
5. **No `sessions-index.json`** exists in this Claude Code version's project folders, so the fallback scan is the normal path on Windows.
6. **Persisted tool output.** Large tool results are stored beside the session as `<sessionId>/tool-results/<id>.txt`. Poirot does not read them.
