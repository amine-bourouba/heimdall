// AC3: "The detail toolbar copies name and content; editor and reveal buttons come later
// (M3-55)." Close (M3-12's own necessary ripple, ConfigDetailToolbar.swift:136-148) is covered
// below too.
import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { vi, beforeEach, describe, expect, it, afterEach } from 'vitest';
import ConfigDetailToolbar from './ConfigDetailToolbar.vue';

vi.mock('@tauri-apps/plugin-clipboard-manager', () => ({ writeText: vi.fn().mockResolvedValue(undefined) }));

beforeEach(() => {
  setActivePinia(createPinia()); // AppIcon reads usePrefsStore() unconditionally.
});

afterEach(() => {
  vi.useRealTimers();
});

describe('ConfigDetailToolbar: copy name and content (AC3)', () => {
  it('copies the name and shows a checkmark for 1.5s', async () => {
    vi.useFakeTimers();
    const { writeText } = await import('@tauri-apps/plugin-clipboard-manager');
    const wrapper = mount(ConfigDetailToolbar, { props: { name: '/deploy', content: 'body text', formatted: true } });

    await wrapper.find('button[title="Copy Name"]').trigger('click');
    expect(writeText).toHaveBeenCalledWith('/deploy');
    expect(wrapper.html()).toContain('lucide-check');

    await vi.advanceTimersByTimeAsync(1500);
    expect(wrapper.html()).toContain('lucide-files');
  });

  it('copies the content', async () => {
    const { writeText } = await import('@tauri-apps/plugin-clipboard-manager');
    const wrapper = mount(ConfigDetailToolbar, { props: { name: '/deploy', content: 'body text', formatted: true } });

    await wrapper.find('button[title="Copy Content"]').trigger('click');
    expect(writeText).toHaveBeenCalledWith('body text');
  });
});

describe('ConfigDetailToolbar: format toggle (read-only, no IPC)', () => {
  // Not wrapper.emitted(): see ConfigLayoutToolbar.spec.ts's own header comment -- @vue/test-utils'
  // emit tracking never fires in this project. A real onUpdate:formatted listener is unaffected.
  it('calls onUpdate:formatted with the flipped value', async () => {
    const onUpdateFormatted = vi.fn();
    const wrapper = mount(ConfigDetailToolbar, { props: { name: 'n', content: 'c', formatted: true, 'onUpdate:formatted': onUpdateFormatted } });

    await wrapper.find('button[title="Show raw content"]').trigger('click');

    expect(onUpdateFormatted).toHaveBeenCalledWith(false);
  });
});

describe('ConfigDetailToolbar: close', () => {
  it('calls onClose when clicked', async () => {
    const onClose = vi.fn();
    const wrapper = mount(ConfigDetailToolbar, { props: { name: 'n', content: 'c', formatted: true, onClose } });

    await wrapper.find('button[title="Close"]').trigger('click');

    expect(onClose).toHaveBeenCalledTimes(1);
  });
});
