<script setup lang="ts">
// Ported from MemoryListView.swift (M3-04). Source chips and file cards are rendered inline,
// not as separate components -- this task's own Outputs list names only MemoryPage.vue and
// MemoryDetail.vue, the same scope-narrowing M3-03's own PlansPage.vue already established.
import { computed, onMounted } from 'vue';
import type { MemoryFileSummary } from '@/ipc/bindings';
import AppIcon from '@/components/AppIcon.vue';
import FilterField from '@/components/FilterField.vue';
import ShimmerBlock from '@/components/ShimmerBlock.vue';
import { useMemoryStore } from './store';
import MemoryDetail from './MemoryDetail.vue';

const store = useMemoryStore();

onMounted(() => void store.load());

// MemoryListView.swift:149-157's own `memoryCountLabel`.
const countText = computed(() => {
  const count = store.files.length;
  const word = count === 1 ? 'file' : 'files';
  const source = store.sources.find((s) => s.id === store.selectedSourceId);
  return source ? `${count} ${word} in ${source.label}` : `${count} ${word}`;
});

function openFile(file: MemoryFileSummary) {
  void store.open(file.sourceId, file.id);
}
</script>

<template>
  <MemoryDetail
    v-if="store.selected"
    :memory="store.selected"
    @back="store.close()"
  />

  <div
    v-else
    class="flex h-full flex-col"
  >
    <div class="flex items-center gap-(--spacing-md) border-b px-(--spacing-xxxl) py-(--spacing-xl)">
      <div class="flex h-9 w-9 items-center justify-center rounded-md bg-(--color-purple)/15">
        <AppIcon
          name="brain.head.profile.fill"
          class="text-(--color-purple)"
        />
      </div>
      <div class="flex flex-col gap-(--spacing-xxs)">
        <h1 class="text-(--text-primary) text-(length:--text-heading)">
          Memory
        </h1>
        <span class="self-start rounded-full bg-(--bg-elevated) px-(--spacing-sm) py-(--spacing-xxs) text-(--text-tertiary) text-(length:--text-tiny)">{{ countText }}</span>
      </div>
      <div class="flex-1" />
      <FilterField
        v-model="store.query"
        placeholder="Find in Memory…"
        class="w-50"
      />
      <button
        type="button"
        class="flex shrink-0 items-center gap-(--spacing-xs) rounded-sm bg-(--bg-elevated) px-(--spacing-sm) py-(--spacing-xs) text-(--text-secondary) text-(length:--text-small)"
        @click="store.addFolder()"
      >
        <AppIcon
          name="folder.badge.plus"
          size="sm"
        />
        Add Folder
      </button>
    </div>

    <div
      v-if="!store.isLoading && store.sources.length > 0"
      class="flex items-center gap-(--spacing-sm) overflow-x-auto border-b px-(--spacing-xxxl) py-(--spacing-sm)"
    >
      <button
        type="button"
        class="shrink-0 rounded-sm px-(--spacing-md) py-(--spacing-xs) text-(length:--text-tiny)"
        :class="store.selectedSourceId === null ? 'bg-(--accent-dim) text-(--accent)' : 'bg-(--bg-card) text-(--text-secondary)'"
        @click="store.selectSource(null)"
      >
        All {{ store.totalCount }}
      </button>
      <span
        v-for="source in store.sources"
        :key="source.id"
        class="flex shrink-0 items-center gap-(--spacing-xs) rounded-sm px-(--spacing-md) py-(--spacing-xs) text-(length:--text-tiny)"
        :class="store.selectedSourceId === source.id ? 'bg-(--accent-dim) text-(--accent)' : 'bg-(--bg-card) text-(--text-secondary)'"
      >
        <button
          type="button"
          class="flex items-center gap-(--spacing-xs)"
          @click="store.selectSource(source.id)"
        >
          <AppIcon
            v-if="source.kind === 'folder'"
            name="folder"
            size="sm"
          />
          {{ source.label }} {{ source.fileCount }}
        </button>
        <button
          v-if="source.kind === 'folder'"
          type="button"
          aria-label="Remove folder"
          title="Remove folder"
          @click="store.removeFolder(source.id)"
        >
          <AppIcon
            name="folder.badge.minus"
            size="sm"
          />
        </button>
      </span>
    </div>

    <div class="flex-1 overflow-y-auto px-(--spacing-xxxl) py-(--spacing-lg)">
      <div
        v-if="store.isLoading"
        class="grid grid-cols-2 gap-(--spacing-md)"
      >
        <ShimmerBlock
          v-for="n in 6"
          :key="n"
          rounded="md"
          class="h-20"
        />
      </div>

      <div
        v-else-if="store.sources.length === 0"
        class="flex h-full flex-col items-center justify-center gap-(--spacing-xs) text-(--text-tertiary)"
      >
        <AppIcon
          name="brain.head.profile"
          size="lg"
        />
        <p>No memory files found</p>
        <p class="text-(length:--text-small)">
          Add a folder above, or use ~/.claude/projects/&lt;project&gt;/memory/
        </p>
      </div>

      <div
        v-else-if="store.filteredFiles.length === 0 && store.query === ''"
        class="flex h-full flex-col items-center justify-center gap-(--spacing-xs) text-(--text-tertiary)"
      >
        <AppIcon
          name="brain.head.profile"
          size="lg"
        />
        <p>No memory files in this source</p>
      </div>

      <div
        v-else-if="store.filteredFiles.length === 0"
        class="flex h-full flex-col items-center justify-center gap-(--spacing-xs) text-(--text-tertiary)"
      >
        <AppIcon
          name="magnifyingglass"
          size="lg"
        />
        <p>No memories match "{{ store.query }}"</p>
      </div>

      <div
        v-else
        class="grid grid-cols-2 gap-(--spacing-md)"
      >
        <button
          v-for="file in store.filteredFiles"
          :key="`${file.sourceId}:${file.id}`"
          type="button"
          class="rounded-md border bg-(--bg-card) p-(--spacing-lg) text-left hover:bg-(--bg-card-hover)"
          @click="openFile(file)"
        >
          <div class="flex items-center gap-(--spacing-sm)">
            <AppIcon
              v-if="file.isMain"
              name="star.fill"
              size="sm"
              class="text-(--accent)"
            />
            <p class="text-(--text-primary) text-(length:--text-body-medium)">
              {{ file.name }}
            </p>
          </div>
          <p class="mt-(--spacing-xs) text-(--text-tertiary) text-(length:--text-tiny)">
            {{ file.filename }}
          </p>
        </button>
      </div>
    </div>
  </div>
</template>
