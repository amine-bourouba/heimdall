<script setup lang="ts">
// Ported from ModelsListView.swift (M3-13). Read-only: "Set as Default"/"Set as Project Default"/
// "Clear Project Override" are all mutating (M3-50's job), so no per-card buttons render, only
// the two pills showing which model currently holds each role. No project picker widget
// (config-screens.md §1.2: Models reads project scope through its own inline banner/per-card
// buttons in Swift, neither of which exists yet here) -- but `list_models` still takes a
// `project` argument (confirmed directly in bindings.ts, unlike list_plugins), so this screen
// still passes the shared `configStore.selectedProject` straight through, reusing whatever
// another screen's own picker may already have set rather than always sending `null`.
//
// No skeleton and no "items missing" empty state (§1.4/§1.5: the curated catalog is never truly
// empty) -- `isLoading` is tracked for the fetch itself but deliberately never passed to
// ConfigListLayout, which always shows the filter-miss flavour of ConfigEmptyState.vue instead of
// a loading skeleton for the brief window before the first list_models resolves.
import { computed, ref, watch } from 'vue';
import { ipc } from '@/ipc/client';
import { fuzzyMatch } from '@/lib/fuzzyMatch';
import { useConfigStore } from '../store';
import ConfigScreenHeader from '../shared/ConfigScreenHeader.vue';
import ConfigLayoutToolbar from '../shared/ConfigLayoutToolbar.vue';
import ConfigListLayout from '../shared/ConfigListLayout.vue';
import ConfigBadge from '../shared/ConfigBadge.vue';
import AppIcon from '@/components/AppIcon.vue';
import type { ClaudeModelInfo, ModelsView } from '@/ipc/bindings';

const SCREEN_ID = 'models';
const configStore = useConfigStore();

const view = ref<ModelsView>({ models: [], defaultModel: '', projectModel: null });
const filterQuery = ref('');

async function load() {
  view.value = await ipc.listModels(configStore.selectedProject);
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
                v-if="item.displayName === view.defaultModel"
                text="Global Default"
                fg-class="text-(--accent)"
                bg-class="bg-(--accent-dim)"
              />
              <ConfigBadge
                v-if="view.projectModel && item.displayName === view.projectModel"
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
          </div>
        </template>
      </ConfigListLayout>
    </div>
  </div>
</template>
