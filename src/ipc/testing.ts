// Test helper: mocks IPC commands with the real, Rust-generated DTO fixtures (M2-01's
// fixtures/dto/**), never a second hand-typed copy of the same data that could drift from what
// the real commands actually serialise.

import { mockIPC } from '@tauri-apps/api/mocks';
import projects from '../../fixtures/dto/projects.json';
import poirotSessions from '../../fixtures/dto/sessions-poirot.json';

export const fixtures = { projects, poirotSessions };

type MockHandler = (args: Record<string, unknown>) => unknown;

/** The real `Loaded<T>` shape (`{ value, skipped }`, ipc-contract.md §3) around a fixture. */
function loaded<T>(value: T) {
  return { value, skipped: 0 };
}

const defaultHandlers: Record<string, MockHandler> = {
  list_projects: () => loaded(projects),
  list_sessions: () => loaded(poirotSessions),
};

/**
 * Installs a mockIPC handler backed by the real DTO fixtures for `list_projects`/`list_sessions`,
 * with `overrides` taking priority (for a specific test's own error case or different args). A
 * command with neither an override nor a default throws loudly rather than returning `undefined`
 * silently -- the same "fail loud, not quiet" rule `src/test/setup.ts`'s own default handler uses.
 */
export function mockIpcWithFixtures(overrides: Record<string, MockHandler> = {}) {
  mockIPC((cmd, args) => {
    const handler = overrides[cmd] ?? defaultHandlers[cmd];
    if (!handler) {
      throw new Error(`mockIpcWithFixtures: no fixture or override wired for "${cmd}"`);
    }
    return handler((args ?? {}) as Record<string, unknown>);
  });
}
