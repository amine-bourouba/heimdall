# M3-04 · Memory list and detail

> Milestone M3 · Workstream: Feature screens · Scope: MVP

## Goal

Memory files from project memory folders and user-added folders list by source with MEMORY.md first, open into rendered detail, and folders can be added or removed.

## Inputs

- `$POIROT/Poirot/Sources/Views/Memory/MemoryListView.swift`
- `$POIROT/Poirot/Sources/Views/Memory/MemoryDetailView.swift`
- `$POIROT/Poirot/Sources/App/MemorySourcesStore.swift`

## Outputs

- `src/features/memory/MemoryPage.vue` — modify placeholder
- `src/features/memory/MemoryDetail.vue` — create
- `src/features/memory/store.ts` — create
- `src/features/memory/store.spec.ts` — create
- `src/features/memory/index.ts` — modify: available

## Acceptance criteria

- [ ] Adding a folder calls `add_memory_folder` (Rust opens the picker); removing calls `remove_memory_folder`.
- [ ] The list refreshes on `data-changed` for the memory area.

## Dependencies

- M1-39 — Preferences storage and preference commands
- M1-44 — Config read commands
- M2-09 — App shell: split view, sidebar with live counts, toolbar, overlays
- M2-10 — renderMarkdown, Shiki highlighting and the single v-html component
- M2-30 — M2 exit gate review

## Owning agent

frontend-developer (with typescript-pro for typed IPC)

## Reviewer agent

code-reviewer, accessibility-tester

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
