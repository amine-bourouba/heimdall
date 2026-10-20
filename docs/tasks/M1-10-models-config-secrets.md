# M1-10 · Config models, catalogs and the redacting Secret type

> Milestone M1 · Workstream: Rust data core · Scope: MVP

## Goal

Config models, the curated model and MCP catalogs, and a redacting `Secret` newtype exist, keeping every field Claude Code may write (MCP type/url/headers, unknown hook handler fields) while never serialising a secret value.

## Inputs

- `$SWIFT_REF/Poirot/Sources/Models/Config/ClaudeConfig.swift`
- `$SWIFT_REF/Poirot/Sources/Models/Config/MCPServer.swift`
- `$SWIFT_REF/Poirot/Sources/Models/Config/MCPServerCatalog.swift`
- `$SWIFT_REF/Poirot/Sources/Models/Config/Hook.swift`
- `$SWIFT_REF/Poirot/Sources/Models/Config/SubAgent.swift`
- `$SWIFT_REF/Poirot/Sources/Models/Config/ClaudeCommand.swift`
- `$SWIFT_REF/Poirot/Sources/Models/Config/ClaudeSkill.swift`
- `$SWIFT_REF/Poirot/Sources/Models/Config/OutputStyle.swift`
- `$SWIFT_REF/Poirot/Sources/Models/Config/Plan.swift`
- `$SWIFT_REF/Poirot/Sources/Models/Config/MemoryFile.swift`
- `$SWIFT_REF/Poirot/Sources/Models/Config/ClaudePlugin.swift`
- `$SWIFT_REF/Poirot/Sources/Models/Config/ClaudeSettings.swift`
- `$SWIFT_REF/Poirot/Sources/Models/Config/ConfigScope.swift`
- `$SWIFT_REF/Poirot/Sources/Models/ClaudeModelCatalog.swift`
- `$SWIFT_REF/PoirotTests/Models/MCPServerCatalogTests.swift`
- `$SWIFT_REF/PoirotTests/Models/MCPServerStatusTests.swift`
- `$SWIFT_REF/PoirotTests/Models/PlanTests.swift`
- `$SWIFT_REF/PoirotTests/Models/MemoryFileTests.swift`
- `$SWIFT_REF/PoirotTests/Models/SubAgentTests.swift`
- `$SWIFT_REF/PoirotTests/Models/ClaudeModelCatalogTests.swift`
- `docs/adr/0009-safe-writes-to-claude-code-files.md`
- `docs/adr/0010-webview-trust-boundary.md` — Secrets
- `docs/ipc-contract.md`
- `$SWIFT_REF/PoirotTests/Services/SettingsWriterMCPTests.swift` — despite its name, it tests the MCP catalog
- `docs/spec/test-carryover.md`

## Outputs

- `crates/heimdall-core/src/models/config/*.rs` — create: scope, mcp, hook, sub_agent, command, skill, output_style, plan, memory_file, plugin, settings
- `crates/heimdall-core/src/models/catalog/claude_models.rs` — fill the placeholder created by M1-05
- `crates/heimdall-core/src/models/catalog/mcp_catalog.rs` — fill the placeholder created by M1-05
- `crates/heimdall-core/src/models/secret.rs` — fill the placeholder created by M1-05

## Acceptance criteria

- [ ] Carried-over model tests pass.
- [ ] `Secret` implements `Debug` and `Display` as `[redacted]` and has no `Serialize` impl; IPC DTOs expose `envKeys`/`headerKeys` only.
- [ ] A test serialises an MCP server DTO built from `env: {"API_KEY": "sk-123"}` and asserts the JSON has no `sk-123`.
- [ ] Hook handlers keep unknown keys in an `extra` map that round-trips; MCP definitions keep `type`, `url` and `headers`.
- [ ] Catalog data matches the Swift catalogs entry for entry (a test compares counts and ids).

## Dependencies

- M1-05 — Declare every heimdall-core module and implement AppError
- M0-06 — Map every PoirotTests case to its Heimdall destination

## Owning agent

rust-engineer

## Reviewer agent

architect-reviewer, security-auditor

## Size

L

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
