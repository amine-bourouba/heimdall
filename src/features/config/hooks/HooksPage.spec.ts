// M3-13's own AC: "Hooks group by event with matcher and handler summaries..." / "No mutating
// controls yet (M3-47, M3-50)." Goal: "...including copy-as-JSON for hooks."
import { flushPromises, mount } from '@vue/test-utils';
import { mockIPC } from '@tauri-apps/api/mocks';
import { createPinia, setActivePinia } from 'pinia';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import type { HookEntry } from '@/ipc/bindings';
import HooksPage from './HooksPage.vue';

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

async function mountPage(ipcHandler: (cmd: string, args: unknown) => unknown) {
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
  return wrapper;
}

beforeEach(() => {
  document.body.innerHTML = '';
});

afterEach(() => {
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

describe('HooksPage: no mutating controls yet (AC)', () => {
  it('renders no Add button', async () => {
    const wrapper = await mountPage((cmd) => {
      if (cmd === 'list_hooks') return { value: [makeEntry()], skipped: 0 };
    });
    expect(wrapper.find('button[title="Add"]').exists()).toBe(false);
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
