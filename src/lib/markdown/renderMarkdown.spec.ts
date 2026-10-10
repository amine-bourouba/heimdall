// @vitest-environment jsdom
//
// DOMPurify's own README is explicit: "tools like happy-dom exist but are not considered safe
// at this point. Combining DOMPurify with happy-dom is currently not recommended and will likely
// lead to XSS." Confirmed for real, not just taken on faith: under this repo's default happy-dom
// environment, `DOMPurify.sanitize('<script>alert(1)</script>safe')` returned `"alert(1)safe"`
// -- even a totally disallowed tag's own *content* survived, which is not how DOMPurify behaves
// under a real DOM at all. The exact same call under jsdom correctly returns `"safe"`. Production
// is unaffected either way -- the real runtime is a genuine WebView (WKWebView/WebView2/
// WebKitGTK), not happy-dom -- but this spec file needs a real DOM to mean anything, so it opts
// out of the project default here the same way `contrast.spec.ts` (M2-02) opts into plain Node.
import DOMPurify from 'dompurify';
import { describe, expect, it } from 'vitest';
import { renderMarkdown } from './renderMarkdown';
import { sanitizeConfig } from './sanitizeConfig';

describe('renderMarkdown: html: false (markdown-it never parses raw HTML)', () => {
  it('a literal <script> tag in the source renders as escaped text, not an executable tag', async () => {
    const html = await renderMarkdown('before <script>alert(1)</script> after', false);
    expect(html).not.toContain('<script>');
    expect(html).toContain('&lt;script&gt;');
  });

  it('a literal onclick-bearing tag in the source is escaped, never parsed as an element', async () => {
    const html = await renderMarkdown('<div onclick="alert(1)">x</div>', false);
    expect(html).not.toMatch(/<div[^>]*onclick/);
  });
});

describe('sanitizeConfig: the real DOMPurify allowlist (ADR 0010 point 1)', () => {
  function sanitize(dirty: string): string {
    return DOMPurify.sanitize(dirty, sanitizeConfig);
  }

  it('strips a tag outside ALLOWED_TAGS entirely, keeping only its text', () => {
    expect(sanitize('<script>alert(1)</script>safe')).toBe('safe');
    expect(sanitize('<iframe src="https://evil.example"></iframe>safe')).toBe('safe');
  });

  it('strips style and every on* attribute from an otherwise-allowed tag', () => {
    const out = sanitize('<a href="https://example.com" style="color:red" onclick="alert(1)" onmouseover="alert(2)">link</a>');
    expect(out).not.toContain('style=');
    expect(out).not.toContain('onclick');
    expect(out).not.toContain('onmouseover');
    expect(out).toContain('href="https://example.com"');
  });

  it('allows http:, https:, mailto: and # links; rejects javascript:, data: and file:', () => {
    expect(sanitize('<a href="https://example.com">x</a>')).toContain('href="https://example.com"');
    expect(sanitize('<a href="http://example.com">x</a>')).toContain('href="http://example.com"');
    expect(sanitize('<a href="mailto:a@example.com">x</a>')).toContain('href="mailto:a@example.com"');
    expect(sanitize('<a href="#section">x</a>')).toContain('href="#section"');

    expect(sanitize('<a href="javascript:alert(1)">x</a>')).not.toContain('href=');
    expect(sanitize('<a href="data:text/html,<script>alert(1)</script>">x</a>')).not.toContain('href=');
    expect(sanitize('<a href="file:///etc/passwd">x</a>')).not.toContain('href=');
  });

  it('never allows an <img> tag at all', () => {
    expect(sanitize('<img src="https://example.com/x.png" onerror="alert(1)">')).toBe('');
  });
});

describe('renderMarkdown: images render as alt text, never an <img>', () => {
  it('a markdown image becomes its alt text with no <img> anywhere in the output', async () => {
    const html = await renderMarkdown('![a cat photo](https://example.com/cat.png)', false);
    expect(html).not.toContain('<img');
    expect(html).toContain('a cat photo');
  });

  it('an image whose src is a dangerous scheme still produces no <img>, just the alt text', async () => {
    const html = await renderMarkdown('![x](javascript:alert(1))', false);
    expect(html).not.toContain('<img');
    expect(html).toContain('x');
  });
});

describe('renderMarkdown: links', () => {
  it('a markdown link keeps its real href, ready for MarkdownView to intercept', async () => {
    const html = await renderMarkdown('[docs](https://example.com/docs)', false);
    expect(html).toContain('<a href="https://example.com/docs">docs</a>');
  });
});

describe('renderMarkdown: Shiki highlighting', () => {
  it('highlights a fenced code block with real per-token colour classes, no inline style= at all', async () => {
    const html = await renderMarkdown('```javascript\nconst x = 1;\n```', false);
    expect(html).toContain('<pre');
    expect(html).toContain('class="shiki');
    expect(html).not.toContain('style=');
  });

  it('re-highlighting the same code for the opposite theme produces different colours', async () => {
    const light = await renderMarkdown('```javascript\nconst x = 1;\n```', false);
    const dark = await renderMarkdown('```javascript\nconst x = 1;\n```', true);
    expect(light).toContain('github-light');
    expect(dark).toContain('github-dark');
    expect(light).not.toBe(dark);
  });

  it('an unfenced/plain code block still renders (no language, Shiki\'s own "text" pseudo-language)', async () => {
    const html = await renderMarkdown('```\nplain text\n```', false);
    expect(html).toContain('plain text');
    expect(html).not.toContain('style=');
  });

  it('an unrecognised language alias falls back to plain escaped text rather than failing the render', async () => {
    const html = await renderMarkdown('```not-a-real-language\n<tag>\n```', false);
    expect(html).toContain('&lt;tag&gt;');
  });
});
