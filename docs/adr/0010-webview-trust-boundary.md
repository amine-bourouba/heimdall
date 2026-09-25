# ADR 0010: Webview trust boundary

- **Status:** Accepted
- **Date:** 2026-09-25

## Context

**Threat model.** Transcripts, tool output, plans and memory files can contain attacker-influenced text (for example from a web page an agent read). Heimdall renders that text in a webview. The webview can call IPC commands that write hooks (`~/.claude/settings.json`) and MCP servers (`~/.claude.json`), which Claude Code later executes. So an XSS bug in the webview can escalate to persistent code execution. Editors and terminals are launched with transcript-derived paths. Poirot has no OS sandbox (`Poirot/Resources/Poirot.entitlements` is an empty dictionary; hardened runtime only), and MarkdownUI never executed HTML [A-4], so a webview Markdown renderer is a new XSS surface. **The webview-to-IPC boundary is the only real control.**

## Decision

Ten controls. Each lists the tasks that implement and prove it; M4-14 collects the evidence for security-auditor sign-off.

1. **Rendering.** `renderMarkdown()` is the only path into `v-html`, inside `src/lib/markdown/MarkdownView.vue` (ESLint `vue/no-v-html` everywhere else). markdown-it runs with `html: false`. DOMPurify uses explicit tag and attribute allowlists; URIs are limited to `http:`, `https:`, `mailto:` and `#`; `style` and `on*` attributes are stripped. Images never load; they render as alt text. Shiki output is converted to classes and sanitised; its CSS is injected as a constructable stylesheet, so the CSP needs no `unsafe-inline` (port-plan P-16). ECharts tooltips escape data-derived strings with `echarts.format.encodeHTML`. *Tasks: M1-04, M2-10, M3-33; tests M2-25, M3-35.*
2. **Links.** One Rust command, `open_external_url`, opens only `http`, `https` and `mailto` URLs through the opener plugin's Rust API; it rejects `file:`, `javascript:`, `data:`, custom schemes, UNC-style hosts and embedded credentials. The main window refuses navigation away from the app origin and refuses new windows. Toast and update URLs go through the same check. *Tasks: M1-33, M2-11, M2-12, M4-04; test M2-26.*
3. **CSP.** Production: `default-src 'self'; script-src 'self'; img-src 'self' data: blob:; connect-src ipc: http://ipc.localhost; object-src 'none'; base-uri 'none'; form-action 'none'; frame-src 'none'`. Development (`devCsp`, used only by `tauri dev`): `default-src 'self'; script-src 'self' http://localhost:*; style-src 'self' 'unsafe-inline' http://localhost:*; img-src 'self' data: blob: http://localhost:*; connect-src ipc: http://ipc.localhost http://localhost:* ws://localhost:*; object-src 'none'; base-uri 'none'; form-action 'none'; frame-src 'none'`. It allows the Vite dev server on any worktree port and the inline `<style>` tags Vite injects during development. The production CSP never contains `unsafe-inline`, and neither CSP contains `unsafe-eval`. `freezePrototype: true`; asset protocol disabled; `withGlobalTauri: false`. Behaviour must be verified on WebView2, WebKitGTK and WKWebView [A-5]. *Tasks: M1-03, M2-27; tests M2-26, M4-12.*
4. **Capabilities.** One capability file per window. Main: the app commands marked `main` in the contract, event listening, clipboard write (text and image). Tray: only `get_tray_snapshot`, `open_in_main_window`, `get_prefs`. No `fs` or `shell` plugin, no `opener:allow-open-path`, no remote URLs. Commands are scoped with `AppManifest::commands` in `build.rs`, and the default exposure of commands across windows is verified against the pinned Tauri version's docs. *Tasks: M1-48, M1-49, M3-59; lint M2-27.*
5. **IPC shape.** Commands take ids and enums, never raw paths. Rust resolves every id under known roots. Project-scoped writes accept only a known project id or the folder chosen through a Rust-opened dialog. *Tasks: M1-37, M1-44; contract §2, §4.*
6. **Code-execution writes.** Writing a hook command or URL, an MCP command, arguments or environment, or importing hooks first shows a native (Rust-side) confirmation with the exact values. Writes fail closed on parse errors, keep a one-time `.bak`, and replace atomically while preserving unknown keys, key order and Unix mode (ADR 0009). *Tasks: M1-22, M3-40, M3-41, M3-44, M3-45.*
7. **Process launch.** `std::process::Command` with argument arrays only, never a shell string. Rust ≥ 1.77.2 (CVE-2024-24576). Windows `.cmd` and `.bat` targets are never spawned; editors use URL schemes such as `vscode://file/...` where verified. Launch paths must be absolute, canonical, existing regular files; a leading `-` and UNC paths are rejected. Session ids must match the contract's pattern. Terminal launch copies the resume command and opens at the project folder, with no command text passed to the terminal (Windows Terminal treats `;` as a separator). *Tasks: M1-01, M1-32, M3-52, M3-53, M3-54.*
8. **Secrets.** Every MCP `env` and `headers` value (`Models/Config/ClaudeConfig.swift:23-25`) is a secret. Values live in a redacting `Secret` newtype with no `Serialize`; IPC responses carry only key names; edits send `keep`, `set` or `remove` markers. Debug output and any future logs record keys, paths and counts only. *Tasks: M1-10, M1-26, M1-45, M3-41, M3-48; test M1-55.*
9. **Updates and signing.** The Tauri updater verifies a minisign signature against a public key in the config and fetches only over HTTPS from the fork's GitHub Releases. `TAURI_SIGNING_PRIVATE_KEY` and its password are CI secrets with an offline backup and a rotation procedure. Windows and macOS builds are unsigned for now (user answer 7; macOS builds are ad-hoc signed). Linux artifacts ship with SHA-256 sums. *Tasks: M4-02, M4-03, M4-06, M4-10.*
10. **Supply chain.** `cargo deny check` and `npm audit --audit-level=high` in CI; committed lockfiles and `npm ci`; every GitHub Action pinned by SHA with minimal per-job permissions (Poirot's release workflow pinned by tag and granted `contents: write` to the whole workflow, `release.yml:8-9, 16`); Dependabot for cargo, npm and Actions; GitHub secret scanning enabled on the repository. *Tasks: M1-01, M1-02, M1-58, M2-29, M4-06, M4-10.*

## Acceptance tests

- Vitest XSS corpus through `renderMarkdown()` and every chart formatter (M2-25, M3-35).
- Playwright run on a malicious fixture transcript asserting no dialog, navigation, off-origin request or new window (M2-26).
- A configuration lint for CSP and capability regressions (M2-27).
- Rust tests proving a corrupt config produces no write (M1-22, M3-40) and that secrets never reach DTOs or debug output (M1-55).
- Cross-OS launcher tests rejecting or escaping spaces, `;&|%^"`, a leading `-` and UNC paths (M1-32).
- A release checklist proving updater signatures verify and checksums match on every OS (M4-10).

## Consequences

- Features that need the file system or processes always go through a narrow, reviewed Rust command.
- Any change to commands, capabilities, the CSP or write paths needs security-auditor sign-off (port-plan §5.6).

## Related

`docs/ipc-contract.md` · ADR 0004, 0005, 0009 · Port-plan §5.6
