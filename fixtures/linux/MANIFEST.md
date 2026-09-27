# Linux fixture set

A scrubbed copy of a real Claude Code home from a Linux machine (task M0-15).

| Field | Value |
|---|---|
| Captured | 2026-09-27 |
| Claude Code | 2.1.283 (`claude --version`) |
| OS | Ubuntu 24.04.4 LTS (Noble Numbat), kernel via **WSL2** on Windows 11 — no native Linux machine was available, per the task's own "WSL2 is acceptable" note |
| Node.js | 22.23.3 (NodeSource build; Ubuntu 24.04's default `apt` Node is 18.x, below this Claude Code release's `engines.node >=22.0.0`) |
| Scrubbed with | `node scripts/scrub-claude-home.mjs --home /home/oooops/.claude --config /home/oooops/.claude.json --out fixtures/linux --user oooops --max-bytes 5000000` |
| Username in paths | `fixture` (real Linux username was a WSL-local account, never a real name) |
| Project path segments | one project only, captured directly at the home directory (no subfolder) |

## Contents

| Path | Files | Notes |
|---|---|---|
| `claude-home/projects/-home-fixture/` | 1 | one real session transcript, 64 lines |
| `claude-home/history.jsonl` | 1 | 3 real prompt entries |
| `claude-home/settings.json` | 1 | non-structural values (e.g. `theme`) are filler per the scrub tool's own rules |
| `../claude.json` | 1 | stands in for `~/.claude.json` |

Not present at capture time: `todos/`, `debug/`, `file-history/`, `usage-data/`, `stats-cache.json`, `commands/`, `skills/`, `plans/`, `output-styles/`, `agents/`, `plugins/installed_plugins.json`, `mcp-needs-auth-cache.json`, `mcp-needs-auth-cache.json`, or any `sessions-index.json` — this was a fresh WSL install with one short real session, not an accumulated real-world home like the Windows capture (`fixtures/windows/`). The synthetic set (`fixtures/synthetic/`) and the Windows capture cover those formats; this fixture's job is specifically Linux path/encoding conventions, not format breadth.

No file exceeds 5 MB (largest is the transcript, 234 KB); a manual `grep` for the real WSL username and the machine owner's real name/email across the entire committed tree found nothing.

## Format facts this capture revealed

1. **Linux project-folder encoding for a home-root project.** `cwd` `/home/fixture` (no subfolder) encodes to exactly `-home-fixture` — confirming Claude Code's "replace every non-alphanumeric character with `-`" rule holds even when the project *is* the home directory, the simplest case of the same rule `fixtures/windows/` already confirmed for nested paths.
2. **Transcript line endings are LF**, same as the Windows capture (`fixtures/windows/MANIFEST.md`) — Claude Code writes LF on both platforms, so no Linux-specific CRLF variant is needed here (unlike Windows, which needed one to answer assumption A-18; Linux never produces CRLF in the first place, so there is nothing to make a variant of).
3. **Newer per-session record types than the Windows capture's Claude Code 2.1.282**: this transcript's first lines are `{"type":"mode",...}` and `{"type":"permission-mode",...}` records preceding the first `file-history-snapshot` — present under 2.1.283 but not documented from the earlier capture. Worth checking against `crates/heimdall-core/src/transcript/*` if the full-parse tier (M1-12) doesn't already tolerate unrecognised leading record types (it does, per M1-12's own fallback-to-skip handling of unknown line shapes, so no action needed — recorded here only as a cross-check).

## Follow-up on capture depth

This is a much smaller real-world sample than `fixtures/windows/` (1 session vs. dozens accumulated over real historical use) because WSL had no prior Claude Code history at all before this task — the environment was created fresh specifically for this capture. If a future task wants a richer Linux corpus (more sessions, multiple project folders, agents/skills/plugins present), re-run the same scrub command after more real usage accumulates in this WSL instance; nothing about the capture command needs to change.
