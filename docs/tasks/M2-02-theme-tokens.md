# M2-02 · Theme tokens for 20 palettes and the Tailwind v4 theme

> Milestone M2 · Workstream: Vue shell and design system · Scope: MVP

## Goal

Twenty palettes become CSS custom-property sets switched by `data-theme` on `<html>`, and typography, spacing, radius and icon sizes map one to one into Tailwind v4 theme keys scaled by `--font-scale`.

## Inputs

- `$POIROT/Poirot/Sources/Theme/PoirotTheme.swift`
- `$POIROT/Poirot/Sources/Views/Components/ThemePicker.swift`
- `$POIROT/Poirot/Sources/Views/Components/GlassBackground.swift`
- `docs/adr/0002-layer-boundaries.md`
- `$POIROT/PoirotTests/App/ColorThemeTests.swift` — palette cases, per docs/spec/test-carryover.md
- `docs/spec/test-carryover.md`

## Outputs

- `src/styles/tokens.css` — create
- `src/styles/main.css` — create
- `src/styles/motion.css` — create: placeholder owned by M2-04
- `src/theme/themes.ts` — create
- `src/theme/contrast.spec.ts` — create
- `docs/design/theme-tokens.md` — create
- `docs/design/webkitgtk-fallbacks.md` — create

## Acceptance criteria

- [ ] tokens.css has 20 `[data-theme=…]` blocks, each defining the 23 palette tokens plus `--accent-dim`; hex values match PoirotTheme.swift (docs/design/theme-tokens.md tabulates them).
- [ ] The adaptive themes (default, solarized, highContrast) switch light/dark with `prefers-color-scheme`; the 17 fixed themes set `color-scheme` from their `isDark` value (PoirotTheme.swift:280-287).
- [ ] Each theme defines explicit text-primary/secondary/tertiary colours (Poirot used native label colours, PoirotTheme.swift:344-349); contrast.spec.ts asserts WCAG AA 4.5:1 for primary and secondary text on bg-app and bg-card.
- [ ] main.css `@theme` exposes 24 text styles plus 4 code styles (PoirotTheme.swift:370-405), 8 spacing steps, 5 radii and 4 icon sizes; every font size is `calc(<px> * var(--font-scale))`.
- [ ] The glass utility uses `backdrop-filter` with an `@supports not` fallback to opaque bg-elevated; webkitgtk-fallbacks.md lists every lagging CSS feature and its fallback.

## Dependencies

- M1-04 — Scaffold the Vite + Vue 3 + TypeScript + Tailwind v4 frontend
- M0-06 — Map every PoirotTests case to its Heimdall destination

## Owning agent

ui-designer + vue-expert

## Reviewer agent

architect-reviewer, accessibility-tester

## Size

L

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
