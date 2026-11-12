<script setup lang="ts">
// Ported from ModelsListView.swift (M3-13 read-only shell + M3-50 actions). The per-card buttons
// follow Swift's own rules (ModelsListView.swift:304-356, config-screens.md §7): "Set as Default"
// (hidden when the model already is the default), "Set as Project Default" (only when a project is
// picked and it is not already the project default) and "Clear Project Override" (only on the project
// default).
//
// **What is written is the model's `id`, never its display name** (port-plan C-27): Swift writes the
// display name ("Opus 5") into Claude Code's `model` setting, which takes an id or alias. A model with
// no id (a discovered one naming no known family, such as `<synthetic>`) gets no "set" buttons at all.
// The badges still compare display names, and the Rust reader maps what was written back to the same
// name (a test pins that for every curated id), so the badge moves to the model just set.
//
// No project picker widget here (config-screens.md §1.2: not on this toolbar in Swift): the project
// comes from the shared `configStore.selectedProject`, whatever another screen's picker set -- which
// is why a hint says where to pick one. `list_models` takes the same project, so the "Project Default"
// badge and the buttons agree.
//
// No skeleton and no "items missing" empty state (§1.4/§1.5: the curated catalog is never truly
// empty) -- `isLoading` is not tracked at all; ConfigListLayout always shows the filter-miss flavour of
// ConfigEmptyState.vue instead of a loading skeleton for the brief window before the first list_models.
import { computed, ref, watch } from 'vue';
import { ipc } from '@/ipc/client';
import { fuzzyMatch } from '@/lib/fuzzyMatch';
import { useToastsStore } from '@/stores/toasts';
import { useConfigStore } from '../store';
import ConfigScreenHeader from '../shared/ConfigScreenHeader.vue';
import ConfigLayoutToolbar from '../shared/ConfigLayoutToolbar.vue';
import ConfigListLayout from '../shared/ConfigListLayout.vue';
import ConfigBadge from '../shared/ConfigBadge.vue';
import AppIcon from '@/components/AppIcon.vue';
import type { ClaudeModelInfo, ModelsView } from '@/ipc/bindings';

const SCREEN_ID = 'models';
const configStore = useConfigStore();
const toastsStore = useToastsStore();

const view = ref<ModelsView>({ models: [], defaultModel: '', projectModel: null });
const filterQuery = ref('');
// One write at a time: a second click while a settings.json write is in flight must not queue another.
const isBusy = ref(false);

async function load() {
  try {
    view.value = await ipc.listModels(configStore.selectedProject);
  } catch (error) {
    toastsStore.showError(error, 'Could not load models');
  }
}
watch(() => configStore.selectedProject, load, { deep: true, immediate: true });

const filteredModels = computed<ClaudeModelInfo[]>(() => {
  const q = filterQuery.value.trim();
  if (q === '') return view.value.models;
  return view.value.models
    .map((model) => ({ model, score: fuzzyMatch(model.displayName, q)?.score ?? 0 }))
    .filter((r) => r.score > 0)
    .sort((a, b) => b.score - a.score)
    .map((r) => r.model);
});

const hasProject = computed(() => configStore.selectedProject !== null);
const isDefault = (model: ClaudeModelInfo) => model.displayName === view.value.defaultModel;
const isProjectDefault = (model: ClaudeModelInfo) => view.value.projectModel !== null && model.displayName === view.value.projectModel;
/** Which buttons a card shows. `id` is what gets written, so no id means nothing can be set. */
function buttons(model: ClaudeModelInfo) {
  const writable = model.id !== '';
  return {
    setDefault: writable && !isDefault(model),
    setProject: writable && hasProject.value && !isProjectDefault(model),
    clear: hasProject.value && isProjectDefault(model),
  };
}

/** Runs one write, then re-reads the view so the badges follow; says so, or says why not. */
async function write(message: string, failure: string, action: () => Promise<unknown>) {
  if (isBusy.value) return;
  isBusy.value = true;
  try {
    await action();
    toastsStore.show(message);
    await load();
  } catch (error) {
    toastsStore.showError(error, failure);
  } finally {
    isBusy.value = false;
  }
}

function setDefault(model: ClaudeModelInfo) {
  return write(`Set ${model.displayName} as default model`, 'Could not set the default model', () => ipc.setDefaultModel(model.id));
}

function setProjectDefault(model: ClaudeModelInfo) {
  const project = configStore.selectedProject;
  if (!project) return;
  return write(`Set ${model.displayName} as project default`, 'Could not set the project model', () => ipc.setProjectModel(project, model.id));
}

function clearProjectDefault() {
  const project = configStore.selectedProject;
  if (!project) return;
  return write('Cleared project model override', 'Could not clear the project model', () => ipc.setProjectModel(project, null));
}

const emptyMessage = computed(() => `No models match "${filterQuery.value}"`);
</script>

<template>
  <div class="flex h-full flex-col">
    <ConfigScreenHeader
      icon="brain.fill"
      title="Models"
      :count="view.models.length"
      description="Claude models available to Claude Code."
    />
    <ConfigLayoutToolbar
      v-model:filter-query="filterQuery"
      :screen-id="SCREEN_ID"
      filter-placeholder="Find in Models…"
    />
    <div class="flex-1 overflow-y-auto px-(--spacing-xxxl) py-(--spacing-lg)">
      <p class="mb-(--spacing-lg) rounded-md bg-(--bg-elevated) p-(--spacing-md) text-(length:--text-small) text-(--text-tertiary)">
        The <code>ANTHROPIC_MODEL</code> environment variable overrides the on-disk default shown here.
        <span v-if="!hasProject"> To set a project default, first pick a project on the Commands, Skills or Output Styles screen.</span>
      </p>
      <ConfigListLayout
        :layout="configStore.layoutFor(SCREEN_ID)"
        :is-loading="false"
        :items="filteredModels"
        empty-icon="brain.fill"
        :empty-message="emptyMessage"
        empty-hint="Try a different search term"
      >
        <template #default="{ item }">
          <div class="flex flex-col gap-(--spacing-xs) rounded-md border bg-(--bg-card) p-(--spacing-lg)">
            <div class="flex items-start gap-(--spacing-sm)">
              <p class="min-w-0 flex-1 text-(length:--text-body-medium) text-(--text-primary)">
                {{ item.displayName }}
              </p>
              <ConfigBadge
                v-if="isDefault(item)"
                text="Global Default"
                fg-class="text-(--accent)"
                bg-class="bg-(--accent-dim)"
              />
              <ConfigBadge
                v-if="isProjectDefault(item)"
                text="Project Default"
                fg-class="text-(--color-green)"
                bg-class="bg-(--color-green)/10"
              />
            </div>
            <p
              v-if="item.description"
              class="text-(length:--text-small) text-(--text-tertiary)"
            >
              {{ item.description }}
            </p>
            <ul
              v-if="item.strengths.length > 0"
              class="flex flex-col gap-(--spacing-xxs)"
            >
              <li
                v-for="strength in item.strengths"
                :key="strength"
                class="flex items-center gap-(--spacing-xs) text-(length:--text-tiny) text-(--text-secondary)"
              >
                <AppIcon
                  name="checkmark"
                  size="sm"
                  class="shrink-0 text-(--color-green)"
                />
                {{ strength }}
              </li>
            </ul>
            <div
              v-if="buttons(item).setDefault || buttons(item).setProject || buttons(item).clear"
              class="mt-(--spacing-xs) flex flex-wrap items-center gap-(--spacing-md) border-t border-(--border)/20 pt-(--spacing-sm)"
            >
              <button
                v-if="buttons(item).setDefault"
                type="button"
                title="Set as global default"
                :aria-label="`Set as Default: ${item.displayName}`"
                :disabled="isBusy"
                class="text-(length:--text-tiny) text-(--accent) disabled:opacity-50"
                @click="setDefault(item)"
              >
                Set as Default
              </button>
              <button
                v-if="buttons(item).setProject"
                type="button"
                title="Set as project default"
                :aria-label="`Set as Project Default: ${item.displayName}`"
                :disabled="isBusy"
                class="text-(length:--text-tiny) text-(--color-green) disabled:opacity-50"
                @click="setProjectDefault(item)"
              >
                Set as Project Default
              </button>
              <button
                v-if="buttons(item).clear"
                type="button"
                title="Clear project override"
                :aria-label="`Clear Project Override: ${item.displayName}`"
                :disabled="isBusy"
                class="text-(length:--text-tiny) text-(--color-red) disabled:opacity-50"
                @click="clearProjectDefault()"
              >
                Clear Project Override
              </button>
            </div>
          </div>
        </template>
      </ConfigListLayout>
    </div>
  </div>
</template>
