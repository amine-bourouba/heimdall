# M3-48 · MCP setup wizard and server actions

> Milestone M3 · Workstream: Feature screens · Scope: Later

## Goal

The MCP wizard adds and edits servers from the catalog or custom input, with arguments edited as a list, secrets never pre-filled, and a fully masked preview.

## Inputs

- `$SWIFT_REF/Poirot/Sources/Views/Config/MCPSetupWizardView.swift`
- `$SWIFT_REF/Poirot/Sources/Models/Config/MCPServerCatalog.swift`
- `docs/spec/config-screens.md`

## Outputs

- `src/features/config/mcp/McpSetupWizard.vue` — create
- `src/features/config/mcp/ArgsEditor.vue` — create
- `src/features/config/mcp/McpServersPage.vue` — modify: actions
- `src/features/config/mcp/McpSetupWizard.spec.ts` — create
- `src/features/config/mcp/index.ts` — modify: canWrite

## Acceptance criteria

- [ ] Custom arguments are an ordered list editor, never split on spaces (fix for MCPSetupWizardView.swift:66-70).
- [ ] The preview masks every env and header value completely, for catalog and custom servers (fix for :561-562).
- [ ] Editing an existing server sends `keep` for untouched secrets.

## Dependencies

- M0-09 — Behaviour spec for the configuration screens, forms and MCP wizard
- M3-14 — MCP servers and sub-agents screens (read-only)
- M3-45 — MCP write commands with native confirmations

## Owning agent

frontend-developer (with typescript-pro for typed IPC)

## Reviewer agent

code-reviewer, accessibility-tester, security-auditor

## Size

L

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
