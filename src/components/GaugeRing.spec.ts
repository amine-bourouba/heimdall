import { mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';
import GaugeRing from './GaugeRing.vue';

const CIRCUMFERENCE = 2 * Math.PI * 15;

// Ported from UsageWindowTests.swift's fraction/percent/severity cases (the gauge-ring-relevant
// ones only -- timeUntilReset has no gauge-ring consumer and is not ported here).
describe('GaugeRing.vue', () => {
  it('renders a deterministic SVG: same utilization, same markup, twice', () => {
    const a = mount(GaugeRing, { props: { utilization: 42 } }).html();
    const b = mount(GaugeRing, { props: { utilization: 42 } }).html();
    expect(a).toBe(b);
  });

  it('fraction and percent: 20% utilization', () => {
    const wrapper = mount(GaugeRing, { props: { utilization: 20 } });
    const fg = wrapper.findAll('circle')[1];
    expect(Number(fg.attributes('stroke-dashoffset'))).toBeCloseTo(CIRCUMFERENCE * 0.8, 5);
    expect(wrapper.text()).toBe('20%');
  });

  it('fraction clamps at 100% for utilization above 100', () => {
    const wrapper = mount(GaugeRing, { props: { utilization: 150 } });
    const fg = wrapper.findAll('circle')[1];
    expect(Number(fg.attributes('stroke-dashoffset'))).toBeCloseTo(0, 5);
  });

  it.each([
    [0, '--color-green'],
    [79.9, '--color-green'],
    [80, '--color-orange'],
    [94.9, '--color-orange'],
    [95, '--color-red'],
    [100, '--color-red'],
  ])('severity thresholds: %s%% utilization uses %s', (utilization, colorVar) => {
    const wrapper = mount(GaugeRing, { props: { utilization } });
    const fg = wrapper.findAll('circle')[1];
    expect(fg.attributes('stroke')).toBe(`var(${colorVar})`);
  });

  it('has an accessible name stating the percentage', () => {
    const wrapper = mount(GaugeRing, { props: { utilization: 63 } });
    expect(wrapper.find('svg').attributes('aria-label')).toBe('63% used');
  });
});
