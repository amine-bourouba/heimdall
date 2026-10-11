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

M2-26 extends both with the real-app half of ADR 0010's malicious-transcript acceptance test (the
mocked-WebKit half is `e2e/security/` below), pointed at `fixtures/security/malicious-claude-home/`
instead of `fixtures/screenshot/claude-home`:
- Windows: `npm run test:e2e:windows-smoke -- fixtures/security/malicious-claude-home` (the script's
  existing `claudeHome` CLI arg; `tauriCdp.ts`'s own `main()` runs the extra checks whenever that
  path contains `"malicious"`). WebView2 is Chromium-based, so this gets the same
  `page.on('request')` network visibility the mocked spec uses.
- Linux: `npm run test:e2e:linux-smoke:security` (sets `HEIMDALL_E2E_CLAUDE_HOME`, which
  `wdio.conf.mjs` reads instead of its own default fixture; `app.smoke.mjs`'s own second
  `describe` block is gated on that same variable, so the ordinary run above never executes it).
  tauri-driver/WebKitGTK's WebDriver protocol has no CDP-style network interception at all, unlike
  Windows' CDP connection or the mocked spec's Playwright one — this half honestly checks window
  count, in-page alert dialogs and the URL's origin, not off-origin requests.

**Neither real-app smoke could be executed on this dev machine**: this machine's own Rust
toolchain is missing `windres` (a pre-existing, already-documented gap since M2-12 — `docs/progress-log.md`),
which blocks `cargo build`/`cargo check` for the `heimdall-desktop` crate entirely, so no debug
binary exists to launch at all. Both files were written to match Tauri's own documented
WebDriver/CDP patterns as closely as possible and were exercised up to the point a real binary
would be needed (`tauriCdp.ts` run directly: a missing-binary spawn failure is caught and reported
cleanly, not an unhandled process crash — a real bug found and fixed this way). Real verification
is Windows and Linux CI.

## Visual baselines (`e2e/visual/`, M2-24, the `visual` Playwright project)

Committed screenshot baselines for the session screens, ported from `PoirotTests/Screenshots/
ScreenshotTests_{Sidebar,SessionDetail,ToolBlocks,ThinkingBlocks,EditDiff,FullViews,Standalone,
Windows}.swift`. Runs through the same mocked-IPC harness as `e2e/mocked/`, but in **WebKit only**:
port-plan.md decision P-13 commits Linux baselines only (WebKitGTK is the weakest/most-divergent
engine, so it is the one worth a committed reference for), and a baseline captured in any other
engine — or on any other OS, since font rendering differs even within WebKit — would be the wrong
answer, not just a noisier one. **Baselines in this repo must come from a real Linux CI run**
(`--update-snapshots` on an `ubuntu-latest` job), never generated locally.

Fixture data for individual block types (every tool, every thinking-block variant, every diff
pair) is ported verbatim from `PoirotTests/Mocks/ScreenshotData.swift` into `e2e/visual/visualData.ts`
— the same hand-built fixtures the real Swift screenshot tests render, not newly invented data.
Each block component under test carries a `data-testid` (`tool-block`, `thinking-block`,
`edit-diff`) specifically so a visual spec can target *one instance* precisely, the same way
Swift's own tests snapshot one `View` in isolation rather than a whole screen.

Run: `npx playwright test --project=visual` (add `--update-snapshots` to regenerate baselines —
only ever meaningful on Linux CI, per the above).

## Malicious transcript (`e2e/security/`, M2-26, the `security` Playwright project)

ADR 0010's own acceptance test: "Playwright run on a malicious fixture transcript asserting no
dialog, navigation, off-origin request or new window." This is the mocked-WebKit half; the
real-app half (Linux and Windows, against `fixtures/security/malicious-claude-home/` instead of
the in-memory mocked session below) is the "Real-app smoke" section above.

The mocked session is built at test time from the M2-25 XSS corpus
(`fixtures/security/xss-payloads.json`), round-robin-split across the four locations that have a
real, built view today (user text, assistant text, tool output, thinking — `e2e/support/
maliciousPayloads.ts` is the one function both this fixture and `fixtures/security/
malicious-claude-home/`'s hand-built content agree on, so they can't silently drift apart). The
plan and memory file buckets have no UI to click through yet (`PlansPage.vue` and
`src/features/memory` are still M2-06-era stubs) — their payloads exist only in the real fixture
tree, ready for whichever M3 task builds those screens.

The spec attaches `page.on('dialog'|'popup'|'request')` listeners before navigating, loads the
session, flips `ToolBlock`'s own "Show rendered markdown" toggle (the one way its tool-output
content actually reaches `MarkdownView`'s v-html path, rather than the raw-text display every
tool shows by default), and clicks the one real link markdown-it's `linkify: true` produces from
otherwise-inert escaped text (a bare `https://evil.example/` substring) — proving
`MarkdownView.vue`'s click-interception holds even for that link, not just that dangerous tags
never parse in the first place.

Run: `npx playwright test --project=security`.

## Performance budgets (`e2e/perf/`, M2-28, the `perf` Playwright project)

port-plan.md §9's two frontend-owned budgets: "Transcript scrolling at 10,000+ blocks" (median
frame ≤ 16.7 ms, p95 ≤ 33 ms) and the JavaScript bundle size ceilings (initial route ≤ 250 KB
gzip, each lazy route ≤ 150 KB gzip, ECharts confined to the analytics chunk). Chromium only —
frame-timing precision matters more here than engine coverage.

`scripts/gen-large-transcript.mjs` generates a 10,000+-block session (not a committed fixture —
too large and synthetic to hand-build; regenerated fresh each run) that
`e2e/perf/transcript-scroll.spec.ts` loads through the usual mocked-IPC harness, then drives a
continuous `scrollTop` increment for 4 seconds while sampling `requestAnimationFrame` deltas.
`scripts/check-bundle-size.mjs` reads `dist/.vite/manifest.json` (`vite.config.ts`'s
`build.manifest: true`) to tell the initial-route chunk and each lazy route's own chunk apart from
Shiki's own ~150 unrelated per-language dynamic-import chunks (several individually over 150 KB
gzip on their own, and not a route at all).

Run: `npx playwright test --project=perf` (scroll budget); `npm run build && node
scripts/check-bundle-size.mjs` (bundle budget). Real measured numbers belong in
`docs/perf-baseline.md`, copied there by hand after a run — neither script writes to it itself.

## Files

| File | Owns |
|---|---|
| `playwright.config.ts` | Five projects, each with its own `testDir`: `chromium`/`webkit` → `e2e/mocked/`, `visual` → `e2e/visual/`, `security` → `e2e/security/`, `perf` → `e2e/perf/`. `e2e/smoke/`'s WebdriverIO/mocha-style suite is never picked up by any of them. |
| `e2e/support/mockIpc.ts` | The `window.__TAURI_INTERNALS__` shape `@tauri-apps/api/core`'s `invoke()` reads, replicated (not imported — `page.addInitScript` serializes a plain function, with no bundler) from `@tauri-apps/api/mocks`'s own real, dependency-free source. Also answers `get_session`/`get_session_messages`/`diff_lines` (a plain LCS line diff, visually reasonable, not Rust's real algorithm) for `e2e/visual/`'s, `e2e/security/`'s and `e2e/perf/`'s own full-session scenarios. |
| `e2e/support/fixtures.ts` | Loads `fixtures/dto/*.json`, shapes it into the IPC responses `mockIpc.ts` answers with, and exports the Playwright fixtures `e2e/mocked/` (`mockedPage`), `e2e/visual/` (`buildVisualFixtures`), `e2e/security/` (`buildMaliciousFixtures`) and `e2e/perf/` (`buildPerfFixtures`) each use. |
| `e2e/support/maliciousPayloads.ts` | Round-robin-splits `fixtures/security/xss-payloads.json` across the six locations ADR 0010's acceptance test names — the one function both `fixtures.ts`'s mocked session and the hand-built `fixtures/security/malicious-claude-home/` tree agree on. |
| `e2e/visual/visualData.ts` | Tool/result/diff/thinking fixtures ported verbatim from `ScreenshotData.swift`. |
| `e2e/visual/session.spec.ts` | The visual baseline spec itself (AC1/AC2). |
| `e2e/security/malicious-transcript.spec.ts` | The mocked-WebKit half of the malicious-transcript acceptance test (M2-26). |
| `scripts/gen-large-transcript.mjs` | Generates a 10,000+-block synthetic session (M2-28), consumed by `fixtures.ts`'s `buildPerfFixtures`. |
| `e2e/perf/transcript-scroll.spec.ts` | The scroll frame-timing spec itself (M2-28). |
| `scripts/check-bundle-size.mjs` | The bundle-size budget check (M2-28), run after `npm run build`. |
| `e2e/support/tauriCdp.ts` | The Windows real-app smoke: the CDP-connection helper, the sidebar-render check, and (M2-26) `assertNoMaliciousTranscriptEscape`, all run directly via `main()`. |
| `e2e/smoke/wdio.conf.mjs` | The Linux real-app smoke's WebdriverIO config (tauri-driver capabilities, binary path, `HEIMDALL_CLAUDE_HOME`); `CLAUDE_HOME` can be overridden via `HEIMDALL_E2E_CLAUDE_HOME` (M2-26). |
| `e2e/smoke/app.smoke.mjs` | The Linux real-app smoke's own spec (mocha-style, run by `wdio`); its second `describe` block (M2-26) only runs when `HEIMDALL_E2E_CLAUDE_HOME` points at the malicious fixture. |
| `e2e/mocked/app.smoke.spec.ts` | The mocked smoke spec itself (AC1) — not in this task's own Outputs list, but a real, necessary addition: nothing else makes that AC mean anything. |
