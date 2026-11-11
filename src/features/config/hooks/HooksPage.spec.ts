// M3-13's own AC: "Hooks group by event with matcher and handler summaries..." / "No mutating
// controls yet (M3-47, M3-50)." Goal: "...including copy-as-JSON for hooks." M3-13's own
// "no mutating controls yet" test was explicitly scoped to "yet", and M3-47 is exactly the task
// that adds them -- so that block is replaced here (not left asserting something now false) by
// the "New Hook, edit, delete and import" block covering the wiring this task adds.
//
// HookForm.vue is a `<Teleport to="body">` root, so every query into it goes through
// `document.body`, not `wrapper.find()` (HookForm.spec.ts's own documented reasoning).
import { DOMWrapper, flushPromises, mount } from '@vue/test-utils';
import { mockIPC } from '@tauri-apps/api/mocks';
import { createPinia, setActivePinia } from 'pinia';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { HookEntry, ProjectContext } from '@/ipc/bindings';
import { useToastsStore } from '@/stores/toasts';
import { useConfigStore } from '../store';
import HooksPage from './HooksPage.vue';

function body() {
  return new DOMWrapper(document.body);
}

function makeEntry(overrides: Partial<HookEntry> = {}): HookEntry {
  return {
    event: 'PreToolUse',
    matcherGroupIndex: 0,
    matcher: 'Bash|Write',
    handlers: [{ type: 'command', command: 'echo hi', url: null, timeout: 30, statusMessage: null, extraKeys: [] }],
    scope: 'global',
    ...overrides,
  };
}

let mounted: ReturnType<typeof mount>[] = [];

async function mountPage(ipcHandler: (cmd: string, args: unknown) => unknown, selectedProject: ProjectContext | null = null) {
  setActivePinia(createPinia());
  mockIPC((cmd, args) => {
    if (cmd === 'list_projects') return { value: [], skipped: 0 };
    const result = ipcHandler(cmd, args);
    if (result !== undefined) return result;
    throw new Error(`unexpected command: ${cmd}`);
  });
  const wrapper = mount(HooksPage, { attachTo: document.body });
  mounted.push(wrapper);
  await flushPromises();
  // Set only AFTER the first load: ConfigProjectPicker's own `configStore.load()` resets
  // `selectedProject` from Prefs (null here) exactly once on mount, so assigning it before
  // mounting is silently overwritten -- a test that "selects a project" that way is selecting
  // nothing. The page re-lists on this change (deep watch), which the handlers here tolerate.
  if (selectedProject) {
    useConfigStore().selectedProject = selectedProject;
    await flushPromises();
  }
  return wrapper;
}

beforeEach(() => {
  document.body.innerHTML = '';
});

afterEach(() => {
  vi.unstubAllGlobals();
  mounted.forEach((w) => w.unmount());
  mounted = [];
});

describe('HooksPage: grouping by event (AC)', () => {
  it('groups entries under their own event section with a matcher and handler summary', async () => {
    const wrapper = await mountPage((cmd) => {
      if (cmd === 'list_hooks') {
        return {
          value: [
            makeEntry({ event: 'PreToolUse', matcher: 'Bash' }),
            makeEntry({ event: 'SessionStart', matcherGroupIndex: 1, matcher: null, handlers: [{ type: 'http', command: null, url: 'https://example.com', timeout: null, statusMessage: 'Notifying', extraKeys: [] }] }),
          ],
          skipped: 0,
        };
      }
    });

    expect(wrapper.text()).toContain('PreToolUse');
    expect(wrapper.text()).toContain('SessionStart');
    expect(wrapper.text()).toContain('Bash');
    expect(wrapper.text()).toContain('https://example.com');
    expect(wrapper.text()).toContain('Notifying');
  });

  it('shows an "N handlers" badge only when a group has more than one handler', async () => {
    const wrapper = await mountPage((cmd) => {
      if (cmd === 'list_hooks') {
        return {
          value: [
            makeEntry({
              handlers: [
                { type: 'command', command: 'one', url: null, timeout: null, statusMessage: null, extraKeys: [] },
                { type: 'command', command: 'two', url: null, timeout: null, statusMessage: null, extraKeys: [] },
              ],
            }),
          ],
          skipped: 0,
        };
      }
    });
    expect(wrapper.text()).toContain('2 handlers');
  });
});

describe('HooksPage: New Hook, edit, delete and import (M3-47)', () => {
  it('opens the form in create mode from the New Hook button', async () => {
    const wrapper = await mountPage((cmd) => {
      if (cmd === 'list_hooks') return { value: [], skipped: 0 };
    });
    await wrapper.find('button[title="New Hook"]').trigger('click');
    expect(body().text()).toContain('New Hook');
    expect(body().find('button[title="Create hook"]').exists()).toBe(true);
  });

  it('opens the form pre-filled in edit mode when a card is clicked', async () => {
    const wrapper = await mountPage((cmd) => {
      if (cmd === 'list_hooks') return { value: [makeEntry({ matcher: 'Bash' })], skipped: 0 };
    });
    await wrapper.find('button[title="Edit hook"]').trigger('click');
    expect(body().text()).toContain('Edit Hook');
    expect((body().find('[placeholder="e.g. Bash|Write"]').element as HTMLInputElement).value).toBe('Bash');
  });

  it('reloads the list once the form reports a save', async () => {
    let listCalls = 0;
    const wrapper = await mountPage((cmd) => {
      if (cmd === 'list_hooks') {
        listCalls++;
        return { value: [], skipped: 0 };
      }
      if (cmd === 'save_hook') return 'written';
    });
    await wrapper.find('button[title="New Hook"]').trigger('click');
    await body().find('[placeholder="e.g. .claude/hooks/check.sh"]').setValue('x');
    await body().find('button[title="Create hook"]').trigger('click');
    await flushPromises();
    expect(listCalls).toBe(2); // once on mount, once after the save closes the form
  });

  it('asks for confirmation before deleting, and calls delete_hook with the entry\'s own event/index when confirmed', async () => {
    const confirmSpy = vi.fn().mockReturnValue(true);
    vi.stubGlobal('confirm', confirmSpy);
    let deleteArgs: unknown = null;
    const wrapper = await mountPage((cmd, args) => {
      if (cmd === 'list_hooks') return { value: [makeEntry({ event: 'PostToolUse', matcherGroupIndex: 3 })], skipped: 0 };
      if (cmd === 'delete_hook') {
        deleteArgs = args;
        return 'written';
      }
    });
    await wrapper.find('button[title="Delete hook"]').trigger('click');
    await flushPromises();
    expect(confirmSpy).toHaveBeenCalledTimes(1);
    expect(deleteArgs).toEqual({ project: null, event: 'PostToolUse', index: 3 });
  });

  it('does not call delete_hook when the confirmation is declined', async () => {
    const confirmSpy = vi.fn().mockReturnValue(false);
    vi.stubGlobal('confirm', confirmSpy);
    let deleteCalled = false;
    const wrapper = await mountPage((cmd) => {
      if (cmd === 'list_hooks') return { value: [makeEntry()], skipped: 0 };
      if (cmd === 'delete_hook') {
        deleteCalled = true;
        return 'written';
      }
    });
    await wrapper.find('button[title="Delete hook"]').trigger('click');
    await flushPromises();
    expect(confirmSpy).toHaveBeenCalledTimes(1);
    expect(deleteCalled).toBe(false);
  });

  it('calls import_hooks and reloads the list on a written outcome', async () => {
    let listCalls = 0;
    let importCalled = false;
    const wrapper = await mountPage((cmd) => {
      if (cmd === 'list_hooks') {
        listCalls++;
        return { value: [], skipped: 0 };
      }
      if (cmd === 'import_hooks') {
        importCalled = true;
        return 'written';
      }
    });
    await wrapper.find('button[title="Import JSON"]').trigger('click');
    await flushPromises();
    expect(importCalled).toBe(true);
    expect(listCalls).toBe(2);
  });
});

// The list merges global + project entries when a project is selected, and each entry's
// `matcherGroupIndex` indexes ITS OWN settings.json -- so a write must name the entry's own scope.
describe('HooksPage: global and project entries side by side', () => {
  const project: ProjectContext = { kind: 'project', projectId: 'p1' };
  // Same event AND same index in two different files: the old `event:index` card key collided here.
  const globalEntry = makeEntry({ event: 'PreToolUse', matcherGroupIndex: 0, matcher: 'GlobalOne', scope: 'global' });
  const projectEntry = makeEntry({ event: 'PreToolUse', matcherGroupIndex: 0, matcher: 'ProjectOne', scope: 'project' });

  function listBoth(cmd: string) {
    if (cmd === 'list_hooks') return { value: [globalEntry, projectEntry], skipped: 0 };
  }

  it('renders both entries even though their event and index are identical', async () => {
    const wrapper = await mountPage(listBoth, project);
    expect(wrapper.text()).toContain('GlobalOne');
    expect(wrapper.text()).toContain('ProjectOne');
    expect(wrapper.findAll('button[title="Delete hook"]')).toHaveLength(2);
  });

  it('deleting the GLOBAL entry sends project: null, not the selected project', async () => {
    vi.stubGlobal('confirm', vi.fn().mockReturnValue(true));
    const deletes: unknown[] = [];
    const wrapper = await mountPage((cmd, args) => {
      if (cmd === 'delete_hook') {
        deletes.push(args);
        return 'written';
      }
      return listBoth(cmd);
    }, project);
    // Precondition: without a project actually selected, `project: null` below proves nothing.
    expect(useConfigStore().selectedProject).toEqual(project);
    await wrapper.findAll('button[title="Delete hook"]')[0].trigger('click');
    await flushPromises();
    expect(deletes).toEqual([{ project: null, event: 'PreToolUse', index: 0 }]);
  });

  it('deleting the PROJECT entry sends the selected project', async () => {
    vi.stubGlobal('confirm', vi.fn().mockReturnValue(true));
    const deletes: unknown[] = [];
    const wrapper = await mountPage((cmd, args) => {
      if (cmd === 'delete_hook') {
        deletes.push(args);
        return 'written';
      }
      return listBoth(cmd);
    }, project);
    await wrapper.findAll('button[title="Delete hook"]')[1].trigger('click');
    await flushPromises();
    expect(deletes).toEqual([{ project, event: 'PreToolUse', index: 0 }]);
  });

  it('warns in the confirmation that deleting removes every handler of a multi-handler group', async () => {
    const confirmSpy = vi.fn().mockReturnValue(false);
    vi.stubGlobal('confirm', confirmSpy);
    const multi = makeEntry({
      handlers: [
        { type: 'command', command: 'a.sh', url: null, timeout: null, statusMessage: null, extraKeys: [] },
        { type: 'command', command: 'b.sh', url: null, timeout: null, statusMessage: null, extraKeys: [] },
      ],
    });
    const wrapper = await mountPage((cmd) => (cmd === 'list_hooks' ? { value: [multi], skipped: 0 } : undefined));
    await wrapper.find('button[title="Delete hook"]').trigger('click');
    expect(confirmSpy.mock.calls[0][0]).toContain('all 2 of its handlers');
  });

  it('opening a global entry for editing while a project is selected shows the Global badge (its own scope)', async () => {
    const wrapper = await mountPage(listBoth, project);
    await wrapper.findAll('button[title="Edit hook"]')[0].trigger('click');
    expect(body().find('[role="dialog"]').text()).toContain('Global');
  });
});

// No global error handler exists, so a rejected command used to vanish -- including the realistic
// ones (a busy file, a settings.json that does not parse, which ADR 0009 refuses to overwrite).
describe('HooksPage: failed commands are shown, not swallowed', () => {
  const busy = () => {
    throw { kind: 'busy', message: 'settings.json is in use by another program', path: null };
  };
  const expectedToast = { message: 'settings.json is in use by another program', style: 'error' };

  it('a failed delete shows the cause', async () => {
    vi.stubGlobal('confirm', vi.fn().mockReturnValue(true));
    const wrapper = await mountPage((cmd) => {
      if (cmd === 'list_hooks') return { value: [makeEntry()], skipped: 0 };
      if (cmd === 'delete_hook') return busy();
    });
    await wrapper.find('button[title="Delete hook"]').trigger('click');
    await flushPromises();
    expect(useToastsStore().current).toMatchObject(expectedToast);
  });

  it('a failed import shows the cause', async () => {
    const wrapper = await mountPage((cmd) => {
      if (cmd === 'list_hooks') return { value: [], skipped: 0 };
      if (cmd === 'import_hooks') return busy();
    });
    await wrapper.find('button[title="Import JSON"]').trigger('click');
    await flushPromises();
    expect(useToastsStore().current).toMatchObject(expectedToast);
  });

  it('a failed export shows the cause', async () => {
    const wrapper = await mountPage((cmd) => {
      if (cmd === 'list_hooks') return { value: [], skipped: 0 };
      if (cmd === 'export_hooks') return busy();
    });
    await wrapper.find('button[title="Export JSON"]').trigger('click');
    await flushPromises();
    expect(useToastsStore().current).toMatchObject(expectedToast);
  });

  it('a failed load shows the cause and does not leave the skeleton up forever', async () => {
    const wrapper = await mountPage((cmd) => {
      if (cmd === 'list_hooks') return busy();
    });
    expect(useToastsStore().current).toMatchObject(expectedToast);
    expect(wrapper.text()).toContain('No hooks found');
  });
});

describe('HooksPage: export JSON (Goal)', () => {
  it('calls export_hooks with the current project context and shows a checkmark on success', async () => {
    let args: unknown = null;
    const wrapper = await mountPage((cmd, a) => {
      if (cmd === 'list_hooks') return { value: [], skipped: 0 };
      if (cmd === 'export_hooks') {
        args = a;
        return 'written';
      }
    });

    await wrapper.find('button[title="Export JSON"]').trigger('click');
    await flushPromises();

    expect(args).toEqual({ project: null });
    expect(wrapper.html()).toContain('lucide-check');
  });
});
