<script setup lang="ts">
// Ported from SubAgentsListView.swift (M3-14 read-only shell + M3-49 actions): create, edit,
// duplicate, export, delete and import. Built-in agents stay read-only: they get Duplicate only
// (spec §9.1 gives them Export too, but `export_agent` is file-based -- `export_agent_json` reads
// `agents/<slug>.md` -- and a built-in has no file, so offering it would only ever fail).
//
// Built-in agents are already synthesised server-side: `agents::load` (confirmed directly) chains
// `SubAgent::built_in()` ahead of the loaded custom agents, so `listConfigItems('agent', null)`
// alone -- always `null`, matching plans/store.ts's own precedent: agents have no project scope
// at all, `agents::load` takes no project parameter -- returns both groups in one call; this
// screen only needs to split the flat list back into its two sections by `scope`.
//
// `ConfigItemSummary` carries no per-agent icon field at all (only name/description/scope/meta) --
// Swift's own `SubAgent.icon` (a distinct SF Symbol per built-in, "person.crop.circle.badge.plus"
// for every custom one) has nothing to read client-side, so every card shares one icon instead of
// guessing per-name icons the API was never asked to expose.
//
// The summary also lacks the prompt, so Edit and Duplicate fetch the full item first
// (`get_config_item`). Delete has no native confirmation (`delete_config_item` shows none, M3-44),
// and M3-51's delete UI is scoped to session/history/project deletes -- a plain `window.confirm`
// is the minimal safety net, as on the hooks screen.
import { computed, ref } from 'vue';
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
import SubAgentForm from './SubAgentForm.vue';
import { agentFromDetail, duplicateOf } from './agentModel';
import type { AgentFormModel } from './agentModel';
import type { ConfigItemSummary } from '@/ipc/bindings';

const SCREEN_ID = 'subAgents';
const configStore = useConfigStore();
const toastsStore = useToastsStore();

const items = ref<ConfigItemSummary[]>([]);
const isLoading = ref(true);
const filterQuery = ref('');

async function load() {
  isLoading.value = true;
  try {
    items.value = (await ipc.listConfigItems('agent', null)).value;
  } catch (error) {
    toastsStore.showError(error, 'Could not load agents');
  } finally {
    isLoading.value = false;
  }
}
void load();

const filteredItems = computed<ConfigItemSummary[]>(() => {
  const q = filterQuery.value.trim();
  if (q === '') return items.value;
  return items.value
    .map((item) => ({ item, score: fuzzyMatch(`${item.name} ${item.description}`, q)?.score ?? 0 }))
    .filter((r) => r.score > 0)
    .sort((a, b) => b.score - a.score)
    .map((r) => r.item);
});

const builtIn = computed(() => filteredItems.value.filter((i) => i.scope === 'builtIn'));
const custom = computed(() => filteredItems.value.filter((i) => i.scope !== 'builtIn'));
// Every custom agent's file stem, filter or no filter: what the form needs to predict whether a
// new name would clash with an existing file.
const existingSlugs = computed(() => items.value.filter((i) => i.scope !== 'builtIn').map((i) => i.ref.slug));

// SubAgentFormView.swift:217's own `AgentColor` Debug-formatted into meta.color by agent_summary
// (`format!("{c:?}")`, confirmed directly in config_read.rs) -- PascalCase Rust variant names, one
// class-pair entry per colour. No colour set (every built-in, and any custom agent with an
// unrecognised value) falls back to orange, SubAgentsListView.swift's own stated default. Written
// out as complete, static class strings (not built from an interpolated colour name) because
// Tailwind's build-time scanner only generates CSS for class names it can find literally in the
// source text -- a template literal like `` `bg-(--color-${x})/15` `` would never match anything.
const AGENT_COLOR_CLASSES: Record<string, { fg: string; bgIcon: string; bgBadge: string }> = {
  Orange: { fg: 'text-(--color-orange)', bgIcon: 'bg-(--color-orange)/15', bgBadge: 'bg-(--color-orange)/10' },
  Red: { fg: 'text-(--color-red)', bgIcon: 'bg-(--color-red)/15', bgBadge: 'bg-(--color-red)/10' },
  Blue: { fg: 'text-(--color-blue)', bgIcon: 'bg-(--color-blue)/15', bgBadge: 'bg-(--color-blue)/10' },
  Green: { fg: 'text-(--color-green)', bgIcon: 'bg-(--color-green)/15', bgBadge: 'bg-(--color-green)/10' },
  Purple: { fg: 'text-(--color-purple)', bgIcon: 'bg-(--color-purple)/15', bgBadge: 'bg-(--color-purple)/10' },
  Teal: { fg: 'text-(--color-teal)', bgIcon: 'bg-(--color-teal)/15', bgBadge: 'bg-(--color-teal)/10' },
};
function agentColorClasses(meta: Record<string, string>) {
  return AGENT_COLOR_CLASSES[meta.color] ?? AGENT_COLOR_CLASSES.Orange;
}

function tools(meta: Record<string, string>): string[] {
  return meta.tools ? meta.tools.split(',').filter(Boolean) : [];
}

const emptyMessage = computed(() => `No agents match "${filterQuery.value}"`);

// --- Actions ---------------------------------------------------------------------------------

const isFormOpen = ref(false);
const editing = ref<AgentFormModel | null>(null);
// A second click while the full item is still being fetched must not queue a second form.
const isBusy = ref(false);

function openCreate() {
  editing.value = null;
  isFormOpen.value = true;
}

async function fetchModel(item: ConfigItemSummary): Promise<AgentFormModel> {
  return agentFromDetail(await ipc.getConfigItem(item.ref));
}

async function openEdit(item: ConfigItemSummary) {
  if (isBusy.value) return;
  isBusy.value = true;
  try {
    editing.value = await fetchModel(item);
    isFormOpen.value = true;
  } catch (error) {
    toastsStore.showError(error, 'Could not open the agent');
  } finally {
    isBusy.value = false;
  }
}

async function duplicate(item: ConfigItemSummary) {
  if (isBusy.value) return;
  isBusy.value = true;
  try {
    const input = duplicateOf(await fetchModel(item));
    // A new file: `save_agent` never overwrites another agent (a clash becomes "-2").
    if ((await ipc.saveAgent(input, null)) === 'written') {
      toastsStore.show(`Created ${input.name}`);
      await load();
    }
  } catch (error) {
    toastsStore.showError(error, 'Could not duplicate the agent');
  } finally {
    isBusy.value = false;
  }
}

async function exportAgent(item: ConfigItemSummary) {
  try {
    if ((await ipc.exportAgent(item.ref.slug)) === 'written') toastsStore.show(`Exported ${item.name}`);
  } catch (error) {
    toastsStore.showError(error, 'Could not export the agent');
  }
}

async function deleteAgent(item: ConfigItemSummary) {
  if (!window.confirm(`Delete the agent "${item.name}"? Its file will be removed.`)) return;
  try {
    if ((await ipc.deleteConfigItem(item.ref)) === 'written') {
      toastsStore.show(`Deleted ${item.name}`, { style: 'info' });
      await load();
    }
  } catch (error) {
    toastsStore.showError(error, 'Could not delete the agent');
  }
}

async function importAgent() {
  try {
    if ((await ipc.importAgent()) === 'written') {
      toastsStore.show('Imported agent');
      await load();
    }
  } catch (error) {
    // `import_agent` rejects invalid JSON with its own message (Swift showed a bare "Invalid agent JSON").
    toastsStore.showError(error, 'Could not import the agent');
  }
}
</script>

<template>
  <div class="flex h-full flex-col">
    <div class="flex items-center gap-(--spacing-sm)">
      <ConfigScreenHeader
        icon="person.2.fill"
        title="Sub-agents"
        :count="items.length"
        description="Specialized agents Claude Code can delegate tasks to."
        class="flex-1"
      />
      <button
        type="button"
        title="Import Agent from JSON"
        class="flex items-center gap-(--spacing-xs) rounded-sm border px-(--spacing-md) py-(--spacing-xs) text-(length:--text-caption) text-(--text-secondary)"
        @click="importAgent"
      >
        <AppIcon
          name="square.and.arrow.up"
          size="sm"
        />
        Import JSON
      </button>
      <button
        type="button"
        title="New Agent"
        class="mr-(--spacing-xxxl) flex items-center gap-(--spacing-xs) rounded-sm border border-(--accent)/30 bg-(--accent-dim) px-(--spacing-md) py-(--spacing-xs) text-(--accent) text-(length:--text-caption)"
        @click="openCreate"
      >
        <AppIcon
          name="plus"
          size="sm"
        />
        New Agent
      </button>
    </div>
    <ConfigLayoutToolbar
      v-model:filter-query="filterQuery"
      :screen-id="SCREEN_ID"
      filter-placeholder="Find in Sub-agents…"
    />
    <div class="flex-1 overflow-y-auto px-(--spacing-xxxl) py-(--spacing-lg)">
      <ConfigSkeleton
        v-if="isLoading"
        :layout="configStore.layoutFor(SCREEN_ID)"
      />
      <ConfigEmptyState
        v-else-if="filteredItems.length === 0"
        icon="person.2.fill"
        :message="emptyMessage"
        hint="Try a different search term"
      />
      <div
        v-else
        class="flex flex-col gap-(--spacing-lg)"
      >
        <div
          v-for="section in [{ label: 'Built-in', items: builtIn }, { label: 'Custom', items: custom }]"
          :key="section.label"
        >
          <template v-if="section.items.length > 0">
            <p class="mb-(--spacing-sm) font-semibold text-(--text-tertiary) text-(length:--text-tiny)">
              {{ section.label }}
            </p>
            <div
              :class="configStore.layoutFor(SCREEN_ID) === 'grid' ? 'grid grid-cols-2 gap-(--spacing-md)' : 'flex flex-col gap-(--spacing-sm)'"
            >
              <div
                v-for="item in section.items"
                :key="`${item.ref.kind}:${item.ref.slug}`"
                class="flex flex-col gap-(--spacing-xs) rounded-md border bg-(--bg-card) p-(--spacing-lg)"
              >
                <div class="flex items-center gap-(--spacing-sm)">
                  <div
                    class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md"
                    :class="agentColorClasses(item.meta).bgIcon"
                  >
                    <AppIcon
                      name="person.2.fill"
                      size="sm"
                      :class="agentColorClasses(item.meta).fg"
                    />
                  </div>
                  <p
                    v-if="item.scope === 'builtIn'"
                    class="min-w-0 flex-1 truncate text-(length:--text-body-medium) text-(--text-primary)"
                  >
                    {{ item.name }}
                  </p>
                  <button
                    v-else
                    type="button"
                    title="Edit agent"
                    class="min-w-0 flex-1 truncate text-left text-(length:--text-body-medium) text-(--text-primary)"
                    @click="openEdit(item)"
                  >
                    {{ item.name }}
                  </button>
                  <ConfigBadge
                    v-if="item.meta.model"
                    :text="item.meta.model"
                    :fg-class="agentColorClasses(item.meta).fg"
                    :bg-class="agentColorClasses(item.meta).bgBadge"
                  />
                  <button
                    type="button"
                    title="Duplicate agent"
                    class="shrink-0 rounded-sm p-(--spacing-xxs) text-(--text-tertiary) hover:text-(--text-primary)"
                    @click="duplicate(item)"
                  >
                    <AppIcon
                      name="doc.on.doc"
                      size="sm"
                    />
                  </button>
                  <template v-if="item.scope !== 'builtIn'">
                    <button
                      type="button"
                      title="Export agent as JSON"
                      class="shrink-0 rounded-sm p-(--spacing-xxs) text-(--text-tertiary) hover:text-(--text-primary)"
                      @click="exportAgent(item)"
                    >
                      <AppIcon
                        name="square.and.arrow.down"
                        size="sm"
                      />
                    </button>
                    <button
                      type="button"
                      title="Delete agent"
                      class="shrink-0 rounded-sm p-(--spacing-xxs) text-(--text-tertiary) hover:text-(--color-red)"
                      @click="deleteAgent(item)"
                    >
                      <AppIcon
                        name="trash"
                        size="sm"
                      />
                    </button>
                  </template>
                </div>
                <p
                  v-if="item.description"
                  class="line-clamp-2 text-(length:--text-small) text-(--text-tertiary)"
                >
                  {{ item.description }}
                </p>
                <div
                  v-if="tools(item.meta).length > 0"
                  class="flex flex-wrap items-center gap-(--spacing-xs)"
                >
                  <ConfigBadge
                    v-for="tool in tools(item.meta)"
                    :key="tool"
                    :text="tool"
                  />
                </div>
              </div>
            </div>
          </template>
        </div>
      </div>
    </div>
    <SubAgentForm
      v-if="isFormOpen"
      :editing="editing"
      :existing-slugs="existingSlugs"
      :on-close="() => (isFormOpen = false)"
      :on-saved="load"
    />
  </div>
</template>
