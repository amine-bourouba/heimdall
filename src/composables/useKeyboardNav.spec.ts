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

// Found by driving the real build (M3-47): a dialog's own Return/arrows/Escape also reached this
// document-level handler. Keys whose target is inside an aria-modal dialog belong to the dialog.
describe('useKeyboardNav: keys from inside an aria-modal dialog are left to the dialog', () => {
  function fireFromModal(init: KeyboardEventInit, tag: 'button' | 'select' | 'input' = 'button') {
    document.body.innerHTML = '<div aria-modal="true" role="dialog"><div><' + tag + '></' + tag + '></div></div>';
    const target = document.querySelector(tag)!;
    const event = new KeyboardEvent('keydown', { bubbles: true, cancelable: true, ...init });
    target.dispatchEvent(event);
    return event;
  }

  it('Return does not activate the sidebar, and is not default-prevented (so a focused button can still be clicked)', () => {
    const { onSidebarActivate } = mountWith();
    const event = fireFromModal({ key: 'Enter' });
    expect(onSidebarActivate).not.toHaveBeenCalled();
    expect(event.defaultPrevented).toBe(false);
  });

  it('arrow keys do not move the sidebar and are not default-prevented (so a <select> can change)', () => {
    const { sidebarIndex } = mountWith();
    const event = fireFromModal({ key: 'ArrowDown' }, 'select');
    expect(sidebarIndex.value).toBe(0);
    expect(event.defaultPrevented).toBe(false);
  });

  it('Escape does not dismiss an unrelated escape layer, nor blur the dialog field, alongside the dialog' + "'s own close", () => {
    let dismissed = false;
    mountWith({ escapeLayers: [{ isActive: () => true, dismiss: () => (dismissed = true) }] });
    fireFromModal({ key: 'Escape' }, 'input');
    expect(dismissed).toBe(false);
  });

  it('the same keys from outside any dialog still navigate (the guard is scoped to aria-modal)', () => {
    const { onSidebarActivate } = mountWith();
    document.body.innerHTML = '<div><p></p></div>';
    document.querySelector('p')!.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true, cancelable: true }));
    expect(onSidebarActivate).toHaveBeenCalledTimes(1);
  });
});

// Found by driving the real build (M3-50): Return on a focused page button ("Set as Default") never
// clicked it and navigated to the highlighted sidebar item instead. A focused control takes its own keys.
describe('useKeyboardNav: a focused control keeps the keys it handles itself', () => {
  function fireFrom(markup: string, init: KeyboardEventInit) {
    document.body.innerHTML = `<main>${markup}</main>`;
    const target = document.body.querySelector('main')!.firstElementChild!;
    const event = new KeyboardEvent('keydown', { bubbles: true, cancelable: true, ...init });
    target.dispatchEvent(event);
    return event;
  }

  const RETURN_OWNERS: [string, string][] = [
    ['a button', '<button></button>'],
    ['a link', '<a href="#/x"></a>'],
    ['a text input', '<input type="text">'],
    ['a textarea', '<textarea></textarea>'],
    ['a select', '<select></select>'],
    ['a switch', '<span role="switch" tabindex="0"></span>'],
    ['an element inside a button', '<button><svg></svg></button>'],
    ['an editable element', '<div contenteditable="true"></div>'],
  ];

  it.each(RETURN_OWNERS)('Return on %s is left to it: no sidebar activation, not default-prevented', (_name, markup) => {
    const { onSidebarActivate } = mountWith();
    const event = fireFrom(markup, { key: 'Enter' });
    expect(onSidebarActivate).not.toHaveBeenCalled();
    expect(event.defaultPrevented).toBe(false);
  });

  it('Return on nothing in particular still activates the sidebar, and on a non-editable div too', () => {
    const { onSidebarActivate } = mountWith();
    fireKeydown({ key: 'Enter' }); // nothing focused: the target is the document
    const event = fireFrom('<div contenteditable="false"></div>', { key: 'Enter' });
    expect(onSidebarActivate).toHaveBeenCalledTimes(2);
    expect(event.defaultPrevented).toBe(true);
  });

  it.each([
    ['a select', '<select></select>'],
    ['a text input', '<input type="text">'],
    ['a textarea', '<textarea></textarea>'],
  ])('the arrow keys on %s are left to it', (_name, markup) => {
    const { sidebarIndex } = mountWith();
    const down = fireFrom(markup, { key: 'ArrowDown' });
    const up = fireFrom(markup, { key: 'ArrowUp' });
    expect(sidebarIndex.value).toBe(0);
    expect([down.defaultPrevented, up.defaultPrevented]).toEqual([false, false]);
  });

  it('a button does not use the arrow keys, so they still move the sidebar from one', () => {
    const { sidebarIndex } = mountWith();
    const event = fireFrom('<button></button>', { key: 'ArrowDown' });
    expect(sidebarIndex.value).toBe(1);
    expect(event.defaultPrevented).toBe(true);
  });
});
