// @vitest-environment jsdom
//
// Same reasoning as src/lib/markdown/renderMarkdown.spec.ts: DOMPurify's own README says it isn't
// safe under happy-dom (the project's default environment), confirmed here too -- this file
// opts into a real DOM the same way.
import { describe, expect, it } from 'vitest';
import { renderMarkdown } from '@/lib/markdown/renderMarkdown';
import payloads from '../../fixtures/security/xss-payloads.json';

// The allowlisted tags/attrs/schemes from sanitizeConfig.ts, restated here independently rather
// than imported: this corpus is proving the *sanitized output's own shape* against ADR 0010's
// plain-language acceptance criteria, not re-deriving the same constant the implementation
// already uses (which would let a change to sanitizeConfig.ts silently change what this corpus
// considers "safe").
const DANGEROUS_TAGS = ['script', 'iframe', 'img', 'meta', 'object', 'embed', 'svg', 'style', 'link', 'base', 'form'];
const ALLOWED_URI = /^(?:https?:|mailto:|#)/i;

function assertSafeDom(html: string) {
  const doc = new DOMParser().parseFromString(`<!doctype html><body>${html}</body>`, 'text/html');
  const all = doc.body.querySelectorAll('*');

  for (const el of all) {
    const tag = el.tagName.toLowerCase();
    expect(DANGEROUS_TAGS, `found disallowed tag <${tag}> in: ${html}`).not.toContain(tag);

    for (const attr of Array.from(el.attributes)) {
      const name = attr.name.toLowerCase();
      expect(name.startsWith('on'), `found event-handler attribute "${name}" in: ${html}`).toBe(false);
      expect(name, `found a style attribute in: ${html}`).not.toBe('style');

      if (name === 'href' || name === 'src') {
        expect(attr.value, `found disallowed URI "${attr.value}" in: ${html}`).toMatch(ALLOWED_URI);
      }
    }
  }
}

describe('XSS corpus through renderMarkdown (ADR 0010 point 1, M2-25)', () => {
  it('the fixture corpus is non-empty and covers every category ADR 0010 lists', () => {
    const categories = new Set(payloads.map((p) => p.category));
    expect(payloads.length).toBeGreaterThan(0);
    expect(categories).toEqual(
      new Set([
        'script',
        'event-handler',
        'disallowed-uri',
        'iframe',
        'remote-image',
        'style-attribute',
        'meta-refresh',
        'link-smuggling',
        'entity-obfuscation',
        'fenced-code',
      ]),
    );
  });

  for (const payload of payloads) {
    it(`${payload.id}: ${payload.description}`, async () => {
      const html = await renderMarkdown(payload.markdown, false);
      assertSafeDom(html);
    });
  }

  it('the remote-image payload leaves no loadable <img> and no otherwise-retained src= anywhere', async () => {
    const payload = payloads.find((p) => p.id === 'remote-image');
    expect(payload).toBeDefined();
    const html = await renderMarkdown(payload!.markdown, false);
    expect(html).not.toContain('<img');
    expect(html).not.toContain('src=');
  });

  it('the fenced-code payloads render their HTML as visible, escaped text, not parsed markup', async () => {
    const payload = payloads.find((p) => p.id === 'html-in-fenced-code');
    expect(payload).toBeDefined();
    const html = await renderMarkdown(payload!.markdown, false);
    // Shiki tokenizes the escaped markup across several <span>s (so `&lt;script&gt;` never
    // survives as one contiguous substring) -- what actually matters is that the angle bracket
    // was escaped at all, and assertSafeDom below confirms no real <script> element exists.
    expect(html).toContain('&lt;');
    assertSafeDom(html);
  });
});
