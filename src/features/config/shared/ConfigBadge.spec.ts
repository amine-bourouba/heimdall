import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it } from 'vitest';
import ConfigBadge from './ConfigBadge.vue';

beforeEach(() => {
  setActivePinia(createPinia());
});

describe('ConfigBadge: plain pill mode', () => {
  it('renders the given text with default classes when no scope is given', () => {
    const wrapper = mount(ConfigBadge, { props: { text: '3 tools' } });
    expect(wrapper.text()).toBe('3 tools');
  });

  it('applies caller-supplied fg/bg classes', () => {
    const wrapper = mount(ConfigBadge, { props: { text: 'sonnet', fgClass: 'text-(--accent)', bgClass: 'bg-(--accent-dim)' } });
    expect(wrapper.classes()).toContain('text-(--accent)');
    expect(wrapper.classes()).toContain('bg-(--accent-dim)');
  });
});

describe('ConfigBadge: scope mode (ConfigScopeBadge.swift)', () => {
  it.each([
    ['global', 'Global'],
    ['project', 'Project'],
    ['builtIn', 'Built-in'],
  ] as const)('scope "%s" renders as "%s"', (scope, label) => {
    const wrapper = mount(ConfigBadge, { props: { scope } });
    expect(wrapper.text()).toContain(label);
  });
});
