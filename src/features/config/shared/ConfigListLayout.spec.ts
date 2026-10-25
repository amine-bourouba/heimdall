import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it } from 'vitest';
import ConfigListLayout from './ConfigListLayout.vue';

beforeEach(() => {
  setActivePinia(createPinia());
});

const emptyProps = { emptyIcon: 'apple.terminal.fill', emptyMessage: 'No commands found', emptyHint: '~/.claude/commands/' };

describe('ConfigListLayout: states', () => {
  it('shows the skeleton while loading, regardless of items', () => {
    const wrapper = mount(ConfigListLayout, { props: { layout: 'grid', isLoading: true, items: ['a'], ...emptyProps } });
    expect(wrapper.text()).not.toContain('No commands found');
    expect(wrapper.findAll('[class*="rounded-md border"]')).toHaveLength(6);
  });

  it('shows the caller-supplied empty state when items is empty and loading is done', () => {
    const wrapper = mount(ConfigListLayout, { props: { layout: 'grid', isLoading: false, items: [], ...emptyProps } });
    expect(wrapper.text()).toContain('No commands found');
  });

  it('renders one slot instance per item, passing the item through', () => {
    const wrapper = mount(ConfigListLayout, {
      props: { layout: 'list', isLoading: false, items: ['first', 'second', 'third'], ...emptyProps },
      slots: { default: '<p class="row">{{ params.item }}</p>' },
    });
    const rows = wrapper.findAll('.row');
    expect(rows.map((r) => r.text())).toEqual(['first', 'second', 'third']);
  });
});

describe('ConfigListLayout: grid vs list classes', () => {
  it('uses a 2-column grid for "grid"', () => {
    const wrapper = mount(ConfigListLayout, { props: { layout: 'grid', isLoading: false, items: ['a'], ...emptyProps }, slots: { default: '<div />' } });
    expect(wrapper.find('[class*="grid-cols-2"]').exists()).toBe(true);
  });

  it('uses a flex column for "list"', () => {
    const wrapper = mount(ConfigListLayout, { props: { layout: 'list', isLoading: false, items: ['a'], ...emptyProps }, slots: { default: '<div />' } });
    expect(wrapper.find('[class*="flex-col"]').exists()).toBe(true);
  });
});
