<script setup lang="ts">
// Ported from PluginsListView.swift (M3-13). Read-only: the enabled/disabled pill is a plain
// badge here, not Swift's own popover-toggle button (M3-50's job); no trash icon either. No
// project picker (config-screens.md §1.2: "Not shown for... Plugins... no project scope at all")
// and no detail route (§1.7: ConfigItemDetailView is used only by Commands/Skills/OutputStyles) --
// list_plugins itself takes no project argument at all, confirmed directly in bindings.ts.
import { computed, ref } from 'vue';
import { ipc } from '@/ipc/client';
import { fuzzyMatch } from '@/lib/fuzzyMatch';
import { formatRelativeTime } from '@/features/session/format';
import { useConfigStore } from '../store';
import ConfigScreenHeader from '../shared/ConfigScreenHeader.vue';
import ConfigLayoutToolbar from '../shared/ConfigLayoutToolbar.vue';
import ConfigListLayout from '../shared/ConfigListLayout.vue';
import ConfigBadge from '../shared/ConfigBadge.vue';
import type { PluginInfo } from '@/ipc/bindings';

const SCREEN_ID = 'plugins';
const configStore = useConfigStore();

const items = ref<PluginInfo[]>([]);
const isLoading = ref(true);
const filterQuery = ref('');

async function load() {
  isLoading.value = true;
  items.value = await ipc.listPlugins();
  isLoading.value = false;
}
void load();

const filteredItems = computed<PluginInfo[]>(() => {
  const q = filterQuery.value.trim();
  if (q === '') return items.value;
  return items.value
    .map((item) => ({ item, score: fuzzyMatch(`${item.name} ${item.author}`, q)?.score ?? 0 }))
    .filter((r) => r.score > 0)
    .sort((a, b) => b.score - a.score)
    .map((r) => r.item);
});

// PluginsListView.swift:177-190's own installedAt parse: a non-ISO-8601 string (or none at all)
// means no "Installed X ago" line, not a fallback label.
function installedAgo(iso: string | null): string | null {
  if (!iso) return null;
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return null;
  return `Installed ${formatRelativeTime(iso)}`;
}

const emptyMessage = computed(() => (filterQuery.value ? `No plugins match "${filterQuery.value}"` : 'No plugins found'));
const emptyHint = computed(() => (filterQuery.value ? 'Try a different search term' : 'Install via claude plugin add'));
</script>

<template>
  <div class="flex h-full flex-col">
    <ConfigScreenHeader
      icon="puzzlepiece.fill"
      title="Plugins"
      :count="items.length"
      description="Plugins installed for Claude Code."
    />
    <ConfigLayoutToolbar
      v-model:filter-query="filterQuery"
      :screen-id="SCREEN_ID"
      filter-placeholder="Find in Plugins…"
    />
    <div class="flex-1 overflow-y-auto px-(--spacing-xxxl) py-(--spacing-lg)">
      <p class="mb-(--spacing-lg) rounded-md bg-(--bg-elevated) p-(--spacing-md) text-(length:--text-small) text-(--text-tertiary)">
        Plugins are installed via <code>claude plugin add</code> or by placing files under <code>~/.claude/plugins/</code>.
      </p>
      <ConfigListLayout
        :layout="configStore.layoutFor(SCREEN_ID)"
        :is-loading="isLoading"
        :items="filteredItems"
        empty-icon="puzzlepiece.fill"
        :empty-message="emptyMessage"
        :empty-hint="emptyHint"
      >
        <template #default="{ item }">
          <div class="flex flex-col gap-(--spacing-xs) rounded-md border bg-(--bg-card) p-(--spacing-lg)">
            <div class="flex items-start gap-(--spacing-sm)">
              <p class="min-w-0 flex-1 truncate text-(length:--text-body-medium) text-(--text-primary)">
                {{ item.name }}
              </p>
              <ConfigBadge
                :text="item.enabled ? 'Enabled' : 'Disabled'"
                :fg-class="item.enabled ? 'text-(--color-green)' : 'text-(--text-tertiary)'"
                :bg-class="item.enabled ? 'bg-(--color-green)/10' : 'bg-(--bg-elevated)'"
              />
            </div>
            <p
              v-if="item.author"
              class="text-(length:--text-small) text-(--text-tertiary)"
            >
              by {{ item.author }}
            </p>
            <div class="flex flex-wrap items-center gap-(--spacing-xs) text-(length:--text-tiny) text-(--text-tertiary)">
              <ConfigBadge
                v-if="item.version"
                :text="`v${item.version}`"
              />
              <span v-if="installedAgo(item.installedAt)">{{ installedAgo(item.installedAt) }}</span>
            </div>
          </div>
        </template>
      </ConfigListLayout>
    </div>
  </div>
</template>
