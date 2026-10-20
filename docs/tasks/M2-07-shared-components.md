# M2-07 · Shared components

> Milestone M2 · Workstream: Vue shell and design system · Scope: MVP

## Goal

The shared components exist as accessible Vue components with specs: MarqueeText, GaugeRing, ShimmerBlock, ToastOverlay, InfoTooltipButton, CardChrome, GlassPanel and FilterField.

## Inputs

- `$SWIFT_REF/Poirot/Sources/Views/Components/MarqueeText.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Components/UsageGaugeRing.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Components/ShimmerModifier.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Components/ToastOverlay.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Components/InfoTooltipButton.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Components/CardChrome.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Components/GlassBackground.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Config/ConfigFilterField.swift`
- `$SWIFT_REF/PoirotTests/Models/UsageWindowTests.swift` — fraction and severity cases for the gauge ring
- `docs/spec/test-carryover.md`

## Outputs

- `src/components/{MarqueeText,GaugeRing,ShimmerBlock,ToastOverlay,InfoTooltipButton,CardChrome,GlassPanel,FilterField}.vue` — create
- `src/components/{MarqueeText,GaugeRing,ToastOverlay,InfoTooltipButton,FilterField}.spec.ts` — create

## Acceptance criteria

- [ ] MarqueeText scrolls on hover only and not under reduced motion; GaugeRing is a deterministic SVG (snapshot spec).
- [ ] InfoTooltipButton opens on click, closes on Esc and returns focus; FilterField clears on its button and on Esc.
- [ ] Every interactive component has a visible focus ring and an accessible name.

## Dependencies

- M2-04 — Lucide icon component, motion utilities and the SF Symbol map
- M2-05 — Prefs, theme, toasts and platform stores
- M0-06 — Map every PoirotTests case to its Heimdall destination

## Owning agent

vue-expert

## Reviewer agent

architect-reviewer, accessibility-tester

## Size

L

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
