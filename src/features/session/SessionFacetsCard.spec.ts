// M3-06's own AC: "Outcome and helpfulness labels match SessionFacets.swift; the card hides when
// no facets exist." Ported from SessionFacets.swift:56-90's own switch statements.
import { flushPromises, mount } from '@vue/test-utils';
import { mockIPC } from '@tauri-apps/api/mocks';
import { createPinia, setActivePinia } from 'pinia';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import SessionFacetsCard from './SessionFacetsCard.vue';

type Facets = {
  sessionId: string;
  underlyingGoal: string;
  goalCategories: Record<string, number>;
  outcome: string;
  userSatisfactionCounts: Record<string, number>;
  claudeHelpfulness: string;
  sessionType: string;
  frictionCounts: Record<string, number>;
  frictionDetail: string;
  primarySuccess: string;
  briefSummary: string;
};

function makeFacets(overrides: Partial<Facets> = {}): Facets {
  return {
    sessionId: 's1',
    underlyingGoal: '',
    goalCategories: {},
    outcome: 'success',
    userSatisfactionCounts: {},
    claudeHelpfulness: 'very_helpful',
    sessionType: 'feature_development',
    frictionCounts: {},
    frictionDetail: '',
    primarySuccess: '',
    briefSummary: '',
    ...overrides,
  };
}

let mounted: ReturnType<typeof mount>[] = [];

async function mountCard(facets: Facets | null) {
  mockIPC((cmd, args) => {
    if (cmd === 'get_session_facets') {
      expect(args).toEqual({ sessionId: 's1' });
      return facets;
    }
    throw new Error(`unexpected command: ${cmd}`);
  });
  const wrapper = mount(SessionFacetsCard, { props: { sessionId: 's1' }, attachTo: document.body });
  mounted.push(wrapper);
  await flushPromises();
  return wrapper;
}

beforeEach(() => {
  document.body.innerHTML = '';
  setActivePinia(createPinia()); // AppIcon reads usePrefsStore() for its motion-effect gating.
});

afterEach(() => {
  mounted.forEach((w) => w.unmount());
  mounted = [];
});

describe('SessionFacetsCard: hides when no facets exist (AC)', () => {
  it('renders nothing when get_session_facets returns null', async () => {
    const wrapper = await mountCard(null);
    expect(wrapper.html()).toBe('<!--v-if-->');
  });

  it('renders the card when facets exist', async () => {
    const wrapper = await mountCard(makeFacets());
    expect(wrapper.text()).toContain('AI Summary');
  });
});

describe('SessionFacetsCard: outcome labels match SessionFacets.swift:56-64 (AC)', () => {
  it.each([
    ['success', 'Success'],
    ['partially_achieved', 'Partial'],
    ['unclear_from_transcript', 'Unclear'],
    ['failure', 'Failed'],
    ['some_new_value', 'Some New Value'], // the Swift switch's own default: "_" -> " ", capitalized
  ])('outcome "%s" renders as "%s"', async (outcome, label) => {
    const wrapper = await mountCard(makeFacets({ outcome }));
    expect(wrapper.text()).toContain(label);
  });
});

describe('SessionFacetsCard: helpfulness labels match SessionFacets.swift:67-75 (AC)', () => {
  it.each([
    ['very_helpful', 'Very Helpful'],
    ['slightly_helpful', 'Slightly Helpful'],
    ['not_helpful', 'Not Helpful'],
    ['harmful', 'Harmful'],
    ['unknown_value', 'Unknown Value'],
  ])('claudeHelpfulness "%s" renders as "%s"', async (helpfulness, label) => {
    const wrapper = await mountCard(makeFacets({ claudeHelpfulness: helpfulness }));
    expect(wrapper.text()).toContain(label);
  });
});

describe('SessionFacetsCard: expanded content (SessionFacetsCard.swift:71-100)', () => {
  it('shows the goal section only when underlyingGoal is non-empty', async () => {
    const withGoal = await mountCard(makeFacets({ underlyingGoal: 'Ship the feature' }));
    expect(withGoal.text()).toContain('Ship the feature');

    const withoutGoal = await mountCard(makeFacets({ underlyingGoal: '' }));
    expect(withoutGoal.text()).not.toContain('Goal');
  });

  it('sorts goal categories by count descending and humanizes snake_case names', async () => {
    const wrapper = await mountCard(makeFacets({ goalCategories: { bug_fix: 1, feature_development: 3 } }));
    const text = wrapper.text();
    expect(text.indexOf('Feature Development')).toBeLessThan(text.indexOf('Bug Fix'));
  });

  it('shows the friction section only when totalFrictionCount > 0', async () => {
    const withFriction = await mountCard(makeFacets({ frictionCounts: { confusing_error: 2 } }));
    expect(withFriction.text()).toContain('Friction');
    expect(withFriction.text()).toContain('Confusing Error');

    const withoutFriction = await mountCard(makeFacets({ frictionCounts: {} }));
    expect(withoutFriction.text()).not.toContain('Friction');
  });

  it('renders briefSummary through MarkdownView when non-empty', async () => {
    const wrapper = await mountCard(makeFacets({ briefSummary: '# Heading' }));
    await flushPromises();
    expect(wrapper.find('.markdown-view').exists()).toBe(true);
  });

  it('collapsing the header hides the expanded content', async () => {
    const wrapper = await mountCard(makeFacets({ underlyingGoal: 'Ship it' }));
    expect(wrapper.text()).toContain('Ship it');

    await wrapper.find('button').trigger('click');

    expect(wrapper.text()).not.toContain('Ship it');
  });
});
