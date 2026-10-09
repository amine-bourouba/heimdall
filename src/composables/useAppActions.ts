// The single dispatcher shared by useShortcuts.ts, the real backend's `menu-action` event
// (macOS menu, M2-13) and the in-app menu (task M2-13) -- each just calls `dispatch(action)`;
// whoever actually performs an action registers a handler with `on(action, handler)`. A plain
// module-level registry, not a Pinia store: ES modules are already singletons, and this has no
// state worth putting in devtools, just plumbing.

import type { AppAction } from '@/ipc/bindings';

type Handler = () => void;

const handlers = new Map<AppAction, Set<Handler>>();

export function useAppActions() {
  function on(action: AppAction, handler: Handler): () => void {
    if (!handlers.has(action)) handlers.set(action, new Set());
    handlers.get(action)!.add(handler);
    return () => handlers.get(action)?.delete(handler);
  }

  function dispatch(action: AppAction) {
    handlers.get(action)?.forEach((handler) => handler());
  }

  return { on, dispatch };
}
