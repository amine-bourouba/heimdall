import { defineComponent, h, ref } from 'vue';
import { mount } from '@vue/test-utils';
import type { VueWrapper } from '@vue/test-utils';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { useKeyboardNav } from './useKeyboardNav';
import type { EscapeLayer } from './useKeyboardNav';

function fireKeydown(init: KeyboardEventInit) {
  document.dispatchEvent(new KeyboardEvent('keydown', { bubbles: true, cancelable: true, ...init }));
}

// useKeyboardNav attaches a document-level keydown listener for the lifetime of its host
// component -- an un-unmounted wrapper from an earlier test would keep firing into later ones
// (confirmed for real: without this cleanup, a later test's escape layer gets a stray dismiss()
// from an earlier test's still-attached listener). Tracked and unmounted in afterEach below.
let mounted: VueWrapper[] = [];

function mountWith(overrides: Partial<Parameters<typeof useKeyboardNav>[0]> = {}) {
  const sidebarIndex = ref(0);
  const onSidebarActivate = vi.fn();
  const defaults = {
    escapeLayers: [] as EscapeLayer[],
    sidebarItemCount: () => 3,
    sidebarIndex,
    onSidebarActivate,
  };
  const Host = defineComponent({
    setup() {
      useKeyboardNav({ ...defaults, ...overrides });
      return () => h('div');
    },
  });
  mounted.push(mount(Host, { attachTo: document.body }));
  return { sidebarIndex, onSidebarActivate };
}

beforeEach(() => {
  document.body.innerHTML = '';
});

afterEach(() => {
  mounted.forEach((wrapper) => wrapper.unmount());
  mounted = [];
});

// Ported from KeyboardNavigationModifier.swift's handleEscape() -- exact layer order.
describe('useKeyboardNav: Escape order', () => {
  function layer(): EscapeLayer & { dismiss: ReturnType<typeof vi.fn> } {
    let active = true;
    const dismiss = vi.fn(() => {
      active = false;
    });
    return { isActive: () => active, dismiss };
  }

  it('dismisses session search first when it is active', () => {
    const search = layer();
    const toolFilter = layer();
    mountWith({ escapeLayers: [search, toolFilter] });

    fireKeydown({ key: 'Escape' });

    expect(search.dismiss).toHaveBeenCalledTimes(1);
    expect(toolFilter.dismiss).not.toHaveBeenCalled();
  });

  it('falls through to the next layer once the earlier ones are inactive', () => {
    const search = { isActive: () => false, dismiss: vi.fn() };
    const toolFilter = { isActive: () => false, dismiss: vi.fn() };
    const configDetail = { isActive: () => true, dismiss: vi.fn() };
    const sessionSelection = { isActive: () => true, dismiss: vi.fn() };
    mountWith({ escapeLayers: [search, toolFilter, configDetail, sessionSelection] });

    fireKeydown({ key: 'Escape' });

    expect(configDetail.dismiss).toHaveBeenCalledTimes(1);
    expect(sessionSelection.dismiss).not.toHaveBeenCalled();
  });

  it('does nothing when no layer is active', () => {
    const layers = [
      { isActive: () => false, dismiss: vi.fn() },
      { isActive: () => false, dismiss: vi.fn() },
    ];
    mountWith({ escapeLayers: layers });

    expect(() => fireKeydown({ key: 'Escape' })).not.toThrow();
    for (const l of layers) expect(l.dismiss).not.toHaveBeenCalled();
  });

  it('blurs a focused input/textarea first, without touching any escape layer', () => {
    const search = { isActive: () => true, dismiss: vi.fn() };
    mountWith({ escapeLayers: [search] });
    const input = document.createElement('input');
    document.body.appendChild(input);
    input.focus();
    expect(document.activeElement).toBe(input);

    fireKeydown({ key: 'Escape' });

    expect(document.activeElement).not.toBe(input);
    expect(search.dismiss).not.toHaveBeenCalled();
  });
});

describe('useKeyboardNav: sidebar arrow navigation and Enter', () => {
  it('ArrowDown/ArrowUp move the index, clamped to [0, count-1]', () => {
    const { sidebarIndex } = mountWith({ sidebarItemCount: () => 3 });

    fireKeydown({ key: 'ArrowDown' });
    expect(sidebarIndex.value).toBe(1);
    fireKeydown({ key: 'ArrowDown' });
    fireKeydown({ key: 'ArrowDown' });
    expect(sidebarIndex.value).toBe(2); // clamped at count - 1

    fireKeydown({ key: 'ArrowUp' });
    fireKeydown({ key: 'ArrowUp' });
    fireKeydown({ key: 'ArrowUp' });
    expect(sidebarIndex.value).toBe(0); // clamped at 0
  });

  it('Enter activates the current sidebar selection', () => {
    const { onSidebarActivate } = mountWith();
    fireKeydown({ key: 'Enter' });
    expect(onSidebarActivate).toHaveBeenCalledTimes(1);
  });

  it('ignores arrow keys held with a modifier (KeyboardNavigationModifier.swift:33)', () => {
    const { sidebarIndex } = mountWith();
    fireKeydown({ key: 'ArrowDown', metaKey: true });
    expect(sidebarIndex.value).toBe(0);
  });

  it('does nothing at all while suppressed (search overlay or shortcut-help sheet open)', () => {
    const { sidebarIndex, onSidebarActivate } = mountWith({ isSuppressed: () => true });
    fireKeydown({ key: 'ArrowDown' });
    fireKeydown({ key: 'Enter' });
    expect(sidebarIndex.value).toBe(0);
    expect(onSidebarActivate).not.toHaveBeenCalled();
  });
});
