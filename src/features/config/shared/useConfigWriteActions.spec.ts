// M3-50: create-from-template and delete for commands, skills and output styles. Each case checks
// the exact command and argument sent: a wrong kind or ref here would create or delete the wrong file.
import { mockIPC } from '@tauri-apps/api/mocks';
import { createPinia, setActivePinia } from 'pinia';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { ConfigRef } from '@/ipc/bindings';
import { useToastsStore } from '@/stores/toasts';
import { useConfigWriteActions } from './useConfigWriteActions';

const deploy: ConfigRef = { kind: 'command', project: null, slug: 'deploy' };
const busy = () => {
  throw { kind: 'busy', message: 'the file is in use by another program', path: null };
};

interface Sent {
  create: unknown[];
  delete: unknown[];
}

let sent: Sent;
let toasts: ReturnType<typeof useToastsStore>;

function setup(handlers: Partial<Record<string, (args: unknown) => unknown>> = {}) {
  setActivePinia(createPinia());
  toasts = useToastsStore();
  sent = { create: [], delete: [] };
  mockIPC((cmd, args) => {
    if (handlers[cmd]) return handlers[cmd]!(args);
    if (cmd === 'create_config_template') {
      sent.create.push(args);
      return { kind: (args as { kind: string }).kind, project: null, slug: 'new-thing' };
    }
    if (cmd === 'delete_config_item') {
      sent.delete.push(args);
      return 'written';
    }
    throw new Error(`unexpected command: ${cmd}`);
  });
}

beforeEach(() => {
  setup();
});

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('createFromTemplate', () => {
  it('asks for a global template of its own kind and names the file it made', async () => {
    const actions = useConfigWriteActions('skill');

    expect(await actions.createFromTemplate()).toBe(true);

    expect(sent.create).toEqual([{ kind: 'skill', project: null }]);
    expect(toasts.current).toMatchObject({ message: 'Created new skill template: new-thing', style: 'success' });
  });

  it.each([
    ['command', 'command'],
    ['outputStyle', 'output style'],
  ] as const)('%s templates are called "%s" in the toast', async (kind, noun) => {
    expect(await useConfigWriteActions(kind).createFromTemplate()).toBe(true);
    expect(sent.create).toEqual([{ kind, project: null }]);
    expect(toasts.current?.message).toContain(`Created new ${noun} template`);
  });

  it('shows the cause and reports nothing created when the write fails, and can be retried', async () => {
    setup({ create_config_template: busy });
    const actions = useConfigWriteActions('command');

    expect(await actions.createFromTemplate()).toBe(false);

    expect(toasts.current).toMatchObject({ message: 'the file is in use by another program', style: 'error' });
    expect(actions.isBusy.value).toBe(false);
  });

  it('ignores a second click while the first is still in flight', async () => {
    let release: (value: unknown) => void = () => {};
    let calls = 0;
    setup({
      create_config_template: () => {
        calls++;
        return new Promise((resolve) => (release = resolve));
      },
    });
    const actions = useConfigWriteActions('command');

    const first = actions.createFromTemplate();
    expect(await actions.createFromTemplate()).toBe(false);
    release({ kind: 'command', project: null, slug: 'new-command' });

    expect(await first).toBe(true);
    expect(calls).toBe(1);
  });
});

describe('deleteItem', () => {
  it('sends nothing when the confirmation is declined', async () => {
    const confirmSpy = vi.fn().mockReturnValue(false);
    vi.stubGlobal('confirm', confirmSpy);

    expect(await useConfigWriteActions('command').deleteItem(deploy, '/deploy')).toBe(false);

    expect(confirmSpy).toHaveBeenCalledTimes(1);
    expect(sent.delete).toEqual([]);
    expect(toasts.current).toBeNull();
  });

  it('deletes exactly the reference it was given once confirmed, and says so', async () => {
    const confirmSpy = vi.fn().mockReturnValue(true);
    vi.stubGlobal('confirm', confirmSpy);

    expect(await useConfigWriteActions('command').deleteItem(deploy, '/deploy')).toBe(true);

    expect(sent.delete).toEqual([{ configRef: deploy }]);
    expect(toasts.current).toMatchObject({ message: 'Deleted /deploy', style: 'info' });
    expect(confirmSpy.mock.calls[0][0]).toContain('"/deploy"');
  });

  it('warns that deleting a skill removes its whole folder, and a file kind does not claim that', async () => {
    const confirmSpy = vi.fn().mockReturnValue(false);
    vi.stubGlobal('confirm', confirmSpy);

    await useConfigWriteActions('skill').deleteItem({ ...deploy, kind: 'skill' }, 'deploy');
    await useConfigWriteActions('outputStyle').deleteItem({ ...deploy, kind: 'outputStyle' }, 'concise');

    expect(confirmSpy.mock.calls[0][0]).toContain('whole folder');
    expect(confirmSpy.mock.calls[1][0]).not.toContain('folder');
    expect(confirmSpy.mock.calls[1][0]).toContain('output style');
  });

  it('treats an item that was already gone as done, so the screen refreshes', async () => {
    vi.stubGlobal('confirm', vi.fn().mockReturnValue(true));
    setup({ delete_config_item: () => 'unchanged' });

    expect(await useConfigWriteActions('command').deleteItem(deploy, '/deploy')).toBe(true);

    expect(toasts.current).toMatchObject({ message: '/deploy was already gone', style: 'info' });
  });

  it('shows the cause and reports nothing deleted when the delete fails', async () => {
    vi.stubGlobal('confirm', vi.fn().mockReturnValue(true));
    setup({ delete_config_item: busy });
    const actions = useConfigWriteActions('command');

    expect(await actions.deleteItem(deploy, '/deploy')).toBe(false);

    expect(toasts.current).toMatchObject({ message: 'the file is in use by another program', style: 'error' });
    expect(actions.isBusy.value).toBe(false);
  });

  it('ignores a second delete while the first is still in flight', async () => {
    vi.stubGlobal('confirm', vi.fn().mockReturnValue(true));
    let release: (value: unknown) => void = () => {};
    let calls = 0;
    setup({
      delete_config_item: () => {
        calls++;
        return new Promise((resolve) => (release = resolve));
      },
    });
    const actions = useConfigWriteActions('command');

    const first = actions.deleteItem(deploy, '/deploy');
    expect(await actions.deleteItem(deploy, '/deploy')).toBe(false);
    release('written');

    expect(await first).toBe(true);
    expect(calls).toBe(1);
  });
});
