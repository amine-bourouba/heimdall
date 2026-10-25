<script setup lang="ts">
// Ported from OutputStylesListView.swift (M3-12). Output styles have no model/tools front-matter
// at all (`output_style_summary` in config_read.rs always sends an empty `meta`, confirmed
// directly) -- the card and detail badges show a filename chip instead, derived as
// `${ref.slug}.md` the same way PlansPage.vue (M3-03) already shows a plan's own filename (output
// styles, like plans and commands, are plain `.md` files, never a folder). No per-card copy
// button either (OutputStylesListView.swift:253-297 has none, unlike Commands/Skills).
//
// OutputStylesListView.swift:47-48 tints its own detail icon red rather than the usual accent --
// not ported: ConfigScreenHeader.vue/ConfigItemDetail.vue (M3-11) hardcode the accent tint with no
// color override prop, and adding one for a single cosmetic difference nowhere in this task's own
// AC isn't worth the ripple. All three screens share one tint instead.
import { computed, ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { ipc } from '@/ipc/client';
import { fuzzyMatch } from '@/lib/fuzzyMatch';
import { useConfigStore } from '../store';
import ConfigScreenHeader from '../shared/ConfigScreenHeader.vue';
import ConfigLayoutToolbar from '../shared/ConfigLayoutToolbar.vue';
import ConfigListLayout from '../shared/ConfigListLayout.vue';
import ConfigItemDetail from '../shared/ConfigItemDetail.vue';
import ConfigBadge from '../shared/ConfigBadge.vue';
import type { ConfigItemDetail as ConfigItemDetailType, ConfigItemSummary, ConfigRef } from '@/ipc/bindings';

const SCREEN_ID = 'outputStyles';
const route = useRoute();
const router = useRouter();
const configStore = useConfigStore();

const items = ref<ConfigItemSummary[]>([]);
const isLoading = ref(true);
const filterQuery = ref('');

async function load() {
  isLoading.value = true;
  items.value = (await ipc.listConfigItems('outputStyle', configStore.selectedProject)).value;
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

function openItem(item: ConfigItemSummary) {
  void router.push({ name: 'config-detail', params: { kind: 'outputStyle', scope: item.scope === 'project' ? 'project' : 'global', slug: item.ref.slug } });
}

function closeDetail() {
  void router.push({ name: 'config-list', params: { kind: 'outputStyle' } });
}

const detailRef = computed<ConfigRef | null>(() => {
  const { scope, slug } = route.params;
  if (!scope || !slug) return null;
  return { kind: 'outputStyle', slug: slug as string, project: scope === 'project' ? configStore.selectedProject : null };
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

const emptyMessage = computed(() => (filterQuery.value ? `No output styles match "${filterQuery.value}"` : 'No output styles found'));
const emptyHint = computed(() => (filterQuery.value ? 'Try a different search term' : '~/.claude/output-styles/'));
</script>

<template>
  <ConfigItemDetail
    v-if="detail"
    icon="speaker.wave.3.fill"
    :title="detail.name"
    :scope="detail.scope"
    :body="detail.body"
    @close="closeDetail"
  >
    <template #badges>
      <ConfigBadge
        v-if="detail.description"
        :text="detail.description"
      />
      <ConfigBadge :text="`${detail.ref.slug}.md`" />
    </template>
  </ConfigItemDetail>

  <div
    v-else
    class="flex h-full flex-col"
  >
    <ConfigScreenHeader
      icon="speaker.wave.3.fill"
      title="Output Styles"
      :count="items.length"
      description="Styles that shape how Claude Code responds."
    />
    <ConfigLayoutToolbar
      v-model:filter-query="filterQuery"
      :screen-id="SCREEN_ID"
      filter-placeholder="Find in Output Styles…"
      show-project-picker
    />
    <div class="flex-1 overflow-y-auto px-(--spacing-xxxl) py-(--spacing-lg)">
      <ConfigListLayout
        :layout="configStore.layoutFor(SCREEN_ID)"
        :is-loading="isLoading"
        :items="filteredItems"
        empty-icon="speaker.wave.3.fill"
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
            <p class="truncate text-(length:--text-body-medium) text-(--text-primary)">
              {{ item.name }}
            </p>
            <p
              v-if="item.description"
              class="line-clamp-2 text-(length:--text-small) text-(--text-tertiary)"
            >
              {{ item.description }}
            </p>
            <div class="flex flex-wrap items-center gap-(--spacing-xs)">
              <ConfigBadge :scope="item.scope" />
              <ConfigBadge :text="`${item.ref.slug}.md`" />
            </div>
          </button>
        </template>
      </ConfigListLayout>
    </div>
  </div>
</template>
