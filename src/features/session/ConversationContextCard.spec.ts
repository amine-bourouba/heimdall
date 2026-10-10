// AC: "The sparkline is inline SVG; spec covers the empty-series case."
import { mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';
import type { ContextUsage } from '@/ipc/bindings';
import ConversationContextCard from './ConversationContextCard.vue';

function makeContext(overrides: Partial<ContextUsage> = {}): ContextUsage {
  return {
    current: 50000,
    peak: 80000,
    window: 200000,
    inputTokens: 10000,
    outputTokens: 5000,
    cacheReadTokens: 30000,
    cacheCreationTokens: 5000,
    series: [20000, 50000, 80000],
    ...overrides,
  };
}

describe('ConversationContextCard: sparkline', () => {
  it('renders the sparkline as an inline SVG polyline when the series has more than one point', () => {
    const wrapper = mount(ConversationContextCard, { props: { context: makeContext() } });
    const svg = wrapper.findAll('svg').find((s) => s.attributes('aria-label') === 'Context size per turn');
    expect(svg).toBeDefined();
    expect(svg!.element.tagName).toBe('svg');
    const polyline = svg!.find('polyline');
    expect(polyline.exists()).toBe(true);
    expect(polyline.attributes('points')).not.toBe('');
  });

  it('omits the sparkline entirely for the empty-series case', () => {
    const wrapper = mount(ConversationContextCard, { props: { context: makeContext({ series: [] }) } });
    expect(wrapper.findAll('svg').some((s) => s.attributes('aria-label') === 'Context size per turn')).toBe(false);
  });

  it('also omits the sparkline for a single-point series (no line to draw)', () => {
    const wrapper = mount(ConversationContextCard, { props: { context: makeContext({ series: [50000] }) } });
    expect(wrapper.findAll('svg').some((s) => s.attributes('aria-label') === 'Context size per turn')).toBe(false);
  });
});

describe('ConversationContextCard: derived figures', () => {
  it('shows the gauge ring, the current/window figures and the cache-hit percentage', () => {
    const wrapper = mount(ConversationContextCard, { props: { context: makeContext() } });
    expect(wrapper.text()).toContain('50.0K / 200.0K');
    // cacheReadTokens(30000) / (input 10000 + cacheRead 30000 + cacheCreation 5000) = 30000/45000 = 66.67% -> 67%
    expect(wrapper.text()).toContain('67% from cache');
    // current(50000) / window(200000) = 25%
    expect(wrapper.find('svg[aria-label="25% used"]').exists()).toBe(true);
  });

  it('the token-mix segments total the full width when nothing is zero', () => {
    const wrapper = mount(ConversationContextCard, { props: { context: makeContext() } });
    const widths = wrapper
      .findAll('div')
      .map((d) => d.attributes('style'))
      .filter((s): s is string => !!s && s.includes('width:'))
      .map((s) => parseFloat(s.match(/width:\s*([\d.]+)%/)?.[1] ?? '0'))
      .filter((w) => w > 0 && w <= 100);
    const total = widths.slice(0, 3).reduce((a, b) => a + b, 0);
    expect(total).toBeCloseTo(100, 0);
  });

  it('a zero-token session does not divide by zero (0% from cache, no NaN anywhere)', () => {
    const wrapper = mount(
      ConversationContextCard,
      { props: { context: makeContext({ inputTokens: 0, outputTokens: 0, cacheReadTokens: 0, cacheCreationTokens: 0, current: 0, series: [] }) } },
    );
    expect(wrapper.text()).toContain('0% from cache');
    expect(wrapper.text()).not.toContain('NaN');
  });
});
