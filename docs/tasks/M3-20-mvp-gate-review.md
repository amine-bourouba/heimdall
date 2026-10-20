# M3-20 · MVP feature gate review

> Milestone M3 · Workstream: Testing · Scope: MVP

## Goal

Every MVP row of the parity matrix has a per-OS status with evidence, and every non-parity row carries exactly one outcome (degrade, hide or replace).

## Inputs

- `docs/port-plan.md` — §7 M3 gate
- `docs/parity-matrix.md`

## Outputs

- `docs/parity-matrix.md` — modify: MVP rows

## Acceptance criteria

- [ ] Evidence links (CI runs, screenshots, manual notes) are recorded per row and OS in the progress log.

## Dependencies

- M2-30 — M2 exit gate review
- M3-01 — Universal search overlay
- M3-02 — Prompt history screen
- M3-03 — Plans list and detail
- M3-04 — Memory list and detail
- M3-05 — Cross-project TODOs overview
- M3-06 — Session todos and AI summary card
- M3-07 — Per-session file history viewer
- M3-08 — Debug log viewer
- M3-10 — Export options dialog and save-as-patch
- M3-12 — Commands, skills and output styles screens (read-only)
- M3-13 — Hooks, plugins and models screens (read-only)
- M3-14 — MCP servers and sub-agents screens (read-only)
- M3-15 — Onboarding flow
- M3-16 — Settings screen
- M3-17 — Help, About and the full shortcut sheet
- M3-18 — Visual baselines for the remaining MVP screens
- M3-19 — Keyboard navigation end-to-end tests

## Owning agent

test-automator

## Reviewer agent

qa-expert, architect-reviewer

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
