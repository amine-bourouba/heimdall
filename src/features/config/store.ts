// Shared state across every config screen (M3-11, docs/spec/config-screens.md §1): which project
// scopes commands/skills/output-styles/hooks/MCP servers, and each screen's own persisted
// list/grid layout. Per-screen filter text is NOT here -- it is each screen's own local state
// (ConfigFilterField has no dedicated file; Swift's own `filterQuery` is `@State` per view too,
// §1.2), this store only owns what is genuinely shared across every config screen at once.
import { ref } from 'vue';
import { defineStore } from 'pinia';
import { ipc } from '@/ipc/client';
import { usePrefsStore } from '@/stores/prefs';
import type { Layout, ProjectContext, ProjectSummary } from '@/ipc/bindings';

export const useConfigStore = defineStore('config', () => {
  const prefsStore = usePrefsStore();

  const projects = ref<ProjectSummary[]>([]);
  // AC1: "the choice persists in configProjectPath." `PrefsPatch` has no `configProjectPath`
  // field at all (confirmed directly in prefs.rs/models/prefs.rs) -- `pick_config_project` is the
  // only command that can ever set it, and there is no command to clear it either. Picking a
  // *known* project therefore has nowhere to persist to and is deliberately ephemeral, the same
  // as every other feature's own in-memory `selectedProjectId` -- only a browsed folder survives
  // a restart, via Prefs.
  const selectedProject = ref<ProjectContext | null>(null);

  // Restoring `selectedProject` from Prefs must happen only once per app session: every config
  // screen shares this one store, and re-deriving it from Prefs on every mount (every time the
  // user switches between Commands/Skills/...) would overwrite an in-session "known project"
  // pick with whatever was last browsed, even though nothing changed on disk. Same one-shot guard
  // shape as memory/store.ts's own `subscribed` flag.
  let restored = false;

  async function load() {
    const result = await ipc.listProjects();
    projects.value = result.value;
    if (!restored) {
      restored = true;
      selectedProject.value = prefsStore.prefs?.configProjectPath ? { kind: 'pickedFolder' } : null;
    }
  }

  // ConfigProjectPicker.swift:3-76's own "picked folder's name" -- the last path component, since
  // the raw path itself is only ever shown, never sent back over IPC (ADR 0010).
  function pickedFolderName(): string | null {
    const path = prefsStore.prefs?.configProjectPath;
    if (!path) return null;
    return (
      path
        .split(/[/\\]/)
        .filter(Boolean)
        .pop() ?? path
    );
  }

  function selectProject(context: ProjectContext | null) {
    selectedProject.value = context;
  }

  async function browseForProject() {
    const outcome = await ipc.pickConfigProject();
    if (outcome === 'written') selectedProject.value = { kind: 'pickedFolder' };
  }

  // AC2: "Per-screen layout persists in configLayouts." `configLayouts` is replaced wholesale by
  // `update_prefs` (`prefs.rs`'s own `apply_patch`: `prefs.config_layouts = v`, not a per-key
  // merge -- the exact gotcha `plans/store.ts`'s own `setLayout` already works around for its one
  // hardcoded screen id), so every write here spreads the current map first too.
  function layoutFor(screenId: string): Layout {
    return prefsStore.prefs?.configLayouts?.[screenId] ?? 'grid';
  }

  async function setLayout(screenId: string, next: Layout) {
    const current = prefsStore.prefs?.configLayouts ?? {};
    await prefsStore.update({ configLayouts: { ...current, [screenId]: next } });
  }

  function toggleLayout(screenId: string) {
    return setLayout(screenId, layoutFor(screenId) === 'grid' ? 'list' : 'grid');
  }

  return {
    projects,
    selectedProject,
    load,
    pickedFolderName,
    selectProject,
    browseForProject,
    layoutFor,
    setLayout,
    toggleLayout,
  };
});
