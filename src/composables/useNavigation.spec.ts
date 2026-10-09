import { defineComponent, h } from 'vue';
import { flushPromises, mount } from '@vue/test-utils';
import { createRouter, createWebHashHistory } from 'vue-router';
import { beforeEach, describe, expect, it } from 'vitest';
import { useNavigation } from './useNavigation';

const Blank = defineComponent({ render: () => h('div') });

function makeRouter() {
  return createRouter({
    history: createWebHashHistory(),
    routes: [
      { path: '/', component: Blank },
      { path: '/a/:id?', name: 'a', component: Blank },
      { path: '/b', name: 'b', component: Blank },
    ],
  });
}

async function mountWith(router: ReturnType<typeof makeRouter>) {
  await router.push('/');
  const Host = defineComponent({
    setup() {
      return useNavigation();
    },
    render() {
      return h('div');
    },
  });
  const wrapper = mount(Host, { global: { plugins: [router] } });
  return wrapper.vm as unknown as ReturnType<typeof useNavigation>;
}

beforeEach(() => {
  // Each test gets its own router/history instance, not a shared window.history position.
  window.history.replaceState(null, '', '#/');
});

describe('useNavigation', () => {
  it('starts with nowhere to go, in either direction', async () => {
    const nav = await mountWith(makeRouter());
    expect(nav.canGoBack).toBe(false);
    expect(nav.canGoForward).toBe(false);
  });

  it('canGoBack becomes true after a push, canGoForward stays false', async () => {
    const router = makeRouter();
    const nav = await mountWith(router);

    await router.push('/a/1');

    expect(nav.canGoBack).toBe(true);
    expect(nav.canGoForward).toBe(false);
  });

  it('goBack navigates back and flips canGoForward on', async () => {
    const router = makeRouter();
    const nav = await mountWith(router);
    await router.push('/a/1');

    nav.goBack();
    await flushPromises();

    expect(router.currentRoute.value.path).toBe('/');
    expect(nav.canGoForward).toBe(true);
  });

  it('router.replace re-hydrating the same route creates no new history entry (the "skip re-push" case)', async () => {
    const router = makeRouter();
    const nav = await mountWith(router);
    await router.push('/a/1');
    expect(nav.canGoBack).toBe(true);

    await router.replace('/a/2'); // e.g. the cached session loaded with more detail

    expect(router.currentRoute.value.params.id).toBe('2');
    // Still only one step back to "/", not two -- replace never pushed a second entry.
    nav.goBack();
    await flushPromises();
    expect(router.currentRoute.value.path).toBe('/');
    expect(nav.canGoBack).toBe(false);
  });

  it('pushing a new route after going back discards the old forward entry (the "trim forward history" case)', async () => {
    const router = makeRouter();
    const nav = await mountWith(router);
    await router.push('/a/1');
    nav.goBack();
    await flushPromises();
    expect(nav.canGoForward).toBe(true);

    await router.push('/b'); // a genuinely new choice, not forward to the old /a/1

    expect(nav.canGoForward).toBe(false);
    expect(router.currentRoute.value.path).toBe('/b');
  });

  it('goBack/goForward are no-ops at either end of history', async () => {
    const router = makeRouter();
    const nav = await mountWith(router);

    nav.goBack(); // nothing behind "/"
    await flushPromises();
    expect(router.currentRoute.value.path).toBe('/');

    nav.goForward(); // nothing ahead either
    await flushPromises();
    expect(router.currentRoute.value.path).toBe('/');
  });
});
