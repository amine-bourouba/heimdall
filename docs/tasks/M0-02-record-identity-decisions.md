# M0-02 · Record the final app identifier and Linux package name

> Milestone M0 · Workstream: Docs · Scope: Infra

## Goal

ADR 0011 moves to Accepted with the user's final app identifier and Linux package/binary name, and no `<APP_ID>` or `<LINUX_PKG>` placeholder remains in the ADRs or the plan.

## Inputs

- `docs/adr/0011-app-identifier-and-release-repo.md`
- `docs/port-plan.md` — section 2.2 follow-up decisions (D1, D2)
- The user's written answer to D1 and D2

## Outputs

- `docs/adr/0011-app-identifier-and-release-repo.md` — modify
- `docs/port-plan.md` — modify: decisions tables

## Acceptance criteria

- [ ] ADR 0011 status reads `Accepted` and names the identifier and the Linux package/binary name.
- [ ] The identifier is reverse-DNS, uses only `A-Z a-z 0-9 . -`, and does not end in `.app`.
- [ ] The Linux name is not an existing package in Debian/Ubuntu, Fedora, or Arch official repositories; the ADR records the three search URLs and the date checked.
- [ ] `grep -rn -E "<APP_ID>|<LINUX_PKG>" docs/adr docs/port-plan.md` returns nothing.

## Dependencies

- None

## Owning agent

technical-writer

## Reviewer agent

architect-reviewer

## Size

S

## Context

Done during planning on 2026-09-25, when the user accepted the recommended values; see the progress log. Write-up only: never choose values yourself.

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
