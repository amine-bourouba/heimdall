import { describe, expect, it } from 'vitest';
import type { ConfigItemDetail } from '@/ipc/bindings';
import { agentFromDetail, agentSlug, duplicateOf, KNOWN_TOOLS, newAgentModel, predictSlug, TOOL_CATEGORIES, toInput } from './agentModel';

// The slug cases below mirror `agent_slug` / `unique_slug` in crates/heimdall-core/src/config/write/
// files.rs and its tests ("Code Reviewer", "Old Name", `-2` on a clash): the form previews the file
// name a save will write, so these two must not drift apart.
describe('agentSlug', () => {
  it.each([
    ['Code Reviewer', 'code-reviewer'],
    ['Old Name', 'old-name'],
    ['  Padded  ', 'padded'],
    ['My_Agent v2!', 'myagent-v2'],
    ['Ünïcode Ägent', 'ünïcode-ägent'],
    ['日本語 エージェント', '日本語-エージェント'],
    ['!!!', ''],
    ['   ', ''],
  ])('%j -> %j', (name, slug) => {
    expect(agentSlug(name)).toBe(slug);
  });
});

describe('predictSlug (unique_slug: a clash never overwrites another agent)', () => {
  it('is the plain slug when nothing clashes', () => {
    expect(predictSlug('Reviewer', null, ['other'])).toBe('reviewer');
  });

  it('adds -2, -3, ... past existing files', () => {
    expect(predictSlug('Reviewer', null, ['reviewer'])).toBe('reviewer-2');
    expect(predictSlug('Reviewer', null, ['reviewer', 'reviewer-2'])).toBe('reviewer-3');
  });

  it('an edit that keeps its own slug overwrites in place, not as reviewer-2', () => {
    expect(predictSlug('Reviewer', 'reviewer', ['reviewer'])).toBe('reviewer');
  });

  it('an edit renamed onto ANOTHER existing agent gets a suffix rather than replacing it', () => {
    expect(predictSlug('Keep This One', 'will-be-renamed', ['keep-this-one', 'will-be-renamed'])).toBe('keep-this-one-2');
  });

  it('an edit renamed to a free slug just takes it', () => {
    expect(predictSlug('New Name', 'old-name', ['old-name'])).toBe('new-name');
  });

  it('is empty for a name with no letters or numbers (the writer would reject it)', () => {
    expect(predictSlug('!!!', null, [])).toBe('');
  });
});

describe('tool catalogue', () => {
  it('has every category tool in the known list, each in exactly one category', () => {
    const categorised = TOOL_CATEGORIES.flatMap((c) => c.tools);
    expect([...categorised].sort()).toEqual([...KNOWN_TOOLS].sort());
    expect(new Set(categorised).size).toBe(categorised.length);
  });
});

function detail(overrides: Partial<ConfigItemDetail> = {}): ConfigItemDetail {
  return {
    ref: { kind: 'agent', project: null, slug: 'reviewer' },
    name: 'Reviewer',
    description: 'Reviews code',
    scope: 'global',
    meta: { tools: 'Read, Grep', model: 'sonnet', color: 'Blue', memory: 'user' },
    body: 'Be helpful.',
    displayPath: '~/.claude/agents/reviewer.md',
    ...overrides,
  };
}

describe('agentFromDetail', () => {
  it('maps the summary meta (Debug-named colour, wire-value memory, comma-joined tools) and the body', () => {
    expect(agentFromDetail(detail())).toEqual({
      slug: 'reviewer',
      name: 'Reviewer',
      description: 'Reviews code',
      tools: ['Read', 'Grep'],
      model: 'sonnet',
      color: 'blue',
      memory: 'user',
      prompt: 'Be helpful.',
    });
  });

  it('keeps an unset model, colour and memory as null instead of inventing Swift\'s defaults', () => {
    const model = agentFromDetail(detail({ meta: {} }));
    expect([model.model, model.color, model.memory, model.tools]).toEqual([null, null, null, []]);
  });

  it('treats a colour or memory outside the editor\'s enums as unset', () => {
    const model = agentFromDetail(detail({ meta: { color: 'Yellow', memory: 'project' } }));
    expect([model.color, model.memory]).toEqual([null, null]);
  });

  it('keeps a model the editor has no button for, verbatim', () => {
    expect(agentFromDetail(detail({ meta: { model: 'claude-opus-4-1' } })).model).toBe('claude-opus-4-1');
  });
});

describe('toInput / duplicateOf', () => {
  it('a new agent starts with Swift\'s defaults (the only place they apply)', () => {
    const m = newAgentModel();
    expect([m.tools, m.model, m.color, m.memory]).toEqual([['Glob', 'Grep', 'Read', 'Bash'], 'sonnet', 'orange', 'user']);
  });

  it('trims name and description, fixes the icon, and sends an empty prompt as null', () => {
    const input = toInput({ ...newAgentModel(), name: '  A  ', description: ' d ', prompt: '' });
    expect(input).toMatchObject({ name: 'A', description: 'd', prompt: null, icon: 'person.crop.circle.badge.plus' });
  });

  it('passes a non-empty prompt through untrimmed (Swift: `prompt.isEmpty ? nil : prompt`)', () => {
    expect(toInput({ ...newAgentModel(), prompt: '\nline one\n\nline two\n' }).prompt).toBe('\nline one\n\nline two\n');
  });

  it('duplicates as "<name> Copy" with everything else carried over and no original slug', () => {
    const source = agentFromDetail(detail());
    expect(duplicateOf(source)).toEqual({
      name: 'Reviewer Copy',
      icon: 'person.crop.circle.badge.plus',
      description: 'Reviews code',
      tools: ['Read', 'Grep'],
      model: 'sonnet',
      color: 'blue',
      prompt: 'Be helpful.',
      memory: 'user',
    });
  });
});
