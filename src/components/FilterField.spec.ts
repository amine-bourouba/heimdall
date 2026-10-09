import { defineComponent, h, ref } from 'vue';
import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it } from 'vitest';
import FilterField from './FilterField.vue';

beforeEach(() => {
  setActivePinia(createPinia());
});

// A real two-way-bound host, not wrapper.emitted(): @vue/test-utils' emit tracking depends on
// Vue.setDevtoolsHook, which only exists in the esm-browser build, not the bundler/cjs build
// Vite/Vitest actually resolve here -- confirmed with the simplest possible repro (a bare
// `emit('x')` component never shows up in `wrapper.emitted()` in this project). A host that
// genuinely implements v-model is unaffected by that gap and is the more realistic test anyway.
function mountWithHost(initial: string) {
  const Host = defineComponent({
    setup() {
      const value = ref(initial);
      return () => h(FilterField, { modelValue: value.value, 'onUpdate:modelValue': (v: string) => (value.value = v) });
    },
  });
  return mount(Host, { attachTo: document.body });
}

describe('FilterField.vue', () => {
  it('has no clear button when empty', () => {
    const wrapper = mount(FilterField, { props: { modelValue: '' } });
    expect(wrapper.find('button').exists()).toBe(false);
  });

  it('shows a clear button once there is a value', () => {
    const wrapper = mount(FilterField, { props: { modelValue: 'auth' } });
    expect(wrapper.find('button').exists()).toBe(true);
  });

  it('clears on its own button and refocuses the input', async () => {
    const wrapper = mountWithHost('auth');

    await wrapper.find('button').trigger('click');

    expect(wrapper.find('input').element.value).toBe('');
    expect(wrapper.find('button').exists()).toBe(false); // the clear button itself disappears
    expect(document.activeElement).toBe(wrapper.find('input').element);
  });

  it('clears on Escape', async () => {
    const wrapper = mountWithHost('auth');

    await wrapper.find('input').trigger('keydown', { key: 'Escape' });

    expect(wrapper.find('input').element.value).toBe('');
  });

  it('does nothing on Escape when already empty', async () => {
    const wrapper = mountWithHost('');

    await wrapper.find('input').trigger('keydown', { key: 'Escape' });

    expect(wrapper.find('input').element.value).toBe('');
    expect(wrapper.find('button').exists()).toBe(false);
  });

  it('has an accessible name from its placeholder', () => {
    const wrapper = mount(FilterField, { props: { modelValue: '', placeholder: 'Search commands…' } });
    expect(wrapper.find('input').attributes('aria-label')).toBe('Search commands…');
  });
});
