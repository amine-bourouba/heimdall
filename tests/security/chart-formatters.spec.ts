// M3-35: every chart tooltip that echoes a data-derived string (ultimately a raw JSONL `model`
// field, M3-33's own threat model) must neutralise the full XSS corpus, not just the 2
// hand-picked payloads formatters.spec.ts already proved the mechanism against.
//
// Unlike xss-corpus.spec.ts (renderMarkdown, which parses markdown/HTML and must selectively
// allow some tags/links), `escapeForTooltip` never parses anything -- it's a plain string -> HTML
// entity escape applied to a value that gets concatenated into a tooltip's HTML string by hand.
// So the categories that only become dangerous once markdown *link syntax* is parsed
// (disallowed-uri, link-smuggling, entity-obfuscation) are inert here regardless of escaping --
// they're tested anyway, for the same reason xss-corpus.spec.ts runs every category: a corpus
// entry that trivially passes is still a real regression guard if the escaping is ever removed.
import { describe, expect, it } from 'vitest';
import { escapeForTooltip } from '@/features/analytics/charts/formatters';
import payloads from '../../fixtures/security/xss-payloads.json';

describe('chart tooltip formatters neutralise the XSS corpus when payloads arrive as model names (ADR 0010, M3-35)', () => {
  it('the fixture corpus is non-empty', () => {
    expect(payloads.length).toBeGreaterThan(0);
  });

  for (const payload of payloads) {
    it(`${payload.id}: ${payload.description}`, () => {
      const escaped = escapeForTooltip(payload.markdown);
      expect(escaped, `raw "<"/">" survived escaping for payload: ${payload.markdown}`).not.toMatch(/[<>]/);
    });
  }

  it('a payload containing a literal <script> tag no longer contains that substring after escaping', () => {
    const payload = payloads.find((p) => p.id === 'script-tag');
    expect(payload).toBeDefined();
    const escaped = escapeForTooltip(payload!.markdown);
    expect(escaped).not.toContain('<script>');
    expect(escaped).toContain('&lt;script&gt;');
  });

  it('an ordinary, non-malicious model name is returned unchanged', () => {
    expect(escapeForTooltip('Opus 4.6')).toBe('Opus 4.6');
    expect(escapeForTooltip('claude-sonnet-4-5-20260101')).toBe('claude-sonnet-4-5-20260101');
  });
});
