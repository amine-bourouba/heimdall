// ConfigSkeletonView.swift:3-100: "Grid skeleton: two columns x 3 shimmering cards; list
// skeleton: 4 shimmering cards."
import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it } from 'vitest';
import ConfigSkeleton from './ConfigSkeleton.vue';

beforeEach(() => {
  setActivePinia(createPinia()); // ShimmerBlock reads usePrefsStore().showAnimations.
});

describe('ConfigSkeleton', () => {
  it('renders 6 cards (2x3) for the grid layout', () => {
    const wrapper = mount(ConfigSkeleton, { props: { layout: 'grid' } });
    expect(wrapper.findAll('[class*="rounded-md border"]')).toHaveLength(6);
  });

  it('renders 4 cards for the list layout', () => {
    const wrapper = mount(ConfigSkeleton, { props: { layout: 'list' } });
    expect(wrapper.findAll('[class*="rounded-md border"]')).toHaveLength(4);
  });
});
