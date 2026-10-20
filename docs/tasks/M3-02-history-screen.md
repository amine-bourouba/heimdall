# M3-02 · Prompt history screen

> Milestone M3 · Workstream: Feature screens · Scope: MVP

## Goal

The history screen lists CLI prompt history grouped into date buckets, with pasted contents, search and copy.

## Inputs

- `$SWIFT_REF/Poirot/Sources/Views/History/HistoryListView.swift`
- `$SWIFT_REF/Poirot/Sources/Models/HistoryEntry.swift:40-84`
- `$SWIFT_REF/PoirotTests/Models/HistoryEntryTests.swift` — date-bucket and relative-time cases
- `docs/spec/test-carryover.md`

## Outputs

- `src/features/history/HistoryPage.vue` — modify placeholder
- `src/features/history/HistoryRow.vue` — create
- `src/features/history/store.ts` — create
- `src/features/history/store.spec.ts` — create
- `src/features/history/index.ts` — modify: available

## Acceptance criteria

- [ ] Buckets Today, Yesterday, This Week, Last Week, This Month and Older match HistoryEntry.swift (spec with a fixed clock).
- [ ] Paging uses `list_history`; copy puts the entry's display text on the clipboard.
- [ ] Delete actions are not present yet (M3-51).

## Dependencies

- M1-43 — History read command
- M2-09 — App shell: split view, sidebar with live counts, toolbar, overlays
- M2-10 — renderMarkdown, Shiki highlighting and the single v-html component
- M0-06 — Map every PoirotTests case to its Heimdall destination
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
