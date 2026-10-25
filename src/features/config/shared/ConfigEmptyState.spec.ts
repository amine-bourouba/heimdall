import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it } from 'vitest';
import ConfigEmptyState from './ConfigEmptyState.vue';

beforeEach(() => {
  setActivePinia(createPinia());
});

describe('ConfigEmptyState', () => {
  it('renders the caller-supplied message and hint for the "no items" flavour', () => {
    const wrapper = mount(ConfigEmptyState, { props: { icon: 'apple.terminal.fill', message: 'No commands found', hint: '~/.claude/commands/' } });
    expect(wrapper.text()).toContain('No commands found');
    expect(wrapper.text()).toContain('~/.claude/commands/');
  });

  it('renders the caller-supplied message and hint for the "no filter matches" flavour', () => {
    const wrapper = mount(ConfigEmptyState, { props: { icon: 'magnifyingglass', message: 'No commands match "xyz"', hint: 'Try a different search term' } });
    expect(wrapper.text()).toContain('No commands match "xyz"');
    expect(wrapper.text()).toContain('Try a different search term');
  });
});
