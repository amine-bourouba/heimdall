# M3-55 · Editor, terminal and reveal buttons

> Milestone M3 · Workstream: Feature screens · Scope: Later

## Goal

Tool blocks, the session toolbar, config detail and the MCP screen gain open-in-editor, open-terminal and reveal actions.

## Inputs

- `$SWIFT_REF/Poirot/Sources/Views/Session/ToolBlockView.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Session/SessionToolbarContent.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Config/ConfigDetailToolbar.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Config/MCPServersListView.swift:250-260`

## Outputs

- `src/features/session/blocks/ToolBlock.vue` — modify
- `src/features/session/SessionToolbar.vue` — modify
- `src/features/config/shared/ConfigDetailToolbar.vue` — modify
- `src/features/config/mcp/McpServersPage.vue` — modify

## Acceptance criteria

- [ ] `openTerminalOnBash` opens a terminal for Bash tool blocks when enabled.
- [ ] Buttons are hidden for editors or terminals not available on the running OS.

## Dependencies

- M2-18 — Tool-use and tool-result block
- M2-22 — Session toolbar
- M3-11 — Shared config list, detail, toolbar, picker and skeleton
- M3-48 — MCP setup wizard and server actions
- M3-53 — Editor, terminal and reveal commands

## Owning agent

frontend-developer (with typescript-pro for typed IPC)

## Reviewer agent

code-reviewer, accessibility-tester

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
