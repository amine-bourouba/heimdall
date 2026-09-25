# corrupt/ -- intentionally unparsable fixtures

Every file here is deliberately broken, to exercise the "loader tolerates a
bad file" paths (ADR 0005 / ADR 0009 style skip-and-report behaviour). None
of these are covered by the "every .json / .jsonl parses" acceptance check.

| File | Purpose |
|---|---|
| `settings.json` | Truncated mid-object (cut inside `permissions.allow`); stands in for `~/.claude/settings.json` and `<project>/.claude/settings.json`. Exercises the settings reader's parse-failure fallback (R28/R29, W1-W7). |
| `claude.json` | Truncated mid-array (cut inside an `args` list); stands in for `~/.claude.json`. Exercises the MCP config reader's parse-failure fallback (R23, W8-W10). |
| `transcript.jsonl` | Three lines: a valid `user` record, a malformed middle line (missing comma, unterminated object), and a truncated last line (unterminated string). Exercises the JSONL reader's per-line skip behaviour (M1-07) on a transcript file. |
| `stats-cache.json` | Parses as JSON but has the wrong shape (`version` is a string, `dailyActivity` is a string instead of an array). Exercises `stats_cache.rs`'s decode-failure fallback (R18). |
