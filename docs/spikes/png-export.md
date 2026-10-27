# Spike: PNG export approach (M3-37, open risk R-7)

Compares in-webview DOM-to-image rendering against Rust-side rendering for the two features that
need a PNG: the analytics dashboard export, and the session diff's "copy as image." Backed by a
real, working prototype (not just library documentation) run against Heimdall's own exact
production CSP string, not a relaxed or assumed one.

## The Swift reference app's own approach — read directly, not assumed

Both `AnalyticsImageExporter.swift` and `EditDiffView.swift:216-239` use SwiftUI's `ImageRenderer`
(an AppKit-only facility, no cross-platform equivalent — this is exactly why this spike exists,
there is no "just port this" option) against a **dedicated, separate, non-interactive view tree**,
never a literal screenshot of the live interactive screen:

- `AnalyticsImageExporter.renderToImage` composes `AnalyticsBrandHeader() + content` at a fixed
  1280px width, forces `.environment(\.colorScheme, .dark)`, and sets a flat
  `.background(PoirotTheme.Colors.bgApp)` — no translucency/blur of any kind.
- `EditDiffView.copyDiffAsImage()` renders a dedicated `DiffSnapshotView(diffLines:gutterWidth:)`,
  writes the PNG to both `NSPasteboard` and a save panel in the same action.

This matters for Heimdall's own design, not just as trivia: it confirms the export surface was
never expected to visually match the live chrome pixel-for-pixel, which directly resolves the one
fidelity gap this spike's own prototype found below (backdrop-filter).

## Candidates compared

1. **In-webview DOM-to-image**: a JS library captures rendered DOM (including the ECharts
   `<canvas>`) into a bitmap, sent to Rust over IPC to write to disk / hand to the clipboard.
2. **Rust-side rendering**: Rust re-renders the same data natively with no DOM capture at all.

## Method

A standalone test page (not part of the Heimdall build) was served over a local HTTP server (not
`file://`, since CSP enforcement differs by scheme) with a `<meta http-equiv="Content-Security-Policy">`
tag set to the **exact, verbatim `csp` string from `src-tauri/tauri.conf.json`** (not `devCsp`,
which explicitly allows `'unsafe-inline'` and would have produced a false pass):

```
default-src 'self'; script-src 'self'; img-src 'self' data: blob:; connect-src ipc: http://ipc.localhost; object-src 'none'; base-uri 'none'; form-action 'none'; frame-src 'none'
```

The page renders a real ECharts bar chart (same `BarChart`/`CanvasRenderer` modules M3-33 already
uses) inside a card using Heimdall's own real `.glass` CSS rule verbatim
(`background-color: color-mix(in srgb, var(--bg-elevated) 72%, transparent); backdrop-filter: blur(20px);`,
copied from `src/styles/main.css:96-99`) over a colourful gradient background, so backdrop-filter
has something visible to blur. Driven with Playwright/Chromium (already a project dependency),
listening for the `securitypolicyviolation` DOM event and reading Chromium's own console CSP
warnings, with every captured PNG saved to disk and **visually inspected**, not just checked for
"no exception thrown" — an early run's crude "count non-background pixels" heuristic reported a
capture as successful when the saved image turned out to contain only a text heading and nothing
else, which is why every result below was confirmed by actually looking at the output image.

**Two real self-inflicted bugs in the test harness itself, found and fixed before trusting any
result**: an inline `<style>` block and an inline `style="..."` attribute in the test page are
themselves blocked by this exact CSP (no `style-src`, falling back to `default-src 'self'`, no
`'unsafe-inline'`) — moving all CSS to an external stylesheet was required before the chart div
even had a non-zero height to render into. This is a real confirmation that the CSP model works as
documented, not a library finding.

**Platform coverage — stated honestly.** This spike ran on Windows/WebView2 (Chromium-based) only,
the one engine available on this machine. WebKitGTK (Linux) and WKWebView (macOS) findings below
are research-based (citations in the Research appendix), not independently reproduced — the single
largest open gap this spike leaves, same class of limitation already recorded for A1/visual checks
elsewhere in this port (M2-30's own sampling-limitation precedent).

## Results

| Library | CSP violations observed | Canvas (ECharts) capture | `color-mix()` (Heimdall's real `.glass` rule) | `backdrop-filter` | Minified / gzip size (measured) |
|---|---|---|---|---|---|
| `html2canvas` 1.4.1 (original) | 1 (`style-src` fallback, non-fatal) | **Blank without a manual fix** — `cloneNode()` does not preserve a live canvas's drawn bitmap; needs an `onclone` callback that manually `drawImage()`s each live canvas into its clone. Works once that workaround is added. | **Crashes**: `Attempting to parse an unsupported color function "color"` — this package is unmaintained (last published ~Jan 2022) and has no support for the modern CSS this app's own theme already uses. | Silently dropped (flat colour, no blur) | 198.7 KB / 45.9 KB gzip |
| `html2canvas-pro` 2.5.2 (actively maintained fork) | 2 (`style-src` fallback; **also trips `base-uri 'none'`** — its clone step sets the cloned document's base URI, which this app's CSP explicitly forbids) | Same `onclone` workaround needed as the original | **Works**, no crash | Silently dropped (flat colour, no blur) | 209.9 KB / 63.1 KB gzip |
| `dom-to-image-more` 3.9.1 | **0** | **Works out of the box**, no manual fix needed | **Works**, no crash | Silently dropped (flat colour, no blur) | 29.0 KB / **10.1 KB gzip** |

All CSP violations observed were non-fatal — every capture that "succeeded" per the table above
produced a visually correct PNG (chart bars, axis labels, card chrome all present and undistorted,
confirmed by eye) despite the console warnings. None of the three could reproduce `backdrop-filter`
— a structural limitation of DOM-redraw techniques (they rebuild the image node-by-node via Canvas
2D or SVG `foreignObject`, never sampling the real compositor's backdrop), not a library bug, and
not something a different library choice would fix.

## Rust-side alternative (not prototyped — evaluated on its real, concrete requirements)

`resvg` + `usvg` + `tiny-skia` is a mature, real Rust SVG-rasterization pipeline. Paired with
ECharts' own SVG renderer mode (`echarts.use([SVGRenderer, ...])`, a separate module from the
`CanvasRenderer` M3-33 already uses — not bundled by default, real bundle-size cost not measured
here), this is a genuine hybrid option: generate the chart as an SVG string client-side, ship the
string over IPC, rasterize in Rust. It sidesteps CSP and canvas-capture concerns entirely, since no
DOM screenshot technique is involved at all. The diff-image case would need either a small, bespoke
Rust text-layout pass (the Swift `DiffSnapshotView` is plain monospace text with coloured line
backgrounds — not visually complex) or its own small client-generated SVG string.

Real costs this spike did not try to minimise away: a second ECharts renderer module to maintain
and bundle-size-budget (M3-32/33's own `check-bundle-size.mjs` gate would need a real run to
confirm the cost); a new Rust dependency chain (`resvg`/`usvg`/`tiny-skia`) needing its own
`cargo deny` licence/advisory review, the same gate that's already flagged real transitive
advisories elsewhere in this project's own dependency tree (see the M3-33/M3-35 commits' own CI
runs); and — the biggest one — this still needs the SVG string generated client-side from the same
Vue components, so it does not actually remove the frontend's involvement, it only moves the final
rasterization step to Rust.

## Recommendation (closes R-7)

**In-webview DOM-to-image, using `dom-to-image-more`**, not the Rust-side option — so, per this
task's own AC2, **M3-39's Outputs are not rewritten.**

Reasoning, each point backed by a result row above, not asserted: zero CSP violations observed
(the cleanest of the three candidates against this app's real, strict CSP); the only one that
captured a live ECharts canvas correctly without a manual workaround; handled this app's actual
`color-mix()`-based theme CSS without crashing (unlike the unmaintained original `html2canvas`);
and at 10.1 KB gzip is 4.5-6x smaller than either `html2canvas` variant — a real, measured number,
not an estimate, and a meaningful one against the same 150 KB lazy-route budget M3-32/33 already
had to fight for.

**Required design consequence, not optional polish**: M3-39 (and the diff "copy as image" feature,
wherever it lands) must render a **dedicated, non-interactive export view** — composing the chart
components over a flat background, no `.glass`/`backdrop-filter` — the exact same strategy
`AnalyticsImageExporter.swift`/`DiffSnapshotView` already use, not a new workaround invented for
this port. This isn't a limitation to design around later; it directly resolves the one real
fidelity gap this spike found, by construction.

**For "copy as image" specifically**: use Tauri's own `@tauri-apps/plugin-clipboard-manager`
`writeImage` (native, added v2.1.0-beta+) rather than the web `navigator.clipboard.write()` API —
the latter's cross-engine image-write behaviour is genuinely unconfirmed by this spike (not tested
on any engine), while the native plugin bypasses the question entirely by never touching the
browser Clipboard API at all.

## What this spike leaves open, named rather than hidden

- **WebKitGTK and WKWebView are untested** — every result above is Windows/WebView2 only. Research
  (not reproduced) found a real, version-gated WebKitGTK bug limiting `canvas.toDataURL()` to PNG
  on older builds — irrelevant here since PNG is already the target format, but a reminder this
  engine is the highest-risk target for anything canvas-encoding-related and deserves a real check
  on real Linux hardware before shipping M3-39.
- **`navigator.clipboard.write(image/png)` cross-engine behaviour** was not tested on any engine —
  resolved above by routing around it via the native Tauri plugin instead of settling the question.
- **ECharts' SVG renderer's real bundle-size cost** was not measured (the chosen approach doesn't
  need it, but if `dom-to-image-more`'s canvas-capture support is ever found to regress on another
  engine, switching the export view's charts to SVG rendering is the documented fallback worth
  revisiting then, not now).
- **Why these specific CSP violations fire** (the `style-src` ones, `html2canvas-pro`'s `base-uri`
  one) was not root-caused line-by-line inside each library's own source — confirmed non-fatal by
  direct visual inspection of the output, which is what this task's own AC asks for, but a library
  update could change that; the recommendation above already avoids the two libraries that showed
  the worse version of this (the `html2canvas` family) specifically because of it.
