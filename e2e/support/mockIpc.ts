// Injects the exact `window.__TAURI_INTERNALS__` shape `@tauri-apps/api/core`'s `invoke()` reads
// (confirmed directly from `@tauri-apps/api/mocks`' own source -- a dependency-free set of plain
// functions touching only `window`/`crypto`), so the real built frontend believes it is running
// inside Tauri. `@tauri-apps/api/mocks` itself cannot be imported into the init script: Playwright
// serializes an `addInitScript` function via `.toString()` with no bundling, so anything the
// function body references has to be self-contained -- this is that same logic, copied rather
// than imported, not reinvented.
//
// port-plan.md P-8: "Playwright against the built frontend with fixture-backed IPC mocks" --
// handled commands return real fixture-shaped DTOs; everything else resolves to `null` rather
// than rejecting, since a broad app-level smoke test should not fail over a feature area it
// never asserts on.
//
// Every case below returns its RAW value, not `{status: 'ok', data: value}` -- confirmed the hard
// way (a real app-level crash: "Cannot read properties of undefined (reading 'map')", traced to
// this exact double-wrap, the same mistake M2-19's own mock bug made): `window.__TAURI_INTERNALS__
// .invoke` stands in for the raw Tauri IPC call, and `typedError()` (bindings.ts) is the layer
// that wraps *that* raw return value into `{status, data}` on the way out -- wrapping it here too
// means the frontend's own `unwrap()` hands callers the inner `{status, data}` object as if it
// were the real payload, which has no `.value`/`.length`/etc. of its own.

import type { Page } from '@playwright/test';
import type { MockFixtures } from './fixtures';

function installMockIpcInPage(fixtures: MockFixtures) {
  const win = window as unknown as {
    __TAURI_INTERNALS__: Record<string, unknown>;
    __TAURI_EVENT_PLUGIN_INTERNALS__: Record<string, unknown>;
  };
  win.__TAURI_INTERNALS__ = win.__TAURI_INTERNALS__ ?? {};
  win.__TAURI_EVENT_PLUGIN_INTERNALS__ = win.__TAURI_EVENT_PLUGIN_INTERNALS__ ?? {};

  const listeners = new Map<string, number[]>();

  async function invoke(cmd: string, args?: Record<string, unknown>): Promise<unknown> {
    switch (cmd) {
      case 'plugin:event|listen': {
        const event = (args as { event: string; handler: number }).event;
        const handler = (args as { event: string; handler: number }).handler;
        if (!listeners.has(event)) listeners.set(event, []);
        listeners.get(event)!.push(handler);
        return handler;
      }
      case 'plugin:event|unlisten':
        return null;
      case 'plugin:event|emit':
        return null;
      case 'get_app_info':
        return fixtures.appInfo;
      case 'get_prefs':
        return fixtures.prefs;
      case 'update_prefs':
        Object.assign(fixtures.prefs, Object.fromEntries(Object.entries((args as { patch: Record<string, unknown> }).patch).filter(([, v]) => v !== null)));
        return fixtures.prefs;
      case 'get_sidebar_counts':
        return fixtures.sidebarCounts;
      case 'list_projects':
        return { value: fixtures.projects, skipped: 0 };
      case 'list_sessions': {
        const projectId = (args as { projectId: string }).projectId;
        return { value: fixtures.sessionsByProject[projectId] ?? [], skipped: 0 };
      }
      default:
         
        console.warn(`[e2e mockIpc] unhandled command, resolving null: ${cmd}`);
        return null;
    }
  }

  let nextCallbackId = 1;
  const callbacks = new Map<number, (data: unknown) => unknown>();

  win.__TAURI_INTERNALS__.invoke = (cmd: string, args?: Record<string, unknown>) => invoke(cmd, args);
  win.__TAURI_INTERNALS__.transformCallback = (callback: (data: unknown) => unknown) => {
    const id = nextCallbackId++;
    callbacks.set(id, callback);
    return id;
  };
  win.__TAURI_INTERNALS__.unregisterCallback = (id: number) => callbacks.delete(id);
  win.__TAURI_INTERNALS__.runCallback = (id: number, data: unknown) => callbacks.get(id)?.(data);
  win.__TAURI_INTERNALS__.callbacks = callbacks;
  win.__TAURI_INTERNALS__.metadata = { currentWindow: { label: 'main' }, currentWebview: { windowLabel: 'main', label: 'main' } };
  win.__TAURI_EVENT_PLUGIN_INTERNALS__.unregisterListener = () => {};
}

/** Installs the mock before the app's own first script runs (Playwright's documented pattern for
 * testing a page that expects a global the real environment would otherwise provide). */
export async function installMockIpc(page: Page, fixtures: MockFixtures) {
  await page.addInitScript(installMockIpcInPage, fixtures);
}
