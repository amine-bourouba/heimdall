# M1-28 · MCP process status on sysinfo

> Milestone M1 · Workstream: Rust data core · Scope: Later

## Goal

MCP status detection matches running processes through `sysinfo` instead of `pgrep`, keeping the npx/uvx-aware pattern rules and the Swift HTTP/SSE behaviour unchanged.

## Inputs

- `$POIROT/Poirot/Sources/Services/Config/MCPServerStatusChecker.swift`
- `$POIROT/PoirotTests/Services/MCPServerStatusCheckerTests.swift`
- `docs/port-plan.md` — decision P-5

## Outputs

- `crates/heimdall-core/src/mcp_status.rs` — fill the placeholder created by M1-05

## Acceptance criteria

- [ ] Carried-over MCPServerStatusCheckerTests pass against an injected process list.
- [ ] Pattern building ports MCPServerStatusChecker.swift:160-214; matching runs on the joined command line.
- [ ] A test covers the Windows shape: `npx.cmd` launching `node.exe …\npm-cache\_npx\…\node_modules\.bin\…` for a scoped package.
- [ ] HTTP/SSE status stays URL well-formedness only (:100-111), labelled as such in code comments (plan decision P-5).
- [ ] The 'any stdio connected implies others failed' heuristic (:15-18) is kept.

## Dependencies

- M1-26 — MCP server (five sources) and plugin readers

## Owning agent

rust-engineer

## Reviewer agent

architect-reviewer

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
