// Ported from ConfigItemDetailView.swift:4-126.
import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { vi, beforeEach, describe, expect, it } from 'vitest';
import ConfigItemDetail from './ConfigItemDetail.vue';

vi.mock('@tauri-apps/plugin-clipboard-manager', () => ({ writeText: vi.fn().mockResolvedValue(undefined) }));

beforeEach(() => {
  setActivePinia(createPinia());
});

describe('ConfigItemDetail: body (ConfigItemDetailView.swift:93-124)', () => {
  it('shows "No content" when the body is empty', () => {
    const wrapper = mount(ConfigItemDetail, { props: { icon: 'apple.terminal.fill', title: '/deploy', scope: 'global', body: '' } });
    expect(wrapper.text()).toContain('No content');
  });

  it('renders the body through MarkdownView by default (formatted)', async () => {
    const wrapper = mount(ConfigItemDetail, { props: { icon: 'apple.terminal.fill', title: '/deploy', scope: 'global', body: '# Heading' } });
    await wrapper.vm.$nextTick();
    expect(wrapper.find('.markdown-view').exists()).toBe(true);
  });

  it('toggling the format button switches to raw <pre> text', async () => {
    const wrapper = mount(ConfigItemDetail, { props: { icon: 'apple.terminal.fill', title: '/deploy', scope: 'global', body: '# Heading' } });
    await wrapper.vm.$nextTick();

    await wrapper.find('button[title="Show raw content"]').trigger('click');

    expect(wrapper.find('.markdown-view').exists()).toBe(false);
    expect(wrapper.find('pre').text()).toBe('# Heading');
  });
});

describe('ConfigItemDetail: header', () => {
  it('shows the title and a scope badge', () => {
    const wrapper = mount(ConfigItemDetail, { props: { icon: 'apple.terminal.fill', title: '/deploy', scope: 'project', body: 'x' } });
    expect(wrapper.text()).toContain('/deploy');
    expect(wrapper.text()).toContain('Project');
  });

  it('renders the caller-supplied badges slot', () => {
    const wrapper = mount(ConfigItemDetail, {
      props: { icon: 'apple.terminal.fill', title: '/deploy', scope: 'global', body: 'x' },
      slots: { badges: '<span class="model-badge">Sonnet</span>' },
    });
    expect(wrapper.find('.model-badge').text()).toBe('Sonnet');
  });
});
