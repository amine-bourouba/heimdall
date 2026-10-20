# M3-49 · Sub-agent form, import and export

> Milestone M3 · Workstream: Feature screens · Scope: Later

## Goal

Custom sub-agents can be created, edited, deleted, imported and exported.

## Inputs

- `$SWIFT_REF/Poirot/Sources/Views/Config/SubAgentFormView.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Config/SubAgentsListView.swift`
- `$SWIFT_REF/Poirot/Sources/Models/Config/SubAgent.swift`

## Outputs

- `src/features/config/subagents/SubAgentForm.vue` — create
- `src/features/config/subagents/SubAgentsPage.vue` — modify: actions
- `src/features/config/subagents/SubAgentForm.spec.ts` — create

## Acceptance criteria

- [ ] Fields: name, description, tools by category, model, colour, memory, prompt; built-in agents stay read-only.

## Dependencies

- M3-14 — MCP servers and sub-agents screens (read-only)
- M3-44 — Config write commands with native confirmations

## Owning agent

frontend-developer (with typescript-pro for typed IPC)

## Reviewer agent

code-reviewer, accessibility-tester

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
