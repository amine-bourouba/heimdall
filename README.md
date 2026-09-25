# Heimdall

A desktop companion for [Claude Code](https://claude.com/claude-code) on Windows, Linux and macOS. Heimdall reads Claude Code's own data under `~/.claude/` and shows your sessions, history and configuration in one place. It runs entirely on your machine: no account, no server, no telemetry.

Heimdall is a cross-platform port of [Poirot](https://github.com/a7t-ai/poirot), built with Tauri 2, Rust and Vue 3.

> **Status: under construction.** Nothing is released yet. The port follows the plan in [`docs/port-plan.md`](docs/port-plan.md); progress is recorded in [`docs/progress-log.md`](docs/progress-log.md).

## Planned for the first release

- **Session browser:** every project and session, including sub-agent sessions, with sorting and search.
- **Conversation view:** Markdown, thinking, tool calls and results, system reminders and edit diffs, with a context-window card and a tool timeline.
- **Session details:** AI summaries, per-session todos, file-history versions with diffs, and the debug log.
- **Export:** a session as Markdown (copy or save), or an edit as a patch.
- **Prompt history, plans, memory and TODOs** across all projects.
- **Universal search** (Ctrl/Cmd+K) across sessions, history, config and more.
- **Read-only config views:** commands, skills, output styles, hooks, plugins, models, MCP servers with live status, and sub-agents.
- **Themes and font scaling,** with keyboard navigation throughout.

The full feature list, with the status of each feature per OS, is in [`docs/parity-matrix.md`](docs/parity-matrix.md).

## Development

See [`AI_INSTRUCTIONS.md`](AI_INSTRUCTIONS.md) for the project rules and commands.

## License

MIT. See [`LICENSE`](LICENSE) and [`NOTICE`](NOTICE). Heimdall is derived from Poirot by Leonardo Cardoso, also MIT-licensed.
