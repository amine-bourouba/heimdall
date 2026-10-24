// Ported from MemoryListView.swift/MemoryDetailView.swift/MemorySourcesStore.swift (M3-04).
// Heimdall's own dedicated commands (`list_memory_sources`/`list_memory_files`/`get_memory_file`,
// M1-44) already separate sources/files/detail cleanly, unlike the Swift reference app's own hand-rolled
// `ClaudeConfigLoader` calls -- and file watching is the real backend watcher (M1-47) plus
// `data-changed` (AC2), not a client-side `FileWatcher` per source the way MemoryListView.swift
// builds one per project-memory-dir and one per custom folder.
import { computed, ref } from 'vue';
import { defineStore } from 'pinia';
import { ipc, onDataChanged } from '@/ipc/client';
import type { MemoryFileDetail, MemoryFileSummary, MemorySource } from '@/ipc/bindings';
import { fuzzyMatch } from '@/lib/fuzzyMatch';

export const useMemoryStore = defineStore('memory', () => {
  const sources = ref<MemorySource[]>([]);
  const files = ref<MemoryFileSummary[]>([]);
  const selectedSourceId = ref<string | null>(null);
  const isLoading = ref(true);
  const query = ref('');
  const selected = ref<MemoryFileDetail | null>(null);
  const isLoadingDetail = ref(false);
  let subscribed = false;

  const totalCount = computed(() => sources.value.reduce((sum, s) => sum + s.fileCount, 0));

  // Goal: "list by source with MEMORY.md first" -- ported from `reloadMemoryFiles`'s own sort
  // (MemoryListView.swift:313-316): isMain files before the rest, then alphabetical.
  const sortedFiles = computed<MemoryFileSummary[]>(() =>
    [...files.value].sort((a, b) => {
      if (a.isMain !== b.isMain) return a.isMain ? -1 : 1;
      return a.name.localeCompare(b.name, undefined, { sensitivity: 'base' });
    }),
  );

  // `MemoryFileSummary` carries no content (confirmed directly against its own generated type --
  // no description/content field, unlike ConfigItemSummary's description for other kinds), so
  // (like M3-03's own plans store) there is nothing to search but the name; re-sorted by score
  // when a query is active, same as `filteredMemoryFiles` not reapplying the isMain-first order.
  const filteredFiles = computed<MemoryFileSummary[]>(() => {
    const q = query.value.trim();
    if (q === '') return sortedFiles.value;
    return sortedFiles.value
      .map((f) => ({ f, score: fuzzyMatch(f.name, q)?.score ?? 0 }))
      .filter((r) => r.score > 0)
      .sort((a, b) => b.score - a.score)
      .map((r) => r.f);
  });

  async function loadFiles() {
    files.value = selectedSourceId.value
      ? await ipc.listMemoryFiles(selectedSourceId.value)
      : (await Promise.all(sources.value.map((s) => ipc.listMemoryFiles(s.id)))).flat();
  }

  async function load() {
    isLoading.value = true;
    sources.value = await ipc.listMemorySources();
    // MemoryListView.swift:296-299: drop a stale selection if its source disappeared (e.g. a
    // removed folder).
    if (selectedSourceId.value && !sources.value.some((s) => s.id === selectedSourceId.value)) {
      selectedSourceId.value = null;
    }
    await loadFiles();
    isLoading.value = false;

    // AC2: "The list refreshes on data-changed for the memory area" -- a file changing on disk
    // (not this store's own add/remove, which has no automatic event, see addFolder/removeFolder
    // below) reaches here the same way session/store.ts's own handleDataChanged does.
    if (!subscribed) {
      subscribed = true;
      await onDataChanged(['memory'], () => void load());
    }
  }

  function selectSource(id: string | null) {
    selectedSourceId.value = id;
    void loadFiles();
  }

  // AC1: "Adding a folder calls add_memory_folder (Rust opens the picker)". Confirmed directly in
  // prefs.rs: add_memory_folder/remove_memory_folder save Prefs and re-arm the live watcher
  // (`reregister_watched_folders`), but emit no data-changed themselves -- adding a folder path is
  // a prefs change, not a filesystem event the watcher would ever see on its own, so this has to
  // reload explicitly rather than wait for AC2's own mechanism to (never) fire.
  async function addFolder() {
    const outcome = await ipc.addMemoryFolder();
    if (outcome === 'written') await load();
  }

  async function removeFolder(sourceId: string) {
    const outcome = await ipc.removeMemoryFolder(sourceId);
    if (outcome === 'written') await load();
  }

  async function open(sourceId: string, fileId: string) {
    isLoadingDetail.value = true;
    selected.value = null;
    selected.value = await ipc.getMemoryFile(sourceId, fileId);
    isLoadingDetail.value = false;
  }

  function close() {
    selected.value = null;
  }

  return {
    sources,
    files,
    selectedSourceId,
    isLoading,
    query,
    selected,
    isLoadingDetail,
    totalCount,
    sortedFiles,
    filteredFiles,
    load,
    selectSource,
    addFolder,
    removeFolder,
    open,
    close,
  };
});
