// Shiki syntax highlighting (ADR 0010 point 1; PoirotMarkdownTheme.swift's `PoirotCodeBlockView`,
// which re-highlights on light/dark change). `codeToHtml` from the top-level `shiki` package
// lazily loads only the grammar/theme bundles a call actually references and caches them
// internally -- no manual highlighter-instance management needed to satisfy "loads only the
// languages a document references".
//
// `transformerStyleToClass` converts an inline `style` Shiki emits into a shared, hashed class
// name -- but only for the root `<pre>` element (its own `pre()` hook) and for per-token colors
// when the *dual-theme* `themes: {light,dark}` API populates `token.htmlStyle` (confirmed by
// reading `@shikijs/core`'s real source: a single-theme call like this one's never sets
// `htmlStyle` at all, so `transformerStyleToClass`'s own `tokens()` hook -- which only acts when
// `token.htmlStyle` is already truthy -- silently does nothing, leaving every token's `color` as
// a raw inline `style` attribute that `sanitizeConfig.ts`'s allowlist would then strip entirely,
// turning every code block monochrome). `fillTokenHtmlStyle` below closes that gap by copying
// each token's real `color`/`bgColor` into `htmlStyle` itself, confirmed for real with a plain
// Node script before relying on it here -- the combined pipeline (`[fillTokenHtmlStyle,
// classTransformer]`) produces zero `style=` attributes anywhere in the output.
//
// That CSS is injected through a constructable stylesheet (`CSSStyleSheet` + `adoptedStyleSheets`)
// rather than a `<style>` tag, so the production CSP needs no `style-src 'unsafe-inline'`
// (port-plan P-16) -- a dynamically-inserted `<style>` tag's text content would otherwise be
// blocked by `default-src 'self'` the same way an inline `<script>` would be.
//
// Shiki ships no "xcode" theme (checked directly: `Object.keys((await import('shiki/themes')).bundledThemes)`
// has no match) -- Poirot's own HighlightSwift "xcode" theme has no real Shiki equivalent, so
// `github-light`/`github-dark` are used instead as the closest real substitute, same "confirmed
// gap, picked the closest real alternative" precedent as M2-04's SF Symbol mapping.

import { codeToHtml, getTokenStyleObject } from 'shiki';
import { transformerStyleToClass } from '@shikijs/transformers';

/** Populates `token.htmlStyle` from the token's real `color`/`bgColor`/`fontStyle` fields --
 * using Shiki's own real `getTokenStyleObject` (exported from `@shikijs/core`, re-exported by
 * `shiki` itself) rather than a hand-rolled copy, so this stays correct if Shiki ever adds more
 * style fields to a token. Runs before `classTransformer` in the pipeline below so that
 * transformer's own `tokens()` hook (which only converts an *already-present* `htmlStyle`) has
 * something to act on for a plain single-theme call. */
const fillTokenHtmlStyle = {
  name: 'heimdall:fill-token-html-style',
  tokens(lines: { color?: string; bgColor?: string; fontStyle?: number; htmlStyle?: Record<string, string> }[][]) {
    for (const line of lines) {
      for (const token of line) {
        const style = getTokenStyleObject(token);
        if (Object.keys(style).length > 0) token.htmlStyle = style;
      }
    }
  },
};

const classTransformer = transformerStyleToClass();
let sheet: CSSStyleSheet | undefined;

function syncStylesheet() {
  if (!sheet) {
    // Set directly rather than spreading the existing `document.adoptedStyleSheets` (this is the
    // only place in the app that ever touches it, so there is nothing else to preserve) --
    // jsdom's own `adoptedStyleSheets` getter is not iterable before anything has been assigned
    // to it yet, confirmed for real (`TypeError: document.adoptedStyleSheets is not iterable`),
    // unlike happy-dom's, which starts as a plain empty array.
    sheet = new CSSStyleSheet();
    document.adoptedStyleSheets = [sheet];
  }
  sheet.replaceSync(classTransformer.getCSS());
}

function escapeHtml(text: string): string {
  return text.replace(/[&<>"']/g, (ch) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[ch]!);
}

/** Highlights one fenced code block. Falls back to plain escaped text for a language Shiki
 * doesn't recognise (it throws on an unknown alias) rather than failing the whole render. */
export async function highlightCode(code: string, lang: string | undefined, isDark: boolean): Promise<string> {
  try {
    const html = await codeToHtml(code, {
      lang: lang || 'text',
      theme: isDark ? 'github-dark' : 'github-light',
      transformers: [fillTokenHtmlStyle, classTransformer],
    });
    syncStylesheet();
    return html;
  } catch {
    return `<pre><code>${escapeHtml(code)}</code></pre>`;
  }
}
