# E2E tests

Two genuinely different kinds of test live here, for two different reasons (port-plan.md decision P-8).

## Mocked (`e2e/mocked/`, run by Playwright directly)

Playwright drives the **built frontend** (`npm run build && npm run preview`) in a real browser,
with `window.__TAURI_INTERNALS__` mocked (`e2e/support/mockIpc.ts`) so the app believes it is
running inside Tauri. Fixture data comes from `fixtures/dto/` — already real, Rust-generated DTOs
(`fixtures/README.md`), not hand-written test data.

Two browser engines, matching the two non-Windows webviews Heimdall actually ships on:

| Playwright project | Stands in for |
|---|---|
| `chromium` | Windows' WebView2 |
| `webkit` | macOS's WKWebView and Linux's WebKitGTK |

Run: `npm run test:e2e` (builds once, serves `dist/`, runs every spec in `e2e/mocked/` against
both engines).

## Real-app smoke (one real binary launch per OS, no mocks)

Mocked tests prove the frontend *code* works; they cannot prove the real compiled app launches,
loads real prefs/config through Rust, and renders — only an actual OS process can prove that. Each
OS needs a different way to drive the one single real window Heimdall opens, because none of
Windows, macOS and Linux shares a webview technology:

| OS | Mechanism | Why |
|---|---|---|
| Windows | `e2e/support/tauriCdp.ts` — Playwright connects directly over CDP | WebView2 is Chromium-based and exposes a real Chrome DevTools Protocol endpoint once launched with `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=<port>`. No WebDriver server needed at all. |
| Linux | `e2e/smoke/wdio.conf.mjs` + `app.smoke.mjs` — WebdriverIO drives `tauri-driver` | WebKitGTK has its own native WebDriver support; `tauri-driver` (Tauri's own tool) proxies WebDriver commands to it. Playwright cannot speak WebDriver to a browser it doesn't control directly, so this OS needs a separate client (`webdriverio`, added by M2-31). Runs under `xvfb-run` in CI — a container has no real display. |
| macOS | **None — mocked only.** | **Assumption A-10: tauri-driver has no macOS support, because WKWebView has no WebDriver implementation at all** (confirmed in port-plan.md's own assumptions table, not something tied to this project's pinned Tauri version — it's a WebKit/macOS platform gap `tauri-driver` itself cannot work around). There is no real-app smoke for macOS; the mocked WebKit run above is the only coverage this OS gets, and macOS also has no visual baselines for the same underlying reason (port-plan.md P-13: Linux-only baselines). |

Both real-app entry points assert the exact same thing the mocked spec does — `Sidebar.vue`'s own
`<nav aria-label="Primary">` is visible — so a pass means the same real thing everywhere: the app
launched, loaded through the real `HEIMDALL_CLAUDE_HOME` fixture override (`fixtures/screenshot/claude-home`,
debug builds only, `state.rs`, M1-38), and rendered.

Run:
- Windows: `npm run test:e2e:windows-smoke` (needs a debug build at `src-tauri/target/debug/heimdall-desktop.exe` first — `cargo build` in `src-tauri/`).
- Linux: `npm run test:e2e:linux-smoke` (needs a debug build at `src-tauri/target/debug/heimdall-desktop`, and `tauri-driver` installed via `cargo install tauri-driver`; wrap in `xvfb-run` on a CI container with no display).

**Neither real-app smoke could be executed on this dev machine**: this machine's own Rust
toolchain is missing `windres` (a pre-existing, already-documented gap since M2-12 — `docs/progress-log.md`),
which blocks `cargo build`/`cargo check` for the `heimdall-desktop` crate entirely, so no debug
binary exists to launch at all. Both files were written to match Tauri's own documented
WebDriver/CDP patterns as closely as possible and were exercised up to the point a real binary
would be needed (`tauriCdp.ts` run directly: a missing-binary spawn failure is caught and reported
cleanly, not an unhandled process crash — a real bug found and fixed this way). Real verification
is Windows and Linux CI.

## Files

| File | Owns |
|---|---|
| `playwright.config.ts` | The mocked harness's own config — `testDir` scoped to `e2e/mocked/` specifically so Playwright's test runner never picks up `e2e/smoke/`'s WebdriverIO/mocha-style suite. |
| `e2e/support/mockIpc.ts` | The `window.__TAURI_INTERNALS__` shape `@tauri-apps/api/core`'s `invoke()` reads, replicated (not imported — `page.addInitScript` serializes a plain function, with no bundler) from `@tauri-apps/api/mocks`'s own real, dependency-free source. |
| `e2e/support/fixtures.ts` | Loads `fixtures/dto/*.json`, shapes it into the IPC responses `mockIpc.ts` answers with, and exports the one Playwright fixture (`mockedPage`) every mocked spec uses. |
| `e2e/support/tauriCdp.ts` | The Windows real-app smoke, both the CDP-connection helper and (run directly) the smoke check itself. |
| `e2e/smoke/wdio.conf.mjs` | The Linux real-app smoke's WebdriverIO config (tauri-driver capabilities, binary path, `HEIMDALL_CLAUDE_HOME`). |
| `e2e/smoke/app.smoke.mjs` | The Linux real-app smoke's own spec (mocha-style, run by `wdio`). |
| `e2e/mocked/app.smoke.spec.ts` | The mocked smoke spec itself (AC1) — not in this task's own Outputs list, but a real, necessary addition: nothing else makes that AC mean anything. |
