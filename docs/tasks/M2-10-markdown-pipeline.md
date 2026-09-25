# M2-10 · renderMarkdown, Shiki highlighting and the single v-html component

> Milestone M2 · Workstream: Security · Scope: MVP

## Goal

`renderMarkdown()` is the only route into `v-html`: markdown-it with HTML off, DOMPurify with explicit allowlists, class-based Shiki output, and links that open only through `open_external_url`.

## Inputs

- `docs/adr/0010-webview-trust-boundary.md` — Rendering, Links, CSP
- `$POIROT/Poirot/Sources/Theme/PoirotMarkdownTheme.swift:136-176`

## Outputs

- `src/lib/markdown/renderMarkdown.ts` — create
- `src/lib/markdown/highlight.ts` — create
- `src/lib/markdown/sanitizeConfig.ts` — create
- `src/lib/markdown/MarkdownView.vue` — create
- `src/lib/markdown/renderMarkdown.spec.ts` — create

## Acceptance criteria

- [ ] markdown-it runs with `html: false`; DOMPurify uses explicit ALLOWED_TAGS and ALLOWED_ATTR, allows only `http:`, `https:`, `mailto:` and `#` URIs, and strips `style` and `on*`.
- [ ] `<img>` never loads: images render as their alt text.
- [ ] Links are intercepted and routed to `open_external_url`; the webview never navigates.
- [ ] Shiki loads only the languages a document references, uses `transformerStyleToClass`, injects its CSS through a constructable stylesheet, and re-highlights on theme change.
- [ ] `grep -rn "v-html" src | grep -v src/lib/markdown/MarkdownView.vue` returns nothing.

## Dependencies

- M1-04 — Scaffold the Vite + Vue 3 + TypeScript + Tailwind v4 frontend
- M2-03 — Typed IPC client wrapper and test helpers
- M2-05 — Prefs, theme, toasts and platform stores

## Owning agent

security-engineer

## Reviewer agent

security-auditor

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
