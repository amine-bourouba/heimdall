# M0-09 · Behaviour spec for the configuration screens, forms and MCP wizard

> Milestone M0 · Workstream: Legacy parity extraction · Scope: Infra

## Goal

A spec captures list, detail, form and wizard behaviour for all eight configuration screens, including the Swift weaknesses the port must not copy.

## Inputs

- `$POIROT/Poirot/Sources/Views/Config/CommandsListView.swift`
- `$POIROT/Poirot/Sources/Views/Config/SkillsListView.swift`
- `$POIROT/Poirot/Sources/Views/Config/OutputStylesListView.swift`
- `$POIROT/Poirot/Sources/Views/Config/HooksListView.swift`
- `$POIROT/Poirot/Sources/Views/Config/HookFormView.swift`
- `$POIROT/Poirot/Sources/Views/Config/MCPServersListView.swift`
- `$POIROT/Poirot/Sources/Views/Config/MCPSetupWizardView.swift`
- `$POIROT/Poirot/Sources/Views/Config/MCPServerStatusIndicator.swift`
- `$POIROT/Poirot/Sources/Views/Config/ModelsListView.swift`
- `$POIROT/Poirot/Sources/Views/Config/PluginsListView.swift`
- `$POIROT/Poirot/Sources/Views/Config/SubAgentsListView.swift`
- `$POIROT/Poirot/Sources/Views/Config/SubAgentFormView.swift`
- `$POIROT/Poirot/Sources/Views/Config/ConfigDetailToolbar.swift`
- `$POIROT/Poirot/Sources/Views/Config/ConfigProjectPicker.swift`
- `$POIROT/Poirot/Sources/Views/Config/ConfigItemDetailView.swift`
- `$POIROT/Poirot/Sources/Models/Config/MCPServerCatalog.swift`
- `$POIROT/Poirot/Sources/Models/Config/Hook.swift`
- `$POIROT/Poirot/Sources/Models/Config/SubAgent.swift`
- `docs/adr/0009-safe-writes-to-claude-code-files.md`
- `docs/adr/0010-webview-trust-boundary.md`

## Outputs

- `docs/spec/config-screens.md` — create

## Acceptance criteria

- [ ] Each screen has sections: data shown, filters, layouts, actions (read-only versus mutating), keyboard (⌘. / Return in forms).
- [ ] The MCP wizard section lists every step, catalog field, validation rule and the preview JSON.
- [ ] A 'Do not port' section repeats with line numbers: naive space-split of custom args (MCPSetupWizardView.swift:66-70), 4-character secret preview that skips custom servers (:561-562), updateMCPServer dropping type/url/headers (SettingsWriter.swift:150-177), hook handler serialisation dropping unknown fields (SettingsWriter.swift:296-314).

## Dependencies

- M0-05 — Exhaustive inventory of every ~/.claude path Poirot reads or writes

## Owning agent

legacy-modernizer (consults swift-expert, who only reads the Swift app)

## Reviewer agent

qa-expert

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
