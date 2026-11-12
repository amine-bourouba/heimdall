// Ported from KeyboardNavigationModifier.swift: arrow-key sidebar navigation, Enter to activate,
// and the layered Escape dismiss. Escape layers are injected in priority order rather than
// hardcoded to specific stores -- `Sidebar.vue` (the one real caller) wires the real four
// (session search, tool filter, config detail, session detail; M3-19), each skipped when inactive.
//
// the Swift reference app's own "blur a focused text field first" is two separate mechanisms (an NSEvent monitor
// for the text-field case, SwiftUI onKeyPress for everything else) because AppKit text editing
// intercepts Escape before SwiftUI sees it. The web has one event per keystroke, so this checks
// the focused element first and returns -- a focused input never also triggers an escape layer
// in the same keystroke, matching the Swift reference app's own two-step behaviour with one check instead of two.

import { onMounted, onScopeDispose } from 'vue';
import type { Ref } from 'vue';

export interface EscapeLayer {
  isActive: () => boolean;
  dismiss: () => void;
}

export interface UseKeyboardNavOptions {
  /** Checked in order (KeyboardNavigationModifier.swift:52-72's exact order); the first active
   * one is dismissed and no other layer is checked for this keystroke. */
  escapeLayers: EscapeLayer[];
  sidebarItemCount: () => number;
  sidebarIndex: Ref<number>;
  onSidebarActivate: () => void;
  /** KeyboardNavigationModifier.swift:17-19's own guard: skip everything while a bigger overlay
   * (search, the shortcut-help sheet) owns keyboard input. */
  isSuppressed?: () => boolean;
}

function isTextInput(element: Element | null): boolean {
  return element instanceof HTMLInputElement || element instanceof HTMLTextAreaElement;
}

// A focused control takes its own keys before this document-level handler (in the Swift app a focused
// field or button consumes the key before the window-level modifier sees it). Without that, Return on a
// page button never clicked it and instead navigated to the highlighted sidebar item (found driving the
// real build, M3-50: "Set as Default" on the Models screen), and the arrows could not change a page-level
// <select> or move a caret.
const UNEDITABLE = ':not([contenteditable="false"])';
/** Controls that act on Return themselves: a button or link clicks, a field submits or adds a line. */
const OWNS_RETURN = `button, a[href], input, textarea, select, summary, [contenteditable]${UNEDITABLE}, [role="button"], [role="switch"], [role="checkbox"], [role="radio"], [role="tab"], [role="menuitem"], [role="option"], [role="link"]`;
/** Controls that also use the arrow keys (a button does not, so arrows still reach the sidebar from one). */
const OWNS_ARROWS = `input, textarea, select, [contenteditable]${UNEDITABLE}, [role="radio"], [role="tab"], [role="menuitem"], [role="option"], [role="slider"], [role="listbox"]`;

export function useKeyboardNav(options: UseKeyboardNavOptions) {
  function handleEscape() {
    if (isTextInput(document.activeElement)) {
      (document.activeElement as HTMLElement).blur();
      return;
    }
    const layer = options.escapeLayers.find((l) => l.isActive());
    layer?.dismiss();
  }

  function moveSidebarSelection(delta: number) {
    const count = options.sidebarItemCount();
    if (count === 0) return;
    options.sidebarIndex.value = Math.max(0, Math.min(options.sidebarIndex.value + delta, count - 1));
  }

  function onKeydown(event: KeyboardEvent) {
    if (options.isSuppressed?.()) return;
    // A modal dialog owns the keyboard while focus is inside it. Without this, Return/arrows from
    // a dialog's own button or select reached this handler too -- Return activated the highlighted
    // sidebar item (navigating away right after a form saved; found by driving the real build,
    // M3-47) and the unconditional `preventDefault` below stopped Return clicking a focused button
    // and the arrows changing a <select> -- and Escape also dismissed an unrelated escape layer
    // alongside the dialog's own close.
    if (event.target instanceof Element && event.target.closest('[aria-modal="true"]')) return;

    if (event.key === 'Escape') {
      handleEscape();
      return;
    }

    // KeyboardNavigationModifier.swift:33's own guard: plain arrow/Enter only, no modifiers.
    if (event.metaKey || event.ctrlKey || event.altKey || event.shiftKey) return;

    // `event.target` is the document itself for a key with nothing focused: not an Element, so no guard.
    const target = event.target instanceof Element ? event.target : null;
    switch (event.key) {
      case 'ArrowDown':
      case 'ArrowUp':
        if (target?.closest(OWNS_ARROWS)) return;
        moveSidebarSelection(event.key === 'ArrowDown' ? 1 : -1);
        event.preventDefault();
        break;
      case 'Enter':
        if (target?.closest(OWNS_RETURN)) return;
        options.onSidebarActivate();
        event.preventDefault();
        break;
    }
  }

  onMounted(() => document.addEventListener('keydown', onKeydown));
  onScopeDispose(() => document.removeEventListener('keydown', onKeydown));
}
