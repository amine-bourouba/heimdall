# M1-26 · MCP server (five sources) and plugin readers

> Milestone M1 · Workstream: Rust data core · Scope: MVP

## Goal

MCP servers load from all five sources with the Swift precedence, plugins load from `installed_plugins.json`, and every env/header value is held as a `Secret`.

## Inputs

- `$SWIFT_REF/Poirot/Sources/Services/Config/ClaudeConfigLoader.swift:111-381`
- `$SWIFT_REF/Poirot/Sources/Services/Config/MCPServerStatusChecker.swift:1-96`
- `$SWIFT_REF/PoirotTests/Services/ClaudeConfigLoaderMCPTests.swift`
- `docs/adr/0010-webview-trust-boundary.md` — Secrets

## Outputs

- `crates/heimdall-core/src/config/read/mcp.rs` — fill the placeholder created by M1-05
- `crates/heimdall-core/src/config/read/plugins.rs` — fill the placeholder created by M1-05

## Acceptance criteria

- [ ] Carried-over ClaudeConfigLoaderMCPTests pass.
- [ ] Sources and precedence: user (`~/.claude.json` mcpServers), local (`projects[<path>].mcpServers`, overrides user), project (`<project>/.mcp.json`), plugin servers, cloud integrations (auth-cache keys prefixed `claude.ai `).
- [ ] Tool permissions are collected from settings.json keyed by server name.

## Dependencies

- M1-06 — Claude home resolution, path normalisation and comparison keys
- M1-10 — Config models, catalogs and the redacting Secret type

## Owning agent

rust-engineer

## Reviewer agent

architect-reviewer, security-auditor

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
