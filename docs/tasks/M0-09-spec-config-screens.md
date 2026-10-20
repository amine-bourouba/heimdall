# M0-09 · Behaviour spec for the configuration screens, forms and MCP wizard

> Milestone M0 · Workstream: Legacy parity extraction · Scope: Infra

## Goal

A spec captures list, detail, form and wizard behaviour for all eight configuration screens, including the Swift weaknesses the port must not copy.

## Inputs

- `$SWIFT_REF/Poirot/Sources/Views/Config/CommandsListView.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Config/SkillsListView.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Config/OutputStylesListView.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Config/HooksListView.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Config/HookFormView.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Config/MCPServersListView.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Config/MCPSetupWizardView.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Config/MCPServerStatusIndicator.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Config/ModelsListView.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Config/PluginsListView.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Config/SubAgentsListView.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Config/SubAgentFormView.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Config/ConfigDetailToolbar.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Config/ConfigProjectPicker.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Config/ConfigItemDetailView.swift`
- `$SWIFT_REF/Poirot/Sources/Models/Config/MCPServerCatalog.swift`
- `$SWIFT_REF/Poirot/Sources/Models/Config/Hook.swift`
- `$SWIFT_REF/Poirot/Sources/Models/Config/SubAgent.swift`
- `docs/adr/0009-safe-writes-to-claude-code-files.md`
- `docs/adr/0010-webview-trust-boundary.md`

## Outputs

- `docs/spec/config-screens.md` — create

## Acceptance criteria

- [ ] Each screen has sections: data shown, filters, layouts, actions (read-only versus mutating), keyboard (⌘. / Return in forms).
- [ ] The MCP wizard section lists every step, catalog field, validation rule and the preview JSON.
- [ ] A 'Do not port' section repeats with line numbers: naive space-split of custom args (MCPSetupWizardView.swift:66-70), 4-character secret preview that skips custom servers (:561-562), updateMCPServer dropping type/url/headers (SettingsWriter.swift:150-177), hook handler serialisation dropping unknown fields (SettingsWriter.swift:296-314).

## Dependencies

- M0-05 — Exhaustive inventory of every ~/.claude path the Swift reference app reads or writes

## Owning agent

legacy-modernizer (consults swift-expert, who only reads the Swift app)

## Reviewer agent

qa-expert

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
