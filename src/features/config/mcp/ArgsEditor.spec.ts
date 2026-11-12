// M3-48 AC: "Custom arguments are an ordered list editor, never split on spaces."
import { defineComponent, h, ref } from 'vue';
import { flushPromises, mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import ArgsEditor from './ArgsEditor.vue';

let mounted: ReturnType<typeof mount>[] = [];
let value = ref<string[]>([]);

// A real two-way-bound host, not wrapper.emitted(): emit tracking never fires in this project
// (see ConfigLayoutToolbar.spec.ts).
function mountEditor(initial: string[]) {
  value = ref([...initial]);
  const Host = defineComponent({
    setup() {
      return () => h(ArgsEditor, { modelValue: value.value, 'onUpdate:modelValue': (v: string[]) => (value.value = v) });
    },
  });
  const wrapper = mount(Host, { attachTo: document.body });
  mounted.push(wrapper);
  return wrapper;
}

const inputs = (w: ReturnType<typeof mount>) => w.findAll('input');

beforeEach(() => {
  setActivePinia(createPinia()); // AppIcon reads usePrefsStore().
  document.body.innerHTML = '';
});

afterEach(() => {
  mounted.forEach((w) => w.unmount());
  mounted = [];
});

describe('ArgsEditor: one input per argument', () => {
  it('shows each argument in its own numbered input, in order', () => {
    const wrapper = mountEditor(['-y', '@scope/server', '--flag']);
    expect(inputs(wrapper).map((i) => (i.element as HTMLInputElement).value)).toEqual(['-y', '@scope/server', '--flag']);
    expect(inputs(wrapper).map((i) => i.attributes('aria-label'))).toEqual(['Argument 1', 'Argument 2', 'Argument 3']);
  });

  it('says so when there are none', () => {
    expect(mountEditor([]).text()).toContain('No arguments.');
  });

  it('keeps an argument with spaces, quotes or JSON as a single entry, exactly as typed', async () => {
    const wrapper = mountEditor(['']);
    await inputs(wrapper)[0].setValue('--config /Users/a b/my config.json "quoted"');
    await flushPromises();
    expect(value.value).toEqual(['--config /Users/a b/my config.json "quoted"']);
  });

  it('changes only the argument that was typed in', async () => {
    const wrapper = mountEditor(['a', 'b', 'c']);
    await inputs(wrapper)[1].setValue('B');
    expect(value.value).toEqual(['a', 'B', 'c']);
  });
});

describe('ArgsEditor: add, remove, reorder', () => {
  it('adds an empty argument at the end and focuses it', async () => {
    const wrapper = mountEditor(['a']);
    await wrapper.find('[data-add]').trigger('click');
    await flushPromises();
    expect(value.value).toEqual(['a', '']);
    expect(document.activeElement).toBe(inputs(wrapper)[1].element);
  });

  it('removes the chosen argument and keeps the keyboard in the list', async () => {
    const wrapper = mountEditor(['a', 'b', 'c']);
    await wrapper.find('button[aria-label="Remove argument 2"]').trigger('click');
    await flushPromises();
    expect(value.value).toEqual(['a', 'c']);
    expect(document.activeElement).toBe(inputs(wrapper)[1].element);
  });

  it('moves an argument up and down, and the keyboard follows it', async () => {
    const wrapper = mountEditor(['a', 'b', 'c']);
    await wrapper.find('button[aria-label="Move argument 3 up"]').trigger('click');
    await flushPromises();
    expect(value.value).toEqual(['a', 'c', 'b']);
    expect(document.activeElement).toBe(wrapper.find('button[aria-label="Move argument 2 up"]').element);

    await wrapper.find('button[aria-label="Move argument 1 down"]').trigger('click');
    await flushPromises();
    expect(value.value).toEqual(['c', 'a', 'b']);
    expect(document.activeElement).toBe(wrapper.find('button[aria-label="Move argument 2 down"]').element);
  });

  it('cannot move the first argument up or the last down', () => {
    const wrapper = mountEditor(['a', 'b']);
    expect(wrapper.find('button[aria-label="Move argument 1 up"]').attributes('disabled')).toBeDefined();
    expect(wrapper.find('button[aria-label="Move argument 2 down"]').attributes('disabled')).toBeDefined();
    expect(wrapper.find('button[aria-label="Move argument 1 down"]').attributes('disabled')).toBeUndefined();
  });

  it('is a labelled group', () => {
    expect(mountEditor(['a']).find('[role="group"]').attributes('aria-label')).toBe('Arguments');
  });
});
