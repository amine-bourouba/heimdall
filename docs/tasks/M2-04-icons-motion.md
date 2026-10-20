# M2-04 · Lucide icon component, motion utilities and the SF Symbol map

> Milestone M2 · Workstream: Vue shell and design system · Scope: MVP

## Goal

One icon component renders Lucide icons with named motion effects that switch off under reduced motion or `showAnimations = false`, and a map covers every SF Symbol the Swift reference app uses.

## Inputs

- `$SWIFT_REF/Poirot/Sources/` — all `systemName:` literals and model `icon` properties
- `$SWIFT_REF/Poirot/Sources/Views/Components/ShimmerModifier.swift`
- `AI_INSTRUCTIONS.md` — Icons

## Outputs

- `src/components/AppIcon.vue` — create
- `src/components/iconMap.ts` — create
- `src/composables/usePrefersReducedMotion.ts` — create
- `src/composables/usePrefersReducedMotion.spec.ts` — create
- `src/styles/motion.css` — modify
- `docs/design/icon-map.md` — create

## Acceptance criteria

- [ ] icon-map.md maps each of the 61 distinct `systemName:` literals at a29dd6e plus every model-provided symbol name (HookEvent.icon, SubAgent icons, MCP catalog icons, NavigationItem icons) to a Lucide name, and lists gaps.
- [ ] motion.css defines `bounce`, `pulse`, `breathe`, `variable-color`, `rotate`, `wiggle` and `replace` utilities, each disabled under `prefers-reduced-motion: reduce`.
- [ ] Spec: AppIcon applies no animation class when reduced motion is on or `showAnimations` is false.

## Dependencies

- M0-01 — Write Heimdall's AI_INSTRUCTIONS.md and CLAUDE.md
- M2-02 — Theme tokens for 20 palettes and the Tailwind v4 theme
- M2-05 — Prefs, theme, toasts and platform stores

## Owning agent

vue-expert

## Reviewer agent

architect-reviewer, accessibility-tester

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
