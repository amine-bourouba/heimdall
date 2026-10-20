# M3-19 · Keyboard navigation end-to-end tests

> Milestone M3 · Workstream: Testing · Scope: MVP

## Goal

End-to-end tests prove every A.7 shortcut and the layered Escape work in the running UI with visible focus.

## Inputs

- `docs/parity-matrix.md` — section J
- `$SWIFT_REF/Poirot/Sources/Views/Components/KeyboardNavigationModifier.swift`

## Outputs

- `e2e/keyboard/navigation.spec.ts` — create

## Acceptance criteria

- [ ] Covers Mod+K, arrows and Return in search, Mod+[ and Mod+], Mod+T, font shortcuts, layered Esc, form ⌘./Return equivalents, onboarding ← and →.

## Dependencies

- M2-20 — Virtualised conversation view
- M2-23 — Playwright and real-app E2E harness
- M3-01 — Universal search overlay
- M3-15 — Onboarding flow

## Owning agent

test-automator

## Reviewer agent

qa-expert, accessibility-tester

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
