# WebKitGTK fallbacks for lagging CSS features

Scope per `docs/port-plan.md` §9 (performance budgets) and risk R-2: "WebKitGTK fallbacks for
lagging CSS features (`backdrop-filter`, custom properties in `@theme`)". WebKitGTK (Linux's
Tauri webview, `libwebkit2gtk-4.1`) is the weakest of the three engines Heimdall ships on — it
tracks upstream WebKit, but distro packaging means the exact feature set on a given Linux
install can lag behind what Safari/WKWebView already ships (assumption A-11). Nothing here was
verified on a real Linux display server (none is available in this environment) — each item
says plainly what is confirmed by reading the spec/engine behaviour versus what still needs a
real check, and names the task that gets one (M2-24's Linux Playwright baselines, M4-12's
cross-OS platform verification).

## 1. `backdrop-filter` (the glass chrome, `GlassBackground.swift`)

**Risk:** WebKitGTK's `backdrop-filter` support has historically lagged Safari/WKWebView's — it
shipped disabled in some distro builds for a long stretch, and even where enabled, compositing a
blur behind a `color-mix()` background adds real GPU cost on the kind of lower-powered Linux
boxes this app also needs to run on.

**Fallback, already built into `src/styles/main.css`:**

```css
.glass {
  background-color: color-mix(in srgb, var(--bg-elevated) 72%, transparent);
  backdrop-filter: blur(20px);
}

@supports not (backdrop-filter: blur(1px)) {
  .glass {
    background-color: var(--bg-elevated);
    backdrop-filter: none;
  }
}
```

`@supports not (backdrop-filter: blur(1px))` is a real feature query, not a browser sniff — any
engine (including a WebKitGTK build that ships the property but disables it at compile time, a
real possibility `@supports` alone cannot distinguish from "doesn't exist at all") still gets
whichever branch its own CSS engine actually honours. The fallback is a fully opaque
`--bg-elevated`, never a half-transparent pane with no blur (which would be unreadable against
whatever is behind the window).

Confirmed by building the real pipeline (`npx vite build` with `.glass` actually referenced in a
template): Tailwind's own Lightning CSS step automatically adds the `-webkit-backdrop-filter`
prefix neither this task's source nor `@tailwindcss/vite`'s config asked for explicitly, and
folds it into the same `@supports` condition (`@supports not ((-webkit-backdrop-filter:blur(1px))
or (backdrop-filter:blur(1px)))`) — exactly the prefixed property an older WebKit-derived engine
would need, generated without this task having to hand-write it.

**Still needs a real check:** whether a current `libwebkit2gtk-4.1` on Ubuntu 22.04+ (the floor
set by assumption A-12) actually takes the `backdrop-filter` branch or the fallback — M2-24's
Linux Playwright baselines are the first real evidence either way, and should record which
branch rendered.

## 2. Custom properties referenced inside `@theme` (Tailwind v4's own generated utilities)

**Risk:** every font size in `main.css`'s `@theme` block is `calc(<px> * var(--font-scale))` —
a custom property read inside a `calc()` inside a Tailwind-generated utility class, not a
plain literal. CSS custom properties themselves are universally supported (WebKitGTK has had
them since long before Tailwind v4 existed), but `@theme`'s own build-time resolution is a
Tailwind/Lightning CSS concern, not a browser one: Tailwind resolves `--font-scale` into the
generated utility's `calc()` expression at *build* time, and the `var(--font-scale)` reference
that survives into the shipped CSS is resolved by the browser at *paint* time like any other
custom property. Confirmed by building the real pipeline (`npx vite build` with `text-hero-title`
actually referenced in a template): the compiled output keeps `var(--font-scale)` as a literal
runtime reference (`--text-hero-title:calc(32px * var(--font-scale))` in the theme layer,
`.text-hero-title{font-size:var(--text-hero-title)}` in utilities), not a value baked in at
build time — so this works
correctly as long as the browser supports `var()` inside `calc()` inside any CSS property value,
which every CSS engine still receiving updates has supported for years, WebKitGTK included.

**No fallback needed, but one thing to watch:** `--font-scale` is set once on `:root` (default
`1`) and is expected to be overridden higher up the tree (`:root` or `html`) once the real
`fontScale` preference wires in (M2-05's theme store). If a future change sets it per-component
instead of at the root, `calc()` re-evaluates correctly on every engine either way — CSS custom
property inheritance and recalculation is not an area where WebKitGTK is known to lag.

**Still needs a real check:** M2-28's performance budgets should confirm font-scale changes
re-paint at an acceptable cost on Linux specifically, since WebKitGTK's layout/paint pipeline is
the one most likely to be slower on the lower-powered hardware this risk is really about.

## 3. Items considered and deliberately not covered here

- **`color-mix()`** (used by `.glass` above): a CSS Color 5 function. WebKit added support
  before WebKitGTK 4.1 became the Linux floor this project targets (A-12); not treated as a
  separate risk since it is already inside the same `@supports` branch backdrop-filter uses —
  if the engine cannot composite a blur at all, the plain opaque fallback never evaluates
  `color-mix()` either.
- **`:has()`, `@container`, `text-wrap: balance`**: not used anywhere in this task's own output
  (`tokens.css`, `main.css`). Not speculatively guarded against here — add a fallback in
  whichever later task first reaches for one of these, with the same real-engine-check
  discipline this document follows, rather than guessing now at features nothing uses yet.
