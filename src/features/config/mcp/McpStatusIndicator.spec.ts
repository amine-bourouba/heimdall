// Ported from MCPServerStatusIndicator.swift:1-75.
import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it } from 'vitest';
import type { McpServerStatus } from '@/ipc/bindings';
import McpStatusIndicator from './McpStatusIndicator.vue';

beforeEach(() => {
  setActivePinia(createPinia()); // AppIcon reads usePrefsStore() unconditionally.
});

describe('McpStatusIndicator: per-status icon, colour and accessibility label', () => {
  it.each([
    ['connected', 'lucide-circle', 'text-(--color-green)', 'Server is connected and running'],
    ['needsAuth', 'lucide-triangle-alert', 'text-(--color-orange)', 'Server requires authentication'],
    ['failed', 'lucide-circle-x', 'text-(--color-red)', 'Server failed to start'],
    ['unreachable', 'lucide-circle-slash', 'text-(--color-red)', 'Server is unreachable'],
    ['starting', 'lucide-circle', 'text-(--color-blue)', 'Server is starting up'],
    ['unknown', 'lucide-circle-question-mark', 'text-(--text-tertiary)', 'Server status is unknown'],
  ] as const)('status "%s" renders %s with %s and the right aria-label', (status, iconClass, colorClass, a11y) => {
    const wrapper = mount(McpStatusIndicator, { props: { status: status as McpServerStatus } });
    expect(wrapper.html()).toContain(iconClass);
    expect(wrapper.classes()).toContain(colorClass);
    expect(wrapper.attributes('aria-label')).toBe(a11y);
  });

  it('only "starting" gets the pulse motion effect', () => {
    const starting = mount(McpStatusIndicator, { props: { status: 'starting' } });
    expect(starting.classes()).toContain('motion-pulse');

    const connected = mount(McpStatusIndicator, { props: { status: 'connected' } });
    expect(connected.classes()).not.toContain('motion-pulse');
  });
});
