// A real bug, found by the user actually clicking a menu item (not caught by any existing test):
// `@focusout="isOpen = false"` closed the menu -- unmounting the very button being clicked --
// before its own `@click` could fire, since native `focusout` fires on an element whenever it
// loses focus and bubbles regardless of where focus actually goes. Every item appeared to do
// nothing but close the menu. These tests pin the fix (`relatedTarget` must be outside the menu
// for a focusout to actually close it) so this can't silently regress again.
import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { describe, expect, it, afterEach, beforeEach } from 'vitest';
import type { AppAction } from '@/ipc/bindings';
import { useAppActions } from '@/composables/useAppActions';
import AppMenu from './AppMenu.vue';

let mounted: ReturnType<typeof mount>[] = [];
let unregisterFns: (() => void)[] = [];

beforeEach(() => {
  setActivePinia(createPinia());
});

afterEach(() => {
  mounted.forEach((w) => w.unmount());
  mounted = [];
  unregisterFns.forEach((off) => off());
  unregisterFns = [];
});

function mountMenu() {
  const wrapper = mount(AppMenu, { attachTo: document.body });
  mounted.push(wrapper);
  return wrapper;
}

function trackDispatch(action: AppAction) {
  const dispatched: AppAction[] = [];
  unregisterFns.push(useAppActions().on(action, () => dispatched.push(action)));
  return dispatched;
}

describe('AppMenu', () => {
  it('clicking a menu item dispatches its action and closes the menu', async () => {
    const dispatched = trackDispatch('about');
    const wrapper = mountMenu();

    await wrapper.find('button[aria-label="Menu"]').trigger('click');
    expect(wrapper.find('[role="menu"]').exists()).toBe(true);

    const item = wrapper.findAll('[role="menuitem"] button').find((b) => b.text().includes('About Heimdall'))!;
    await item.trigger('focusin');
    await item.trigger('click');

    expect(dispatched).toEqual(['about']);
    expect(wrapper.find('[role="menu"]').exists()).toBe(false);
  });

  it('a focusout that moves focus to another item inside the menu does not close it', async () => {
    const wrapper = mountMenu();
    await wrapper.find('button[aria-label="Menu"]').trigger('click');

    const items = wrapper.findAll('[role="menuitem"] button');
    const container = wrapper.find('.relative').element;
    container.dispatchEvent(new FocusEvent('focusout', { relatedTarget: items[1].element, bubbles: true }));
    await wrapper.vm.$nextTick();

    expect(wrapper.find('[role="menu"]').exists()).toBe(true);
  });

  it('a focusout that moves focus outside the menu closes it', async () => {
    const wrapper = mountMenu();
    await wrapper.find('button[aria-label="Menu"]').trigger('click');

    const outside = document.createElement('button');
    document.body.appendChild(outside);
    const container = wrapper.find('.relative').element;
    container.dispatchEvent(new FocusEvent('focusout', { relatedTarget: outside, bubbles: true }));
    await wrapper.vm.$nextTick();

    expect(wrapper.find('[role="menu"]').exists()).toBe(false);
    outside.remove();
  });
});
