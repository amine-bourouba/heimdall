// Ported from ConfigItemDetailView.swift:4-126.
import { flushPromises, mount } from '@vue/test-utils';
import { mockIPC } from '@tauri-apps/api/mocks';
import { createPinia, setActivePinia } from 'pinia';
import { vi, beforeEach, describe, expect, it } from 'vitest';
import type { ConfigRef } from '@/ipc/bindings';
import { usePlatformStore } from '@/stores/platform';
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

  it('forwards the toolbar\'s close event (M3-12 necessary ripple)', async () => {
    const onClose = vi.fn();
    const wrapper = mount(ConfigItemDetail, { props: { icon: 'apple.terminal.fill', title: '/deploy', scope: 'global', body: 'x', onClose } });

    await wrapper.find('button[title="Close"]').trigger('click');

    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('offers Delete only when deletable, and forwards the toolbar\'s delete event (M3-50)', async () => {
    const onDelete = vi.fn();
    const plain = mount(ConfigItemDetail, { props: { icon: 'apple.terminal.fill', title: '/deploy', scope: 'global', body: 'x' } });
    expect(plain.find('button[title="Delete"]').exists()).toBe(false);

    const wrapper = mount(ConfigItemDetail, { props: { icon: 'apple.terminal.fill', title: '/deploy', scope: 'global', body: 'x', deletable: true, onDelete } });
    await wrapper.find('button[title="Delete"]').trigger('click');

    expect(onDelete).toHaveBeenCalledTimes(1);
  });

  // M3-55: the page says which item this is, and the toolbar turns the editor and file-manager buttons on.
  it('passes the item\'s reference to the toolbar, which then offers the editor and file-manager buttons', async () => {
    const configRef: ConfigRef = { kind: 'skill', slug: 'lint', project: null };
    const launched: [string, unknown][] = [];
    usePlatformStore().os = 'macos';
    mockIPC((cmd, args) => {
      launched.push([cmd, args]);
      return null;
    });
    const wrapper = mount(ConfigItemDetail, { props: { icon: 'bolt.fill', title: 'lint', scope: 'global', body: 'x', configRef } });

    await wrapper.find('button[title="Open in VS Code"]').trigger('click');
    await wrapper.find('button[title="Show in Finder"]').trigger('click');
    await flushPromises();

    expect(launched).toEqual([
      ['open_in_editor', { target: { kind: 'configItem', configRef } }],
      ['reveal_in_file_manager', { target: { kind: 'configItem', configRef } }],
    ]);
  });

  it('offers neither without one', () => {
    const wrapper = mount(ConfigItemDetail, { props: { icon: 'bolt.fill', title: 'lint', scope: 'global', body: 'x' } });
    expect(wrapper.find('button[title^="Open in"]').exists()).toBe(false);
    expect(wrapper.find('button[title^="Show in"]').exists()).toBe(false);
  });
});
