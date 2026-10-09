import { describe, expect, it } from 'vitest';
import { ipc } from './client';
import { fixtures, mockIpcWithFixtures } from './testing';

describe('mockIpcWithFixtures', () => {
  it('answers list_projects with the real, Rust-generated projects fixture', async () => {
    mockIpcWithFixtures();
    await expect(ipc.listProjects()).resolves.toEqual({ value: fixtures.projects, skipped: 0 });
  });

  it('answers list_sessions with the real poirot-sessions fixture', async () => {
    mockIpcWithFixtures();
    await expect(ipc.listSessions('-Users-fixture-Dev-poirot', false)).resolves.toEqual({
      value: fixtures.poirotSessions,
      skipped: 0,
    });
  });

  it('lets a test override one command while keeping the others on fixtures', async () => {
    mockIpcWithFixtures({ list_projects: () => ({ value: [], skipped: 0 }) });
    await expect(ipc.listProjects()).resolves.toEqual({ value: [], skipped: 0 });
    await expect(ipc.listSessions('-Users-fixture-Dev-poirot', false)).resolves.toEqual({
      value: fixtures.poirotSessions,
      skipped: 0,
    });
  });

  it('throws loudly for a command with neither a default nor an override', async () => {
    mockIpcWithFixtures();
    await expect(ipc.getAppInfo()).rejects.toThrow(/no fixture or override wired for "get_app_info"/);
  });
});
