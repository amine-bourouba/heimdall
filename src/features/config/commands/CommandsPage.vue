<script setup lang="ts">
// Ported from CommandsListView.swift (M3-12), built on the shared blocks M3-11 establishes.
// Mounted for both /config/command (list) and /config/command/:scope/:slug (detail) --
// ConfigListRoute.vue/ConfigDetailRoute.vue both dispatch here for the "command" kind, so which
// pane shows is purely a function of whether :scope/:slug are present on the current route.
//
// AC: "No create, edit or delete controls yet (M3-50)" -- showAddButton stays false even though
// Swift's own CommandsListView passes true: the "+" button exists in Swift only because its own
// create flow (createAndOpen -> createCommandTemplate) already exists; Heimdall's does not yet.
import { computed, ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import { ipc } from '@/ipc/client';
import { fuzzyMatch } from '@/lib/fuzzyMatch';
import { useConfigStore } from '../store';
import ConfigScreenHeader from '../shared/ConfigScreenHeader.vue';
import ConfigLayoutToolbar from '../shared/ConfigLayoutToolbar.vue';
import ConfigListLayout from '../shared/ConfigListLayout.vue';
import ConfigItemDetail from '../shared/ConfigItemDetail.vue';
import ConfigBadge from '../shared/ConfigBadge.vue';
import AppIcon from '@/components/AppIcon.vue';
import type { ConfigItemDetail as ConfigItemDetailType, ConfigItemSummary, ConfigRef } from '@/ipc/bindings';

const SCREEN_ID = 'commands';
const route = useRoute();
const router = useRouter();
const configStore = useConfigStore();

const items = ref<ConfigItemSummary[]>([]);
const isLoading = ref(true);
const filterQuery = ref('');
const copiedSlug = ref<string | null>(null);

async function load() {
  isLoading.value = true;
  items.value = (await ipc.listConfigItems('command', configStore.selectedProject)).value;
  isLoading.value = false;
}
watch(() => configStore.selectedProject, load, { deep: true, immediate: true });

const filteredItems = computed<ConfigItemSummary[]>(() => {
  const q = filterQuery.value.trim();
  if (q === '') return items.value;
  return items.value
    .map((item) => ({ item, score: fuzzyMatch(`${item.name} ${item.description}`, q)?.score ?? 0 }))
    .filter((r) => r.score > 0)
    .sort((a, b) => b.score - a.score)
    .map((r) => r.item);
});

// ConfigBadge.swift:80-86's own `ConfigHelpers.formatModel`: strip the "claude-" prefix, take the
// first two hyphen-separated segments, title-case them -- a plain generic title-caser, distinct
// from session/format.ts's own formatModel (which looks up a fixed opus/sonnet/haiku label table
// for a *dated* model id; command/skill front-matter's `model:` is often just a bare alias like
// "opus" that table was never built to handle).
function formatModel(model: string): string {
  return model
    .replace('claude-', '')
    .split('-')
    .slice(0, 2)
    .map((s) => s.charAt(0).toUpperCase() + s.slice(1))
    .join(' ');
}

function toolCount(meta: Record<string, string>): number {
  return meta.allowedTools ? meta.allowedTools.split(',').filter(Boolean).length : 0;
}

async function copy(item: ConfigItemSummary) {
  await writeText(item.name); // already "/name" -- command_loader::load's own format!("/{stem}")
  copiedSlug.value = item.ref.slug;
  setTimeout(() => (copiedSlug.value = null), 2000);
}

function openItem(item: ConfigItemSummary) {
  void router.push({ name: 'config-detail', params: { kind: 'command', scope: item.scope === 'project' ? 'project' : 'global', slug: item.ref.slug } });
}

function closeDetail() {
  void router.push({ name: 'config-list', params: { kind: 'command' } });
}

const detailRef = computed<ConfigRef | null>(() => {
  const { scope, slug } = route.params;
  if (!scope || !slug) return null;
  return { kind: 'command', slug: slug as string, project: scope === 'project' ? configStore.selectedProject : null };
});
const emptyMessage = computed(() => (filterQuery.value ? `No commands match "${filterQuery.value}"` : 'No commands found'));
const emptyHint = computed(() => (filterQuery.value ? 'Try a different search term' : '~/.claude/commands/'));

const detail = ref<ConfigItemDetailType | null>(null);
watch(
  detailRef,
  async (ref) => {
    detail.value = null;
    if (ref) detail.value = await ipc.getConfigItem(ref);
  },
  { immediate: true },
);
</script>

<template>
  <ConfigItemDetail
    v-if="detail"
    icon="apple.terminal.fill"
    :title="detail.name"
    :scope="detail.scope"
    :body="detail.body"
    @close="closeDetail"
  >
    <template #badges>
      <ConfigBadge
        v-if="detail.meta.model"
        :text="formatModel(detail.meta.model)"
      />
      <ConfigBadge
        v-if="toolCount(detail.meta) > 0"
        :text="`${toolCount(detail.meta)} tools`"
      />
      <ConfigBadge
        v-if="detail.meta.argumentHint"
        :text="detail.meta.argumentHint"
      />
      <ConfigBadge
        v-if="detail.meta.outputStyle"
        :text="detail.meta.outputStyle"
      />
    </template>
  </ConfigItemDetail>

  <div
    v-else
    class="flex h-full flex-col"
  >
    <ConfigScreenHeader
      icon="apple.terminal.fill"
      title="Commands"
      :count="items.length"
      description="Slash commands available to Claude Code."
    />
    <ConfigLayoutToolbar
      v-model:filter-query="filterQuery"
      :screen-id="SCREEN_ID"
      filter-placeholder="Find in Commands…"
      show-project-picker
    />
    <div class="flex-1 overflow-y-auto px-(--spacing-xxxl) py-(--spacing-lg)">
      <ConfigListLayout
        :layout="configStore.layoutFor(SCREEN_ID)"
        :is-loading="isLoading"
        :items="filteredItems"
        empty-icon="apple.terminal.fill"
        :empty-message="emptyMessage"
        :empty-hint="emptyHint"
      >
        <template #default="{ item }">
          <button
            type="button"
            data-testid="config-item-card"
            class="flex flex-col gap-(--spacing-xs) rounded-md border bg-(--bg-card) p-(--spacing-lg) text-left hover:bg-(--bg-card-hover)"
            @click="openItem(item)"
          >
            <div class="flex items-start gap-(--spacing-sm)">
              <p class="min-w-0 flex-1 truncate text-(length:--text-body-medium) text-(--text-primary)">
                {{ item.name }}
              </p>
              <button
                type="button"
                title="Copy /name"
                class="shrink-0 rounded-sm p-(--spacing-xxs)"
                @click.stop="copy(item)"
              >
                <AppIcon
                  :name="copiedSlug === item.ref.slug ? 'checkmark' : 'doc.on.doc'"
                  size="sm"
                  :class="copiedSlug === item.ref.slug ? 'text-(--color-green)' : 'text-(--text-tertiary)'"
                />
              </button>
            </div>
            <p
              v-if="item.description"
              class="line-clamp-2 text-(length:--text-small) text-(--text-tertiary)"
            >
              {{ item.description }}
            </p>
            <div class="flex flex-wrap items-center gap-(--spacing-xs)">
              <ConfigBadge :scope="item.scope" />
              <ConfigBadge
                v-if="item.meta.model"
                :text="formatModel(item.meta.model)"
              />
              <ConfigBadge
                v-if="toolCount(item.meta) > 0"
                :text="`${toolCount(item.meta)} tools`"
              />
              <ConfigBadge
                v-if="item.meta.outputStyle"
                :text="item.meta.outputStyle"
              />
            </div>
          </button>
        </template>
      </ConfigListLayout>
    </div>
  </div>
</template>
