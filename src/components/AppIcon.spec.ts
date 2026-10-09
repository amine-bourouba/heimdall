import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { usePrefsStore } from '@/stores/prefs';
import AppIcon from './AppIcon.vue';

function mockMatchMedia(reduced: boolean) {
  vi.spyOn(window, 'matchMedia').mockReturnValue({
    matches: reduced,
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
  } as unknown as MediaQueryList);
}

beforeEach(() => {
  setActivePinia(createPinia());
});

describe('AppIcon: effect gating (AC3)', () => {
  it('applies the motion class when neither gate is tripped', () => {
    mockMatchMedia(false);
    const wrapper = mount(AppIcon, { props: { name: 'arrow.clockwise', effect: 'rotate' } });
    expect(wrapper.classes()).toContain('motion-rotate');
  });

  it('applies no animation class when the OS prefers reduced motion', () => {
    mockMatchMedia(true);
    const wrapper = mount(AppIcon, { props: { name: 'arrow.clockwise', effect: 'rotate' } });
    expect(wrapper.classes().some((c) => c.startsWith('motion-'))).toBe(false);
  });

  it('applies no animation class when showAnimations is false, even with no OS preference', () => {
    mockMatchMedia(false);
    usePrefsStore().prefs = { showAnimations: false } as ReturnType<typeof usePrefsStore>['prefs'] & object;
    const wrapper = mount(AppIcon, { props: { name: 'arrow.clockwise', effect: 'rotate' } });
    expect(wrapper.classes().some((c) => c.startsWith('motion-'))).toBe(false);
  });

  it('applies no class at all when no effect is requested', () => {
    mockMatchMedia(false);
    const wrapper = mount(AppIcon, { props: { name: 'arrow.clockwise' } });
    expect(wrapper.classes().some((c) => c.startsWith('motion-'))).toBe(false);
  });

  it('falls back to a visible marker icon for an unmapped name, instead of rendering nothing', () => {
    mockMatchMedia(false);
    const wrapper = mount(AppIcon, { props: { name: 'sf.symbol.not.in.the.map' } });
    expect(wrapper.find('svg').exists()).toBe(true);
  });

  it('renders the mapped icon for a known name', () => {
    mockMatchMedia(false);
    const wrapper = mount(AppIcon, { props: { name: 'trash' } });
    expect(wrapper.find('svg').exists()).toBe(true);
  });
});
