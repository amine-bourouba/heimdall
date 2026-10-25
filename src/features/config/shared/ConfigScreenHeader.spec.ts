import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it } from 'vitest';
import ConfigScreenHeader from './ConfigScreenHeader.vue';

beforeEach(() => {
  setActivePinia(createPinia()); // AppIcon reads usePrefsStore().
});

describe('ConfigScreenHeader', () => {
  it('renders the title, count and description', () => {
    const wrapper = mount(ConfigScreenHeader, { props: { icon: 'apple.terminal.fill', title: 'Commands', count: 12, description: 'Slash commands available to Claude Code.' } });
    expect(wrapper.text()).toContain('Commands');
    expect(wrapper.text()).toContain('12');
    expect(wrapper.text()).toContain('Slash commands available to Claude Code.');
  });
});
