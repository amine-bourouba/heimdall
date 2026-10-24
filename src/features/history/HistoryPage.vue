<script setup lang="ts">
// Ported from HistoryListView.swift (M3-02): header, search, date-bucketed list, paging. The
// project-filter picker, grid/list toggle and "clear old prompts" menu are not in this task's
// own Outputs/AC list -- left for a later pass if ever asked for, not an oversight (M3-02's own
// scope is explicitly narrower than the full Swift screen: "Delete actions are not present yet").
import { onMounted } from 'vue';
import AppIcon from '@/components/AppIcon.vue';
import FilterField from '@/components/FilterField.vue';
import ShimmerBlock from '@/components/ShimmerBlock.vue';
import { HISTORY_GROUP_TITLES, useHistoryStore } from './store';
import HistoryRow from './HistoryRow.vue';

const store = useHistoryStore();

onMounted(() => void store.load());
</script>

<template>
  <div class="flex h-full flex-col">
    <div class="flex items-center gap-(--spacing-md) border-b px-(--spacing-xxxl) py-(--spacing-xl)">
      <div class="flex h-9 w-9 items-center justify-center rounded-md bg-(--accent-dim)">
        <AppIcon
          name="clock.arrow.circlepath"
          class="text-(--accent)"
        />
      </div>
      <div class="flex flex-col gap-(--spacing-xxs)">
        <h1 class="text-(--text-primary) text-(length:--text-heading)">
          History
        </h1>
        <span class="self-start rounded-full bg-(--bg-elevated) px-(--spacing-sm) py-(--spacing-xxs) text-(--text-tertiary) text-(length:--text-tiny)">
          {{ store.filteredEntries.length === store.total ? store.total : `${store.filteredEntries.length} of ${store.total}` }}
          {{ store.total === 1 ? 'prompt' : 'prompts' }}
        </span>
      </div>
      <div class="flex-1" />
      <FilterField
        v-model="store.query"
        placeholder="Search prompts…"
        class="w-50"
      />
    </div>

    <div class="flex-1 overflow-y-auto px-(--spacing-xxxl) py-(--spacing-lg)">
      <div
        v-if="store.isLoading"
        class="flex flex-col gap-(--spacing-sm)"
      >
        <ShimmerBlock
          v-for="n in 6"
          :key="n"
          rounded="md"
          class="h-20"
        />
      </div>

      <div
        v-else-if="store.entries.length === 0"
        class="flex h-full flex-col items-center justify-center gap-(--spacing-xs) text-(--text-tertiary)"
      >
        <AppIcon
          name="clock.arrow.circlepath"
          size="lg"
        />
        <p>No history found</p>
        <p class="text-(length:--text-small)">
          ~/.claude/history.jsonl
        </p>
      </div>

      <div
        v-else-if="store.filteredEntries.length === 0"
        class="flex h-full flex-col items-center justify-center gap-(--spacing-xs) text-(--text-tertiary)"
      >
        <AppIcon
          name="magnifyingglass"
          size="lg"
        />
        <p>No prompts match your search</p>
      </div>

      <template v-else>
        <div
          v-for="group in store.groupedEntries"
          :key="group.group"
          class="flex flex-col gap-(--spacing-sm)"
        >
          <div class="flex items-center gap-(--spacing-sm) pt-(--spacing-md) pb-(--spacing-xs)">
            <h2 class="text-(--text-tertiary) text-(length:--text-tiny)">
              {{ HISTORY_GROUP_TITLES[group.group].toUpperCase() }}
            </h2>
            <div class="h-px flex-1 bg-(--border)" />
          </div>
          <HistoryRow
            v-for="entry in group.entries"
            :key="entry.id"
            :entry="entry"
            :query="store.query"
          />
        </div>

        <div
          v-if="store.hasMore"
          class="flex justify-center pt-(--spacing-lg)"
        >
          <button
            type="button"
            class="rounded-sm bg-(--bg-elevated) px-(--spacing-md) py-(--spacing-sm) text-(--text-secondary) text-(length:--text-small)"
            :disabled="store.isLoadingMore"
            @click="store.loadMore()"
          >
            {{ store.isLoadingMore ? 'Loading…' : 'Load more' }}
          </button>
        </div>
      </template>
    </div>
  </div>
</template>
