<script setup lang="ts">
// Ported from SkillsListView.swift (M3-12), structurally identical to CommandsPage.vue with the
// differences SkillsListView.swift itself documents: a bare name (no "/" prefix -- skills.rs's
// own loader never adds one, unlike command_loader's `format!("/{stem}")`), a 3-line description
// clamp (commands use 2), no output-style badge (skills have no output_style front-matter key at
// all), and a detail view that lists one badge per allowed tool instead of a single "N tools"
// count -- SkillsListView.swift:53-72's own, deliberately different choice from
// CommandsListView.swift's aggregate badge.
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

const SCREEN_ID = 'skills';
const route = useRoute();
const router = useRouter();
const configStore = useConfigStore();

const items = ref<ConfigItemSummary[]>([]);
const isLoading = ref(true);
const filterQuery = ref('');
const copiedSlug = ref<string | null>(null);

async function load() {
  isLoading.value = true;
  items.value = (await ipc.listConfigItems('skill', configStore.selectedProject)).value;
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

// Same algorithm as CommandsPage.vue's own formatModel -- see that file's header comment.
function formatModel(model: string): string {
  return model
    .replace('claude-', '')
    .split('-')
    .slice(0, 2)
    .map((s) => s.charAt(0).toUpperCase() + s.slice(1))
    .join(' ');
}

function tools(meta: Record<string, string>): string[] {
  return meta.allowedTools ? meta.allowedTools.split(',').filter(Boolean) : [];
}

async function copy(item: ConfigItemSummary) {
  await writeText(item.name);
  copiedSlug.value = item.ref.slug;
  setTimeout(() => (copiedSlug.value = null), 2000);
}

function openItem(item: ConfigItemSummary) {
  void router.push({ name: 'config-detail', params: { kind: 'skill', scope: item.scope === 'project' ? 'project' : 'global', slug: item.ref.slug } });
}

function closeDetail() {
  void router.push({ name: 'config-list', params: { kind: 'skill' } });
}

const detailRef = computed<ConfigRef | null>(() => {
  const { scope, slug } = route.params;
  if (!scope || !slug) return null;
  return { kind: 'skill', slug: slug as string, project: scope === 'project' ? configStore.selectedProject : null };
});
const detail = ref<ConfigItemDetailType | null>(null);
watch(
  detailRef,
  async (ref) => {
    detail.value = null;
    if (ref) detail.value = await ipc.getConfigItem(ref);
  },
  { immediate: true },
);

const emptyMessage = computed(() => (filterQuery.value ? `No skills match "${filterQuery.value}"` : 'No skills found'));
const emptyHint = computed(() => (filterQuery.value ? 'Try a different search term' : '~/.claude/skills/'));
</script>

<template>
  <ConfigItemDetail
    v-if="detail"
    icon="bolt.fill"
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
        v-for="tool in tools(detail.meta)"
        :key="tool"
        :text="tool"
      />
    </template>
  </ConfigItemDetail>

  <div
    v-else
    class="flex h-full flex-col"
  >
    <ConfigScreenHeader
      icon="bolt.fill"
      title="Skills"
      :count="items.length"
      description="Skills Claude Code can invoke for specialized tasks."
    />
    <ConfigLayoutToolbar
      v-model:filter-query="filterQuery"
      :screen-id="SCREEN_ID"
      filter-placeholder="Find in Skills…"
      show-project-picker
    />
    <div class="flex-1 overflow-y-auto px-(--spacing-xxxl) py-(--spacing-lg)">
      <ConfigListLayout
        :layout="configStore.layoutFor(SCREEN_ID)"
        :is-loading="isLoading"
        :items="filteredItems"
        empty-icon="bolt.fill"
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
                title="Copy name"
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
              class="line-clamp-3 text-(length:--text-small) text-(--text-tertiary)"
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
                v-if="tools(item.meta).length > 0"
                :text="`${tools(item.meta).length} tools`"
              />
            </div>
          </button>
        </template>
      </ConfigListLayout>
    </div>
  </div>
</template>
