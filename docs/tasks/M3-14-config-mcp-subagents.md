# M3-14 · MCP servers and sub-agents screens (read-only)

> Milestone M3 · Workstream: Feature screens · Scope: MVP

## Goal

MCP servers list from all five sources with a live status dot and key-only env/header display, and sub-agents list built-in and custom agents, read-only.

## Inputs

- `$POIROT/Poirot/Sources/Views/Config/MCPServersListView.swift`
- `$POIROT/Poirot/Sources/Views/Config/MCPServerStatusIndicator.swift`
- `$POIROT/Poirot/Sources/Views/Config/SubAgentsListView.swift`
- `docs/spec/config-screens.md`

## Outputs

- `src/features/config/mcp/McpServersPage.vue` — modify placeholder
- `src/features/config/mcp/McpStatusIndicator.vue` — create
- `src/features/config/subagents/SubAgentsPage.vue` — modify placeholder
- `src/features/config/mcp/index.ts` — modify: available
- `src/features/config/subagents/index.ts` — modify: available
- `src/features/config/mcp/McpServersPage.spec.ts` — create

## Acceptance criteria

- [ ] Statuses come from `get_mcp_statuses`; env and header values are never displayed (only keys).
- [ ] No add, edit or delete controls yet (M3-48, M3-49).

## Dependencies

- M1-45 — MCP read commands
- M3-11 — Shared config list, detail, toolbar, picker and skeleton

## Owning agent

frontend-developer (with typescript-pro for typed IPC)

## Reviewer agent

code-reviewer, accessibility-tester

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
