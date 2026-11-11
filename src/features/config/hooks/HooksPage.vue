<script setup lang="ts">
// Ported from HooksListView.swift (M3-13, read-only shell) + HookFormView.swift (M3-47, create/
// edit/delete/import wired in here). `HookEntry` has no detail route at all -- unlike Commands/Skills/OutputStyles,
// §1.7 names ConfigItemDetailView as used only by those three kinds -- so this screen never
// imports ConfigItemDetail.vue or ConfigListLayout.vue (that component renders one flat
// grid/list, with no concept of the event-grouped sections this screen needs; Hooks reuses just
// ConfigSkeleton.vue/ConfigEmptyState.vue directly instead).
//
// Grouping order is alphabetical by event name, not HookEvent.allCases' own enum-declaration
// order (HooksListView.swift:45-51): Heimdall's own `HookEntry.event` is a free string, not a
// closed enum ("kept as text so events newer than Heimdall still show up", confirmed directly in
// bindings.ts) -- there is no fixed 18-case catalog left to iterate server-side, and building one
// just to order section headers would be speculative for a read-only screen with no form.
//
// "Copy-as-JSON for hooks" (M3-13's own Goal) stayed a single header-level "Export JSON" button
// calling `export_hooks` (M3-44), not Swift's own per-card clipboard copy of one entry:
// `export_hooks(project)` exports every currently-in-scope entry to one file via a Rust-opened
// save dialog -- confirmed directly in config_write.rs. "Import JSON" (M3-47, same button
// styling) is the same round trip in reverse (`import_hooks`), which the Rust side itself shows a
// native confirmation for before writing (M3-44) -- nothing more is needed here than reloading
// the list on a `written` outcome.
//
// `delete_hook` has no native confirmation of its own (confirmed directly against M3-44's own
// progress-log entry: "Neither delete_hook nor any other command shows this dialog -- not an
// oversight ... this task's own AC1 names only 'hook save and import'") -- M3-51 (delete UI) is
// scoped to session/history/project deletes only (its own Dependencies list M3-46, not M3-47), so
// there is no shared confirmation component yet for this screen to reuse. A plain `window.confirm`
// is the minimal safety net until a real one exists; nothing in this task's own AC calls for more.
import { computed, ref, watch } from 'vue';
import { ipc } from '@/ipc/client';
import { fuzzyMatch } from '@/lib/fuzzyMatch';
import { useToastsStore } from '@/stores/toasts';
import { useConfigStore } from '../store';
import ConfigScreenHeader from '../shared/ConfigScreenHeader.vue';
import ConfigLayoutToolbar from '../shared/ConfigLayoutToolbar.vue';
import ConfigSkeleton from '../shared/ConfigSkeleton.vue';
import ConfigEmptyState from '../shared/ConfigEmptyState.vue';
import ConfigBadge from '../shared/ConfigBadge.vue';
import AppIcon from '@/components/AppIcon.vue';
import HookForm from './HookForm.vue';
import type { HookEntry } from '@/ipc/bindings';

const SCREEN_ID = 'hooks';
const configStore = useConfigStore();
const toastsStore = useToastsStore();

const isFormOpen = ref(false);
const editingEntry = ref<HookEntry | null>(null);

function openCreate() {
  editingEntry.value = null;
  isFormOpen.value = true;
}
function openEdit(entry: HookEntry) {
  editingEntry.value = entry;
  isFormOpen.value = true;
}
// `matcherGroupIndex` indexes the entry's OWN settings.json, and this list merges global + project
// entries, so the delete must name the entry's own scope -- a global entry with a project selected
// is `null`, not the selected project (HookForm.vue's header comment, hazard 2).
async function deleteEntry(entry: HookEntry) {
  const label = firstHandler(entry)?.command ?? firstHandler(entry)?.url ?? entry.event;
  const handlerNote = entry.handlers.length > 1 ? ` and all ${entry.handlers.length} of its handlers` : '';
  if (!window.confirm(`Delete this ${entry.event} hook (${label})${handlerNote}?`)) return;
  const project = entry.scope === 'project' ? configStore.selectedProject : null;
  try {
    const outcome = await ipc.deleteHook(project, entry.event, entry.matcherGroupIndex);
    if (outcome === 'written') await load();
  } catch (error) {
    toastsStore.showError(error, 'Could not delete the hook');
  }
}
async function importJson() {
  try {
    const outcome = await ipc.importHooks(configStore.selectedProject);
    if (outcome === 'written') await load();
  } catch (error) {
    toastsStore.showError(error, 'Could not import hooks');
  }
}

const entries = ref<HookEntry[]>([]);
const isLoading = ref(true);
const filterQuery = ref('');
const exported = ref(false);

async function load() {
  isLoading.value = true;
  try {
    entries.value = (await ipc.listHooks(configStore.selectedProject)).value;
  } catch (error) {
    toastsStore.showError(error, 'Could not load hooks');
  } finally {
    isLoading.value = false;
  }
}
watch(() => configStore.selectedProject, load, { deep: true, immediate: true });

function firstHandler(entry: HookEntry) {
  return entry.handlers[0] as HookEntry['handlers'][number] | undefined;
}
function displayCommand(entry: HookEntry): string {
  const h = firstHandler(entry);
  return h?.command ?? h?.url ?? '';
}

const filteredEntries = computed<HookEntry[]>(() => {
  const q = filterQuery.value.trim();
  if (q === '') return entries.value;
  return entries.value
    .map((entry) => ({ entry, score: fuzzyMatch(`${entry.event} ${entry.matcher ?? ''} ${displayCommand(entry)}`, q)?.score ?? 0 }))
    .filter((r) => r.score > 0)
    .sort((a, b) => b.score - a.score)
    .map((r) => r.entry);
});

const groups = computed<{ event: string; entries: HookEntry[] }[]>(() => {
  const byEvent = new Map<string, HookEntry[]>();
  for (const entry of filteredEntries.value) {
    const list = byEvent.get(entry.event) ?? [];
    list.push(entry);
    byEvent.set(entry.event, list);
  }
  return [...byEvent.entries()].sort(([a], [b]) => a.localeCompare(b)).map(([event, es]) => ({ event, entries: es }));
});

async function exportJson() {
  try {
    const outcome = await ipc.exportHooks(configStore.selectedProject);
    if (outcome !== 'written') return;
    exported.value = true;
    setTimeout(() => (exported.value = false), 1500);
  } catch (error) {
    toastsStore.showError(error, 'Could not export hooks');
  }
}

const emptyMessage = computed(() => (filterQuery.value ? `No hooks match "${filterQuery.value}"` : 'No hooks found'));
const emptyHint = computed(() => (filterQuery.value ? 'Try a different search term' : '~/.claude/settings.json'));
</script>

<template>
  <div class="flex h-full flex-col">
    <div class="flex items-center gap-(--spacing-sm)">
      <ConfigScreenHeader
        icon="arrow.triangle.branch"
        title="Hooks"
        :count="entries.length"
        description="Commands and HTTP calls Claude Code runs on lifecycle events."
        class="flex-1"
      />
      <button
        type="button"
        title="Import JSON"
        class="flex items-center gap-(--spacing-xs) rounded-sm border px-(--spacing-md) py-(--spacing-xs) text-(length:--text-caption) text-(--text-secondary)"
        @click="importJson"
      >
        <AppIcon
          name="square.and.arrow.up"
          size="sm"
        />
        Import JSON
      </button>
      <button
        type="button"
        title="Export JSON"
        class="flex items-center gap-(--spacing-xs) rounded-sm border px-(--spacing-md) py-(--spacing-xs) text-(length:--text-caption) text-(--text-secondary)"
        @click="exportJson"
      >
        <AppIcon
          :name="exported ? 'checkmark' : 'square.and.arrow.down'"
          size="sm"
          :class="exported ? 'text-(--color-green)' : ''"
        />
        Export JSON
      </button>
      <button
        type="button"
        title="New Hook"
        class="mr-(--spacing-xxxl) flex items-center gap-(--spacing-xs) rounded-sm border border-(--accent)/30 bg-(--accent-dim) px-(--spacing-md) py-(--spacing-xs) text-(--accent) text-(length:--text-caption)"
        @click="openCreate"
      >
        <AppIcon
          name="plus"
          size="sm"
        />
        New Hook
      </button>
    </div>
    <ConfigLayoutToolbar
      v-model:filter-query="filterQuery"
      :screen-id="SCREEN_ID"
      filter-placeholder="Find in Hooks…"
      show-project-picker
    />
    <div class="flex-1 overflow-y-auto px-(--spacing-xxxl) py-(--spacing-lg)">
      <ConfigSkeleton
        v-if="isLoading"
        :layout="configStore.layoutFor(SCREEN_ID)"
      />
      <ConfigEmptyState
        v-else-if="filteredEntries.length === 0"
        icon="arrow.triangle.branch"
        :message="emptyMessage"
        :hint="emptyHint"
      />
      <div
        v-else
        class="flex flex-col gap-(--spacing-lg)"
      >
        <div
          v-for="group in groups"
          :key="group.event"
          class="flex flex-col gap-(--spacing-sm)"
        >
          <div class="flex items-center gap-(--spacing-xs) text-(--text-tertiary) text-(length:--text-tiny)">
            <span class="font-semibold">{{ group.event }}</span>
            <span class="rounded-full bg-(--bg-elevated) px-(--spacing-sm) py-px">{{ group.entries.length }}</span>
          </div>
          <div
            :class="configStore.layoutFor(SCREEN_ID) === 'grid' ? 'grid grid-cols-2 gap-(--spacing-md)' : 'flex flex-col gap-(--spacing-sm)'"
          >
            <div
              v-for="entry in group.entries"
              :key="`${entry.event}:${entry.scope}:${entry.matcherGroupIndex}`"
              class="flex flex-col gap-(--spacing-xs) rounded-md border bg-(--bg-card) p-(--spacing-lg)"
            >
              <div class="flex items-center gap-(--spacing-sm)">
                <AppIcon
                  :name="firstHandler(entry)?.type === 'http' ? 'globe' : 'terminal'"
                  size="sm"
                  class="shrink-0 text-(--text-secondary)"
                />
                <button
                  type="button"
                  title="Edit hook"
                  class="min-w-0 flex-1 truncate text-left text-(length:--text-code-small) text-(--text-primary)"
                  @click="openEdit(entry)"
                >
                  {{ displayCommand(entry) }}
                </button>
                <button
                  type="button"
                  title="Delete hook"
                  class="shrink-0 rounded-sm p-(--spacing-xxs) text-(--text-tertiary) hover:text-(--color-red)"
                  @click="deleteEntry(entry)"
                >
                  <AppIcon
                    name="trash"
                    size="sm"
                  />
                </button>
              </div>
              <p
                v-if="entry.matcher"
                class="truncate text-(length:--text-code-small) text-(--text-tertiary)"
              >
                {{ entry.matcher }}
              </p>
              <div class="flex flex-wrap items-center gap-(--spacing-xs)">
                <ConfigBadge :scope="entry.scope" />
                <ConfigBadge
                  v-if="firstHandler(entry)"
                  :text="firstHandler(entry)!.type"
                />
                <ConfigBadge
                  v-if="firstHandler(entry)?.timeout"
                  :text="`${firstHandler(entry)!.timeout}s`"
                />
                <ConfigBadge
                  v-if="entry.handlers.length > 1"
                  :text="`${entry.handlers.length} handlers`"
                />
              </div>
              <p
                v-if="firstHandler(entry)?.statusMessage"
                class="italic text-(length:--text-tiny) text-(--text-tertiary)"
              >
                {{ firstHandler(entry)!.statusMessage }}
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
    <HookForm
      v-if="isFormOpen"
      :editing-entry="editingEntry"
      :on-close="() => (isFormOpen = false)"
      :on-saved="load"
    />
  </div>
</template>
