import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it } from 'vitest';
import { useToastsStore } from '@/stores/toasts';
import ToastOverlay from './ToastOverlay.vue';

beforeEach(() => {
  setActivePinia(createPinia());
});

describe('ToastOverlay.vue', () => {
  it('renders nothing when the queue is empty', () => {
    const wrapper = mount(ToastOverlay);
    expect(wrapper.find('button').exists()).toBe(false);
  });

  it("renders the current toast's message", () => {
    useToastsStore().show('saved successfully');
    const wrapper = mount(ToastOverlay);
    expect(wrapper.text()).toContain('saved successfully');
  });

  it('clicking the toast calls clickCurrent (dismisses, opens its URL if any)', async () => {
    const store = useToastsStore();
    store.show('click me');
    const wrapper = mount(ToastOverlay);

    await wrapper.find('button').trigger('click');

    expect(store.current).toBeNull();
  });

  it('announces politely for assistive tech (aria-live) rather than interrupting', () => {
    const wrapper = mount(ToastOverlay);
    expect(wrapper.attributes('aria-live')).toBe('polite');
  });
});
