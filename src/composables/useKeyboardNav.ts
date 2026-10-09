// Ported from KeyboardNavigationModifier.swift: arrow-key sidebar navigation, Enter to activate,
// and the layered Escape dismiss. Escape layers are injected in priority order rather than
// hardcoded to specific stores -- several of the real ones (session search, tool filter, config
// detail) have no Heimdall store yet; a caller passes only the layers it actually has, and an
// inactive layer is simply skipped.
//
// Poirot's own "blur a focused text field first" is two separate mechanisms (an NSEvent monitor
// for the text-field case, SwiftUI onKeyPress for everything else) because AppKit text editing
// intercepts Escape before SwiftUI sees it. The web has one event per keystroke, so this checks
// the focused element first and returns -- a focused input never also triggers an escape layer
// in the same keystroke, matching Poirot's own two-step behaviour with one check instead of two.

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

    if (event.key === 'Escape') {
      handleEscape();
      return;
    }

    // KeyboardNavigationModifier.swift:33's own guard: plain arrow/Enter only, no modifiers.
    if (event.metaKey || event.ctrlKey || event.altKey || event.shiftKey) return;

    switch (event.key) {
      case 'ArrowDown':
        moveSidebarSelection(1);
        event.preventDefault();
        break;
      case 'ArrowUp':
        moveSidebarSelection(-1);
        event.preventDefault();
        break;
      case 'Enter':
        options.onSidebarActivate();
        event.preventDefault();
        break;
    }
  }

  onMounted(() => document.addEventListener('keydown', onKeydown));
  onScopeDispose(() => document.removeEventListener('keydown', onKeydown));
}
