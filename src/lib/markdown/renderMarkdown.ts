// The one real markdown->HTML pipeline (ADR 0010 point 1): markdown-it (no raw HTML) -> Shiki
// (syntax highlighting, async) -> DOMPurify (explicit allowlist). `MarkdownView.vue` is the only
// place the result reaches the raw-HTML template binding this task's own AC5 greps for.
//
// Shiki's highlighter is async, but markdown-it's own render pass is synchronous (a `highlight`
// option callback must return a string, not a Promise) -- so each fenced code block is rendered
// as a small placeholder `<pre data-shiki-id="N"></pre>` first (markdown-it's own fence renderer
// returns a `highlight()` result verbatim, un-wrapped, whenever it starts with "<pre" --
// confirmed directly in `markdown-it`'s own renderer source rather than assumed), then every
// placeholder is swapped for its real highlighted HTML once Shiki resolves, and only then is the
// whole string sanitized once.

import MarkdownIt from 'markdown-it';
import DOMPurify from 'dompurify';
import { highlightCode } from './highlight';
import { sanitizeConfig } from './sanitizeConfig';

interface PendingBlock {
  id: string;
  code: string;
  lang: string | undefined;
}

function buildMarkdownIt(blocks: PendingBlock[]) {
  const md = new MarkdownIt({ html: false, linkify: true });

  // Images never load (ADR 0010 point 1): render the alt text in place of an <img>. markdown-it
  // has already resolved the alt text into the image token's own `content` by this point.
  md.renderer.rules.image = (tokens, idx) => md.utils.escapeHtml(tokens[idx].content);

  md.set({
    highlight: (code, lang) => {
      const id = `shiki-${blocks.length}`;
      blocks.push({ id, code, lang: lang || undefined });
      return `<pre data-shiki-id="${id}"></pre>`;
    },
  });

  return md;
}

export async function renderMarkdown(source: string, isDark: boolean): Promise<string> {
  const blocks: PendingBlock[] = [];
  const html = buildMarkdownIt(blocks).render(source);

  const withHighlighting = (
    await Promise.all(blocks.map(async (block) => [block.id, await highlightCode(block.code, block.lang, isDark)] as const))
  ).reduce((acc, [id, highlighted]) => acc.replace(`<pre data-shiki-id="${id}"></pre>`, highlighted), html);

  return DOMPurify.sanitize(withHighlighting, sanitizeConfig);
}
