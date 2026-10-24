// M3-04's own AC: "Adding a folder calls add_memory_folder... removing calls
// remove_memory_folder" and "The list refreshes on data-changed for the memory area."
import { mockIPC } from '@tauri-apps/api/mocks';
import { flushPromises } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { describe, expect, it } from 'vitest';
import { events } from '@/ipc/bindings';
import type { MemoryFileDetail, MemoryFileSummary, MemorySource } from '@/ipc/bindings';
import { useMemoryStore } from './store';

function makeSource(overrides: Partial<MemorySource> = {}): MemorySource {
  return { id: 'project:p1', label: 'my-project', kind: 'project', fileCount: 1, ...overrides };
}

function makeFile(overrides: Partial<MemoryFileSummary> = {}): MemoryFileSummary {
  return { id: 'f1', sourceId: 'project:p1', name: 'MEMORY', filename: 'MEMORY.md', isMain: true, ...overrides };
}

function makeDetail(overrides: Partial<MemoryFileDetail> = {}): MemoryFileDetail {
  return { id: 'f1', sourceId: 'project:p1', name: 'MEMORY', filename: 'MEMORY.md', isMain: true, content: '# Notes', displayPath: '~/.claude/projects/p1/memory/MEMORY.md', ...overrides };
}

describe('memory store: loading sources and files', () => {
  it('load() fetches every source, then every source\'s files when none is selected', async () => {
    setActivePinia(createPinia());
    mockIPC((cmd, args) => {
      if (cmd === 'list_memory_sources') return [makeSource({ id: 'project:p1' }), makeSource({ id: 'folder:0', kind: 'folder', label: 'Notes' })];
      if (cmd === 'list_memory_files') {
        const sourceId = (args as { sourceId: string }).sourceId;
        return sourceId === 'project:p1' ? [makeFile({ id: 'f1', sourceId: 'project:p1' })] : [makeFile({ id: 'f2', sourceId: 'folder:0', isMain: false, name: 'notes' })];
      }
      throw new Error(`unexpected command: ${cmd}`);
    }, { shouldMockEvents: true });
    const store = useMemoryStore();

    await store.load();

    expect(store.sources.length).toBe(2);
    expect(store.files.map((f) => f.id).sort()).toEqual(['f1', 'f2']);
    expect(store.isLoading).toBe(false);
  });

  it('selectSource() loads only that source\'s files', async () => {
    setActivePinia(createPinia());
    const calls: string[] = [];
    mockIPC((cmd, args) => {
      if (cmd === 'list_memory_sources') return [makeSource({ id: 'project:p1' }), makeSource({ id: 'folder:0', kind: 'folder' })];
      if (cmd === 'list_memory_files') {
        calls.push((args as { sourceId: string }).sourceId);
        return [makeFile({ sourceId: (args as { sourceId: string }).sourceId })];
      }
      throw new Error(`unexpected command: ${cmd}`);
    }, { shouldMockEvents: true });
    const store = useMemoryStore();
    await store.load();
    calls.length = 0;

    store.selectSource('project:p1');
    await flushPromises();

    expect(calls).toEqual(['project:p1']);
  });

  it('drops a stale source selection if that source disappears on reload', async () => {
    setActivePinia(createPinia());
    let sourcesGone = false;
    mockIPC((cmd) => {
      if (cmd === 'list_memory_sources') return sourcesGone ? [] : [makeSource({ id: 'folder:0', kind: 'folder' })];
      if (cmd === 'list_memory_files') return [];
      throw new Error(`unexpected command: ${cmd}`);
    }, { shouldMockEvents: true });
    const store = useMemoryStore();
    await store.load();
    store.selectSource('folder:0');
    sourcesGone = true;

    await store.load();

    expect(store.selectedSourceId).toBeNull();
  });
});

describe('memory store: sorting and search (Goal: "MEMORY.md first")', () => {
  it('sortedFiles puts isMain files before the rest, then alphabetically', async () => {
    setActivePinia(createPinia());
    mockIPC((cmd) => {
      if (cmd === 'list_memory_sources') return [makeSource()];
      if (cmd === 'list_memory_files') return [makeFile({ id: 'z', name: 'Zebra', isMain: false }), makeFile({ id: 'main', name: 'MEMORY', isMain: true }), makeFile({ id: 'a', name: 'Apple', isMain: false })];
      throw new Error(`unexpected command: ${cmd}`);
    }, { shouldMockEvents: true });
    const store = useMemoryStore();
    await store.load();

    expect(store.sortedFiles.map((f) => f.id)).toEqual(['main', 'a', 'z']);
  });

  it('filteredFiles ranks by fuzzy score on the name, ignoring sort order', async () => {
    setActivePinia(createPinia());
    mockIPC((cmd) => {
      if (cmd === 'list_memory_sources') return [makeSource()];
      if (cmd === 'list_memory_files') return [makeFile({ id: 'main', name: 'MEMORY', isMain: true }), makeFile({ id: 'deploy', name: 'deployment notes', isMain: false })];
      throw new Error(`unexpected command: ${cmd}`);
    }, { shouldMockEvents: true });
    const store = useMemoryStore();
    await store.load();

    store.query = 'deploy';

    expect(store.filteredFiles.map((f) => f.id)).toEqual(['deploy']);
  });
});

describe('memory store: add/remove folder (AC: calls add_memory_folder/remove_memory_folder)', () => {
  it('addFolder() calls add_memory_folder and reloads only when the outcome is "written"', async () => {
    setActivePinia(createPinia());
    let sourceAdded = false;
    mockIPC((cmd) => {
      if (cmd === 'add_memory_folder') {
        sourceAdded = true;
        return 'written';
      }
      if (cmd === 'list_memory_sources') return sourceAdded ? [makeSource({ id: 'folder:0', kind: 'folder' })] : [];
      if (cmd === 'list_memory_files') return [];
      throw new Error(`unexpected command: ${cmd}`);
    }, { shouldMockEvents: true });
    const store = useMemoryStore();
    await store.load();
    expect(store.sources.length).toBe(0);

    await store.addFolder();

    expect(store.sources.length).toBe(1);
  });

  it('addFolder() does not reload when the picker is cancelled', async () => {
    setActivePinia(createPinia());
    let listCalls = 0;
    mockIPC((cmd) => {
      if (cmd === 'add_memory_folder') return 'cancelled';
      if (cmd === 'list_memory_sources') {
        listCalls += 1;
        return [];
      }
      if (cmd === 'list_memory_files') return [];
      throw new Error(`unexpected command: ${cmd}`);
    }, { shouldMockEvents: true });
    const store = useMemoryStore();
    await store.load();
    const callsAfterLoad = listCalls;

    await store.addFolder();

    expect(listCalls).toBe(callsAfterLoad); // no second list_memory_sources call
  });

  it('removeFolder() calls remove_memory_folder with the source id and reloads on success', async () => {
    setActivePinia(createPinia());
    let removed = false;
    mockIPC((cmd, args) => {
      if (cmd === 'remove_memory_folder') {
        expect(args).toEqual({ sourceId: 'folder:0' });
        removed = true;
        return 'written';
      }
      if (cmd === 'list_memory_sources') return removed ? [] : [makeSource({ id: 'folder:0', kind: 'folder' })];
      if (cmd === 'list_memory_files') return [];
      throw new Error(`unexpected command: ${cmd}`);
    }, { shouldMockEvents: true });
    const store = useMemoryStore();
    await store.load();
    expect(store.sources.length).toBe(1);

    await store.removeFolder('folder:0');

    expect(store.sources.length).toBe(0);
  });
});

describe('memory store: open/close', () => {
  it('open() fetches the full detail via getMemoryFile', async () => {
    setActivePinia(createPinia());
    mockIPC((cmd, args) => {
      if (cmd === 'get_memory_file') {
        expect(args).toEqual({ sourceId: 'project:p1', fileId: 'f1' });
        return makeDetail();
      }
      throw new Error(`unexpected command: ${cmd}`);
    });
    const store = useMemoryStore();

    await store.open('project:p1', 'f1');

    expect(store.selected?.content).toBe('# Notes');
  });

  it('close() clears the selected detail', async () => {
    setActivePinia(createPinia());
    mockIPC(() => makeDetail());
    const store = useMemoryStore();
    await store.open('project:p1', 'f1');

    store.close();

    expect(store.selected).toBeNull();
  });
});

describe('memory store: refreshes on data-changed for the memory area (AC2)', () => {
  it('a real data-changed event for "memory" triggers a reload', async () => {
    setActivePinia(createPinia());
    let version = 0;
    mockIPC((cmd) => {
      if (cmd === 'list_memory_sources') return [makeSource({ fileCount: version })];
      if (cmd === 'list_memory_files') return version === 0 ? [] : [makeFile()];
      throw new Error(`unexpected command: ${cmd}`);
    }, { shouldMockEvents: true });
    const store = useMemoryStore();
    await store.load();
    expect(store.files.length).toBe(0);

    version = 1;
    await events.dataChanged.emit([{ area: 'memory', key: null }]);
    await flushPromises();

    expect(store.files.length).toBe(1);
  });

  it('a data-changed event for an unrelated area does not reload', async () => {
    setActivePinia(createPinia());
    let calls = 0;
    mockIPC((cmd) => {
      if (cmd === 'list_memory_sources') {
        calls += 1;
        return [];
      }
      if (cmd === 'list_memory_files') return [];
      throw new Error(`unexpected command: ${cmd}`);
    }, { shouldMockEvents: true });
    const store = useMemoryStore();
    await store.load();
    const callsAfterLoad = calls;

    await events.dataChanged.emit([{ area: 'todos', key: null }]);
    await flushPromises();

    expect(calls).toBe(callsAfterLoad);
  });
});
