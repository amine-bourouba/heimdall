import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { shouldMarqueeScroll } from './marqueeLogic';
import MarqueeText from './MarqueeText.vue';

beforeEach(() => {
  setActivePinia(createPinia());
  vi.spyOn(window, 'matchMedia').mockReturnValue({
    matches: false,
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
  } as unknown as MediaQueryList);
});

// AC1: "scrolls on hover only and not under reduced motion." happy-dom has no real layout
// (scrollWidth/clientWidth are always 0), so the decision this AC actually describes is tested
// as the pure function MarqueeText.vue delegates to -- a mount-level test could never observe a
// real overflow value here.
describe('shouldMarqueeScroll (MarqueeText.vue)', () => {
  it('scrolls only when hovered, overflowing, and neither motion gate is tripped', () => {
    expect(
      shouldMarqueeScroll({ isHovered: true, overflow: 40, prefersReducedMotion: false, showAnimations: true }),
    ).toBe(true);
  });

  it('does not scroll when not hovered', () => {
    expect(
      shouldMarqueeScroll({ isHovered: false, overflow: 40, prefersReducedMotion: false, showAnimations: true }),
    ).toBe(false);
  });

  it('does not scroll when the text does not overflow', () => {
    expect(
      shouldMarqueeScroll({ isHovered: true, overflow: 0, prefersReducedMotion: false, showAnimations: true }),
    ).toBe(false);
  });

  it('does not scroll under reduced motion, even while hovered and overflowing', () => {
    expect(
      shouldMarqueeScroll({ isHovered: true, overflow: 40, prefersReducedMotion: true, showAnimations: true }),
    ).toBe(false);
  });

  it('does not scroll when showAnimations is false', () => {
    expect(
      shouldMarqueeScroll({ isHovered: true, overflow: 40, prefersReducedMotion: false, showAnimations: false }),
    ).toBe(false);
  });
});

describe('MarqueeText.vue', () => {
  it('renders the given text', () => {
    const wrapper = mount(MarqueeText, { props: { text: 'a long session title' } });
    expect(wrapper.text()).toBe('a long session title');
  });

  it('never applies the scrolling class before any hover (no real layout in this environment, so overflow starts at 0)', () => {
    const wrapper = mount(MarqueeText, { props: { text: 'short' } });
    expect(wrapper.find('span').classes()).not.toContain('marquee-scrolling');
  });
});
