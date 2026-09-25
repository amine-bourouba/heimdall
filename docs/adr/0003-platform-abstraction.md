# ADR 0003: Platform abstraction

- **Status:** Accepted
- **Date:** 2026-09-25

## Context

Poirot is macOS-only in several places: every path comes from `homeDirectoryForCurrentUser` (for example `SessionLoader.swift:66`, `SettingsWriter.swift:5`); project paths are reconstructed from hyphen-encoded folder names (`SessionLoader.swift:98, 314-353`, `TodosOverviewView.swift:375`); terminals launch through AppleScript and bundle ids (`TerminalLauncher.swift`); editors use bundle ids and Homebrew paths (`EditorLauncher.swift:23-48`); `claude` is found by probing four macOS paths (`OnboardingView.swift:281-297`); MCP status shells out to `/usr/bin/pgrep` (`MCPServerStatusChecker.swift:222-236`).

## Decision

### Structure

- **Pure decisions in core, execution in src-tauri.** Core functions take a `TargetOs` value and return data (paths, argv, URLs, verdicts). `src-tauri/src/platform/{windows,macos,linux}.rs` executes them under `#[cfg(target_os)]`. No platform trait.

### Paths and case sensitivity

- The Claude home comes from `dirs::home_dir()` joined with `.claude` (and `.claude.json`), through one helper in `paths.rs`. A config-dir override is honoured only if M1-06 verifies Claude Code supports one [A-2].
- **Project path resolution order:** the `cwd` whose hyphen-encoding equals the folder name, compared case-insensitively → the most frequent `cwd` in the project's transcripts → the sessions index (`projectPath` or `originalPath`, both accepted; port-plan C-7) → folder-name decoding as a last resort. Records carry the shell's current folder, which moves when a session changes directory, and Claude Code writes both `C:` and `c:` drive letters (fixtures/windows/MANIFEST.md). `ProjectSummary.pathSource` reports which won. Both Swift `-`→`/` fallbacks are retired.
- One normaliser accepts `\` and `/`, drive letters, `\\?\` prefixes and trailing separators. Comparison keys are case-insensitive on Windows and macOS (default file systems) and case-sensitive on Linux. Keys are for comparison only; the original string is always displayed. `FileHistoryLoader.sameFile` suffix matching is replaced by this utility.
- UNC paths are recognised and rejected wherever a path would be launched (ADR 0010).

### Launchers

| | Windows | Linux | macOS |
|---|---|---|---|
| Terminals (`PreferredTerminal` per OS) | Windows Terminal `wt.exe -d <cwd>`; PowerShell; `cmd.exe` | GNOME Terminal `--working-directory`; Konsole `--workdir`; kitty `--directory`; Alacritty `--working-directory`; `x-terminal-emulator` with the process cwd | Terminal, iTerm2, Warp, Ghostty, kitty, Alacritty via `open -a <App> <cwd>` |
| Behaviour | Copy `claude --resume <uuid>` to the clipboard, then open the terminal at the project folder; no command is injected (keeps Poirot's `TerminalLauncher.launch` user flow without AppleScript). `wt` treats `;` as a separator, so no command text is ever passed. | same | same |
| Editors (`PreferredEditor`) | `vscode://file/<abs>:<line>`, `cursor://file/...`, `zed://file/...` where M3-54 confirms them; never spawn `.cmd` shims | URL schemes, else CLI found by PATH search | URL schemes, else CLI; Xcode via `xed` (macOS only) |
| `claude` detection | PATH search with PATHEXT (`claude.cmd`, `claude.exe`), then `%APPDATA%\npm` | PATH, then `~/.local/bin`, `~/.npm-global/bin` | PATH, then `/opt/homebrew/bin`, `/usr/local/bin`, `~/.local/bin` |
| Reveal in file manager | opener `reveal_item_in_dir` | opener; degrades to opening the parent folder | opener |

### OS-divergent features (one outcome each, recorded in the parity matrix)

| Feature | Windows | Linux | macOS |
|---|---|---|---|
| Tray (menu-bar popover) | Tray window | **Degrade:** native menu with recent sessions; GNOME needs the AppIndicator extension | Tray window |
| Glass background | `backdrop-filter` | **Degrade** to opaque if WebKitGTK lacks it [A-11] | `backdrop-filter` |
| Launch at login | **Replace:** autostart plugin (Run key) | **Replace:** XDG autostart | **Replace:** LaunchAgent instead of `SMAppService` |
| Title bar | **Replace:** native decorations | **Replace:** native decorations | Overlay title bar (parity) |
| Share sheet | not applicable: Poirot has none (port-plan C-13) | | |

### MCP process detection

`sysinfo` supplies the process list; core matches command lines with the Swift npx/uvx-aware patterns. On Windows, `npx` runs through `npx.cmd`, which launches `node.exe` with the resolved script path; that shape is tested explicitly (M1-28).

## Consequences

- Every OS branch of every launcher and path rule runs in CI on every host, because core never uses `#[cfg]` for decisions.
- Terminal launching loses Poirot's AppleScript "run this command" path, which had no callers anyway (port-plan C-5).
- Linux users on GNOME get a reduced tray unless they install the extension.

## Related

Parity matrix sections A, C, L · ADR 0002, 0010 · Tasks M1-06, M1-14, M1-19, M1-28, M1-31, M1-32, M3-52 to M3-56
