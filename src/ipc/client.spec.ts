import { mockIPC } from '@tauri-apps/api/mocks';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { events } from './bindings';
import type { AppError, DataChange } from './bindings';
import { IpcError, ipc, onDataChanged, onMenuAction } from './client';

describe('ipc: success', () => {
  it('resolves with the unwrapped data, not the { status, data } envelope', async () => {
    const appInfo = { version: '1.0.0', identifier: 'io.test', os: 'windows', arch: 'x86_64', claudeHomeFound: true };
    mockIPC(() => appInfo);

    await expect(ipc.getAppInfo()).resolves.toEqual(appInfo);
  });

  it('passes arguments through to the real command name', async () => {
    mockIPC((cmd, args) => {
      expect(cmd).toBe('list_sessions');
      expect(args).toEqual({ projectId: 'proj-1', includeAgents: true });
      return { value: [], skipped: 0 };
    });

    await ipc.listSessions('proj-1', true);
  });
});

describe('ipc: every AppError kind throws a typed IpcError', () => {
  const kinds: AppError['kind'][] = ['notFound', 'io', 'parse', 'permission', 'busy', 'invalidInput'];

  for (const kind of kinds) {
    it(`kind "${kind}"`, async () => {
      const error: AppError = { kind, message: `${kind} happened`, path: kind === 'notFound' ? '/some/path' : null };
      mockIPC(() => {
        throw error;
      });

      await expect(ipc.getAppInfo()).rejects.toMatchObject({
        name: 'IpcError',
        kind: error.kind,
        message: error.message,
        path: error.path,
      });
      await expect(ipc.getAppInfo()).rejects.toBeInstanceOf(IpcError);
    });
  }
});

describe('onDataChanged filtering', () => {
  beforeEach(() => {
    // Overrides src/test/setup.ts's default mockIPC (which has no event support) with one that
    // does, per @tauri-apps/api/mocks' own documented pattern for testing listen()/emit().
    mockIPC(() => {}, { shouldMockEvents: true });
  });

  async function emitAndCollect(areas: Parameters<typeof onDataChanged>[0], changes: DataChange[]) {
    const handler = vi.fn();
    const unlisten = await onDataChanged(areas, handler);
    await events.dataChanged.emit(changes);
    unlisten();
    return handler;
  }

  it('calls the handler when a change matches a watched area', async () => {
    const handler = await emitAndCollect(['sessions'], [{ area: 'sessions', key: 'proj-1' }]);
    expect(handler).toHaveBeenCalledWith([{ area: 'sessions', key: 'proj-1' }]);
  });

  it('does not call the handler when no change matches a watched area', async () => {
    const handler = await emitAndCollect(['sessions'], [{ area: 'history', key: null }]);
    expect(handler).not.toHaveBeenCalled();
  });

  it('always calls the handler for area "all" (watcher overflow/rescan), even if unrequested', async () => {
    const handler = await emitAndCollect(['sessions'], [{ area: 'all', key: null }]);
    expect(handler).toHaveBeenCalledWith([{ area: 'all', key: null }]);
  });

  it('filters a mixed batch down to only the relevant changes', async () => {
    const handler = await emitAndCollect(
      ['sessions'],
      [
        { area: 'history', key: null },
        { area: 'sessions', key: 'proj-1' },
      ],
    );
    expect(handler).toHaveBeenCalledWith([{ area: 'sessions', key: 'proj-1' }]);
    expect(handler).toHaveBeenCalledTimes(1);
  });

  it('matches any of several watched areas', async () => {
    const handler = await emitAndCollect(['todos', 'history'], [{ area: 'history', key: null }]);
    expect(handler).toHaveBeenCalledWith([{ area: 'history', key: null }]);
  });
});

describe('onMenuAction', () => {
  beforeEach(() => {
    mockIPC(() => {}, { shouldMockEvents: true });
  });

  it('calls the handler with the action name, unwrapped from the event payload', async () => {
    const handler = vi.fn();
    const unlisten = await onMenuAction(handler);
    await events.menuAction.emit({ action: 'settings' });
    unlisten();
    expect(handler).toHaveBeenCalledWith('settings');
  });
});
