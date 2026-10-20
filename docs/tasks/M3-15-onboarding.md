# M3-15 · Onboarding flow

> Milestone M3 · Workstream: Feature screens · Scope: MVP

## Goal

A three-page onboarding modal appears until completed, cannot be dismissed early, detects the Claude CLI and existing sessions, and gives per-OS install hints.

## Inputs

- `$SWIFT_REF/Poirot/Sources/Views/Onboarding/OnboardingView.swift`

## Outputs

- `src/features/onboarding/Onboarding.vue` — modify placeholder
- `src/features/onboarding/Onboarding.spec.ts` — create

## Acceptance criteria

- [ ] ← and → move between pages; completion sets `hasCompletedOnboarding`.
- [ ] Detection uses `detect_claude_cli`.

## Dependencies

- M1-40 — App commands: app info, sidebar counts, CLI detection
- M2-08 — Keyboard navigation, global shortcuts and app actions
- M2-09 — App shell: split view, sidebar with live counts, toolbar, overlays
- M2-30 — M2 exit gate review

## Owning agent

frontend-developer (with typescript-pro for typed IPC)

## Reviewer agent

code-reviewer, accessibility-tester

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
