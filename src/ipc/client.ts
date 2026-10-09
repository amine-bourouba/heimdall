// The one path into the webview's IPC (ipc-contract.md §2: "The webview imports IPC only
// through src/ipc/client.ts" -- ESLint enforces this everywhere outside src/ipc/**).
//
// `ipc` unwraps every generated command: resolves with the real data on success, throws a typed
// IpcError on failure, so callers use plain try/catch instead of checking `.status` everywhere.
// `onDataChanged` wraps the `data-changed` event with the area filtering stores actually need.

import { commands, events } from './bindings';
import type { Area, AppAction, AppError, DataChange } from './bindings';

/** ADR 0005's AppError, thrown instead of returned so callers can use plain try/catch. */
export class IpcError extends Error {
  readonly kind: AppError['kind'];
  readonly path: string | null;

  constructor(error: AppError) {
    super(error.message);
    this.name = 'IpcError';
    this.kind = error.kind;
    this.path = error.path;
  }
}

type TauriResult<T> = { status: 'ok'; data: T } | { status: 'error'; error: AppError };

function unwrap<T>(result: Promise<TauriResult<T>>): Promise<T> {
  return result.then((r) => {
    if (r.status === 'ok') return r.data;
    throw new IpcError(r.error);
  });
}

type Commands = typeof commands;

/** Every generated command, each returning the unwrapped data or throwing IpcError. */
type UnwrappedCommands = {
  [K in keyof Commands]: Commands[K] extends (...args: infer A) => Promise<TauriResult<infer D>>
    ? (...args: A) => Promise<D>
    : never;
};

export const ipc = Object.fromEntries(
  Object.entries(commands).map(([name, fn]) => [
    name,
    (...args: unknown[]) => unwrap((fn as (...a: unknown[]) => Promise<TauriResult<unknown>>)(...args)),
  ]),
) as UnwrappedCommands;

/**
 * Subscribes to `data-changed`, calling `handler` only with the changes relevant to `areas` --
 * area `"all"` (watcher overflow/rescan, ipc-contract.md §7) always matches, since it means
 * "reload whatever is on screen" regardless of which areas a store asked about.
 */
export function onDataChanged(areas: Area[], handler: (changes: DataChange[]) => void) {
  const wanted = new Set<Area>(areas);
  return events.dataChanged.listen((event) => {
    const relevant = event.payload.filter((change) => change.area === 'all' || wanted.has(change.area));
    if (relevant.length > 0) handler(relevant);
  });
}

/** Subscribes to `menu-action` (macOS native menu, M2-13), calling `handler` with the action
 * name so a caller can feed it straight into `useAppActions().dispatch(...)`. */
export function onMenuAction(handler: (action: AppAction) => void) {
  return events.menuAction.listen((event) => handler(event.payload.action));
}
