// M3-07's own AC: "Diffs come from diff_file_versions and render with DiffTable.vue." Ported
// from FileHistoryView.swift.
import { flushPromises, mount } from '@vue/test-utils';
import { mockIPC } from '@tauri-apps/api/mocks';
import { createPinia, setActivePinia } from 'pinia';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import type { DiffResult, FileHistory as FileHistoryContract } from '@/ipc/bindings';
import FileHistory from './FileHistory.vue';

function makeHistory(overrides: Partial<FileHistoryContract> = {}): FileHistoryContract {
  return {
    files: [
      {
        fileName: 'src/App.vue',
        versions: [
          { versionId: 'v1', version: 1, backupTime: '2026-01-01T00:00:00Z' },
          { versionId: 'v2', version: 2, backupTime: '2026-01-02T00:00:00Z' },
        ],
      },
    ],
    ...overrides,
  };
}

const diffResult: DiffResult = { lines: [{ kind: 'add', oldLine: null, newLine: 1, text: 'hello' }], truncated: false, maxLineNumber: 1 };

let mounted: ReturnType<typeof mount>[] = [];

async function mountView(history: FileHistoryContract, ipcHandler?: (cmd: string, args: unknown) => unknown) {
  mockIPC((cmd, args) => {
    if (cmd === 'get_file_history') {
      expect(args).toEqual({ projectId: 'p1', sessionId: 's1' });
      return history;
    }
    if (ipcHandler) return ipcHandler(cmd, args);
    throw new Error(`unexpected command: ${cmd}`);
  });
  const wrapper = mount(FileHistory, { props: { projectId: 'p1', sessionId: 's1' }, attachTo: document.body });
  mounted.push(wrapper);
  await flushPromises();
  return wrapper;
}

beforeEach(() => {
  document.body.innerHTML = '';
  setActivePinia(createPinia()); // AppIcon reads usePrefsStore(); the close button reads useSessionStore().
});

afterEach(() => {
  mounted.forEach((w) => w.unmount());
  mounted = [];
});

describe('FileHistory: empty and loading states', () => {
  it('shows the empty state when no files changed', async () => {
    const wrapper = await mountView(makeHistory({ files: [] }));
    expect(wrapper.text()).toContain('No File History');
  });
});

describe('FileHistory: auto-selection (FileHistoryView.swift:347-350/191)', () => {
  it('auto-selects the first file and its latest version, diffing against the previous one', async () => {
    let diffArgs: unknown = null;
    const wrapper = await mountView(makeHistory(), (cmd, args) => {
      if (cmd === 'diff_file_versions') {
        diffArgs = args;
        return diffResult;
      }
      throw new Error(`unexpected command: ${cmd}`);
    });

    expect(diffArgs).toEqual({ sessionId: 's1', fromVersionId: 'v1', toVersionId: 'v2' });
    expect(wrapper.text()).toContain('App.vue');
    expect(wrapper.text()).toContain('hello'); // rendered via DiffTable
    expect(wrapper.text()).toContain('v2');
  });
});

describe('FileHistory: first version has no predecessor (AC1)', () => {
  it('diffs a lone version against empty content via diff_lines, not diff_file_versions', async () => {
    let diffLinesArgs: unknown = null;
    const history = makeHistory({ files: [{ fileName: 'only.txt', versions: [{ versionId: 'v1', version: 1, backupTime: '2026-01-01T00:00:00Z' }] }] });
    const wrapper = await mountView(history, (cmd, args) => {
      if (cmd === 'get_file_version') {
        expect(args).toEqual({ sessionId: 's1', versionId: 'v1' });
        return { versionId: 'v1', content: 'full content' };
      }
      if (cmd === 'diff_lines') {
        diffLinesArgs = args;
        return diffResult;
      }
      throw new Error(`unexpected command: ${cmd}`);
    });

    expect(diffLinesArgs).toEqual({ oldText: '', newText: 'full content' });
    expect(wrapper.text()).toContain('hello');
  });
});

describe('FileHistory: selecting a file or version re-fetches the diff', () => {
  it('clicking an earlier version diffs it against its own predecessor', async () => {
    const calls: unknown[] = [];
    const wrapper = await mountView(makeHistory(), (cmd, args) => {
      if (cmd === 'get_file_version') {
        return { versionId: 'v1', content: 'v1 content' };
      }
      if (cmd === 'diff_lines' || cmd === 'diff_file_versions') {
        calls.push([cmd, args]);
        return diffResult;
      }
      throw new Error(`unexpected command: ${cmd}`);
    });
    calls.length = 0; // drop the auto-selected latest-version diff from setup

    // The version chip has no title attr in this port (unlike Swift's own .help) -- select it
    // by its visible "v1" label instead.
    const chip = wrapper.findAll('button').find((b) => b.text().includes('v1'));
    await chip!.trigger('click');
    await flushPromises();

    expect(calls).toContainEqual(['diff_lines', { oldText: '', newText: 'v1 content' }]);
  });

  it('clicking a different file loads its own history and re-selects its latest version', async () => {
    const history = makeHistory({
      files: [
        ...makeHistory().files,
        { fileName: 'other.txt', versions: [{ versionId: 'o1', version: 1, backupTime: '2026-01-03T00:00:00Z' }] },
      ],
    });
    let lastVersionContentArgs: unknown = null;
    const wrapper = await mountView(history, (cmd, args) => {
      if (cmd === 'get_file_version') {
        lastVersionContentArgs = args;
        return { versionId: (args as { versionId: string }).versionId, content: 'x' };
      }
      if (cmd === 'diff_lines' || cmd === 'diff_file_versions') return diffResult;
      throw new Error(`unexpected command: ${cmd}`);
    });

    const otherFileButton = wrapper.findAll('button').find((b) => b.text().includes('other.txt'));
    await otherFileButton!.trigger('click');
    await flushPromises();

    expect(lastVersionContentArgs).toEqual({ sessionId: 's1', versionId: 'o1' });
  });
});

describe('FileHistory: close button', () => {
  it('clicking Close turns off isShowingFileHistory', async () => {
    const { useSessionStore } = await import('./store');
    const wrapper = await mountView(makeHistory(), () => diffResult);
    useSessionStore().isShowingFileHistory = true;

    await wrapper.find('button[title="Close"]').trigger('click');

    expect(useSessionStore().isShowingFileHistory).toBe(false);
  });
});
