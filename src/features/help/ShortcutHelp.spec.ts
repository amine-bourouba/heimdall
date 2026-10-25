// M3-17's own AC: a dismissable sheet toggled by the 'shortcuts' AppAction, listing every
// shortcut straight from useShortcuts' own BINDINGS array (port-plan.md P-7) so it can never drift
// from what Mod-key actually dispatches.
//
// Teleport-to-body root: queries go through `document.body` (`body()`), not `wrapper.find()` --
// the same reasoning SearchOverlay.spec.ts's own header comment already documents.
import { DOMWrapper, mount } from '@vue/test-utils';
import type { VueWrapper } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { usePlatformStore } from '@/stores/platform';
import { useAppActions } from '@/composables/useAppActions';
import { BINDINGS } from '@/composables/useShortcuts';
import ShortcutHelp from './ShortcutHelp.vue';

function body() {
  return new DOMWrapper(document.body);
}

let mounted: VueWrapper[] = [];

function mountSheet() {
  setActivePinia(createPinia());
  usePlatformStore().os = 'windows';
  const wrapper = mount(ShortcutHelp, { attachTo: document.body });
  mounted.push(wrapper);
  return wrapper;
}

beforeEach(() => {
  document.body.innerHTML = '';
});

afterEach(() => {
  mounted.forEach((w) => w.unmount());
  mounted = [];
});

describe('ShortcutHelp', () => {
  it('is hidden until the "shortcuts" action dispatches, and toggles on a second dispatch', async () => {
    mountSheet();
    const { dispatch } = useAppActions();
    expect(body().find('[role="dialog"]').exists()).toBe(false);

    dispatch('shortcuts');
    await mounted[0].vm.$nextTick();
    expect(body().find('[role="dialog"]').exists()).toBe(true);

    dispatch('shortcuts');
    await mounted[0].vm.$nextTick();
    expect(body().find('[role="dialog"]').exists()).toBe(false);
  });

  it('lists every BINDINGS entry, not a hand-copied subset', async () => {
    mountSheet();
    useAppActions().dispatch('shortcuts');
    await mounted[0].vm.$nextTick();

    const rows = body().findAll('[role="dialog"] kbd');
    expect(rows).toHaveLength(BINDINGS.length);
    expect(body().text()).toContain('Universal Search');
    expect(body().text()).toContain('Ctrl+k');
  });

  it('Escape and the Close button both close it', async () => {
    mountSheet();
    const { dispatch } = useAppActions();

    dispatch('shortcuts');
    await mounted[0].vm.$nextTick();
    await body().find('[role="dialog"]').trigger('keydown', { key: 'Escape' });
    expect(body().find('[role="dialog"]').exists()).toBe(false);

    dispatch('shortcuts');
    await mounted[0].vm.$nextTick();
    await body().find('button[title="Close"]').trigger('click');
    expect(body().find('[role="dialog"]').exists()).toBe(false);
  });
});
