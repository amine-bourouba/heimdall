# M0-11 · Extract PoirotTests inline fixtures into a shared synthetic ~/.claude tree

> Milestone M0 · Workstream: Testing · Scope: Infra

## Goal

The inline fixtures in PoirotTests exist as standalone files, one folder per test case plus one integrated macOS-shaped `~/.claude` tree covering every format, that Rust tests and Vitest both read.

## Inputs

- `docs/spec/test-carryover.md`
- `$SWIFT_REF/PoirotTests/Services/*.swift`
- `$SWIFT_REF/PoirotTests/Models/*.swift`
- `$SWIFT_REF/PoirotTests/App/*.swift`
- `docs/spec/claude-home-inventory.md`

## Outputs

- `fixtures/README.md` — create
- `fixtures/synthetic/README.md` — create
- `fixtures/synthetic/claude-home/**` — create
- `fixtures/synthetic/claude.json` — create; stands in for ~/.claude.json
- `fixtures/synthetic/project-repo/**` — create; holds .mcp.json and .claude/
- `fixtures/synthetic/corrupt/**` — create; intentionally unparsable files
- `fixtures/synthetic/cases/**` — create; one folder per Swift test case that reads files

## Acceptance criteria

- [ ] fixtures/synthetic/README.md maps every carried-over loader, parser and decoder test (docs/spec/test-carryover.md) to fixture paths that exist; tests that only build values in code are listed as `inline`.
- [ ] `claude-home/` contains at least one valid instance of every format in docs/spec/claude-home-inventory.md §1, including `sessions-index.json`, sub-agents with `.meta.json`, todos, debug logs, file-history backups with matching snapshot and delta records, facets, `stats-cache.json`, commands, skills, plans, output styles, agents, `settings.json` with hooks, `installed_plugins.json` and the MCP auth cache.
- [ ] Every `.json` outside `fixtures/synthetic/corrupt/` parses (`node -e` loop over the tree exits 0); every `.jsonl` line outside `corrupt/` parses.
- [ ] The only absolute home paths used are `/Users/fixture/...`.
- [ ] fixtures/README.md describes all fixture sets (synthetic, screenshot, windows, linux, golden, dto, security) and who owns each.

## Dependencies

- M0-03 — Initialise the Heimdall repository and its ignore/attribute rules
- M0-05 — Exhaustive inventory of every ~/.claude path the Swift reference app reads or writes
- M0-06 — Map every PoirotTests case to its Heimdall destination

## Owning agent

test-automator

## Reviewer agent

qa-expert

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
