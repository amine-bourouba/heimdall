# ADR 0007: Preferences storage and migration

- **Status:** Accepted
- **Date:** 2026-09-25

## Context

Poirot keeps 18 preference keys in `UserDefaults`, read across 19 files, some outside the UI layer (`App/MemorySourcesStore.swift:52`, `Utilities/TerminalLauncher.swift:10`). Rust needs several of them (terminal, editor, custom memory folders), so preferences cannot be a UI-only concept. The UserDefaults domain is uncertain (the bundle id `fyi.poirot-54` in `project.yml:41` is a per-worktree leftover), and only three keys hold user input worth keeping: `customMemoryFolders`, `claudeCodePath`, `configProjectPath`.

## Decision

1. **One typed `Prefs` struct owned by Rust** (`heimdall-core/src/models/prefs.rs`) with `#[serde(default)]` and `schemaVersion: 1`. Unknown keys are ignored; missing keys take defaults.
2. **Stored by the store plugin** as `prefs.json` in the OS app-config folder for the app identifier [A-8]: `%APPDATA%\<id>` on Windows, `~/Library/Application Support/<id>` on macOS, `~/.config/<id>` on Linux.
3. **The webview gets `get_prefs` and `update_prefs` only**, with no store-plugin permission. `update_prefs` validates every field. `customMemoryFolders` and `configProjectPath` change only through commands that open a Rust-side folder picker (`add_memory_folder`, `remove_memory_folder`, `pick_config_project`), so the webview never supplies a path.
4. **Keys and defaults:**

| Key | Type | Default | Note |
|---|---|---|---|
| `schemaVersion` | number | 1 | Bumped with a migration function |
| `showMenuBarIcon` | bool | true | |
| `hasCompletedOnboarding` | bool | false | |
| `colorTheme` | enum (20 ids) | `default` | |
| `fontScale` | number | 1.0 | 0.75–1.5 in 0.05 steps |
| `sessionLayout` | `grid` \| `list` | `grid` | |
| `configProjectPath` | string \| null | null | Set only by `pick_config_project` |
| `configLayouts` | map screen → `grid` \| `list` | {} | |
| `showAgentSessions` | bool | false | |
| `textEditor` | enum | `vscode` | Xcode offered on macOS only |
| `preferredTerminal` | enum per OS | macOS `terminal`, Windows `windowsTerminal`, Linux `system` (`x-terminal-emulator`) | ADR 0003 |
| `openTerminalOnBash` | bool | false | |
| `claudeCodePath` | string \| null | null | Poirot defaulted to `/usr/local/bin/claude`; null means PATH search |
| `showAnimations` | bool | true | |
| `wrapCodeLines` | bool | true | |
| `autoExpandBlocks` | bool | true | |
| `parseMarkdownInResults` | bool | true | |
| `globalSearchMode` | `fuzzy` \| `exact` | `fuzzy` | |
| `customMemoryFolders` | string[] | [] | |

Launch at login is not a preference; it is read live from the autostart plugin.

5. **No import from Poirot's UserDefaults.** The domain is uncertain and the few valuable keys take seconds to re-enter.
6. **Never write app state under `~/.claude`.**
7. **Development builds use their own identifier** (`io.github.amine-bourouba.heimdall.dev`, and `io.github.amine-bourouba.heimdall.dev<N>` per worktree; port-plan §5.8), so they never touch release preferences.
8. **The identifier is final** (`io.github.amine-bourouba.heimdall`, D1, ADR 0011) and must not change after the first tagged release: changing it would orphan every user's preferences.

## Consequences

- Rust code reads preferences directly; no preference flows through the UI to reach a launcher.
- The migration hook exists from v1, so future key changes are code, not guesswork.

## Related

Parity matrix section I · ADR 0003, 0010, 0011 · Tasks M1-11, M1-39, M2-05, M3-16
