# M3-57 · Separate frontend entry for the tray window

> Milestone M3 · Workstream: Vue shell and design system · Scope: Later

## Goal

The tray window loads its own small HTML entry so its bundle and capabilities stay separate from the main window.

## Inputs

- `vite.config.ts`
- `docs/adr/0010-webview-trust-boundary.md` — Capabilities

## Outputs

- `tray.html` — create
- `src/tray/main.ts` — create
- `vite.config.ts` — modify: multi-page inputs

## Acceptance criteria

- [ ] `npm run build` emits `dist/tray.html` with its own chunk.

## Dependencies

- M1-04 — Scaffold the Vite + Vue 3 + TypeScript + Tailwind v4 frontend
- M2-05 — Prefs, theme, toasts and platform stores

## Owning agent

vue-expert

## Reviewer agent

architect-reviewer

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
