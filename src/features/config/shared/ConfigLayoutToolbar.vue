<script setup lang="ts">
// Ported from ConfigLayoutToolbar.swift:3-48. The filter field itself has no dedicated file in
// this task's own Outputs list because it already exists: FilterField.vue (M2-07, ported from
// ConfigFilterField.swift directly, confirmed by that file's own header comment) is the shared
// component every other screen's own filter box already uses (TodosPage.vue, MemoryPage.vue,
// PlansPage.vue, HistoryPage.vue) -- reused here rather than hand-rolling a second filter input.
import AppIcon from '@/components/AppIcon.vue';
import FilterField from '@/components/FilterField.vue';
import ConfigProjectPicker from './ConfigProjectPicker.vue';
import { useConfigStore } from '../store';

const filterQuery = defineModel<string>('filterQuery', { required: true });
// `addLabel` names what the (icon-only) add button creates: "New command", not just "Add".
withDefaults(
  defineProps<{
    screenId: string;
    filterPlaceholder: string;
    showProjectPicker?: boolean;
    showAddButton?: boolean;
    addLabel?: string;
  }>(),
  { addLabel: 'Add' },
);
defineEmits<{ add: [] }>();

const configStore = useConfigStore();
</script>

<template>
  <div class="flex flex-wrap items-center gap-(--spacing-sm) border-b border-(--border)/30 px-(--spacing-xxxl) py-(--spacing-sm)">
    <ConfigProjectPicker v-if="showProjectPicker" />

    <FilterField
      v-model="filterQuery"
      :placeholder="filterPlaceholder"
      class="w-50 bg-(--bg-elevated)"
    />

    <!-- One group, pushed right with ml-auto rather than a flex-1 spacer: when the picker and the
         filter fill the row (900px, the minimum window) the buttons wrap to a second row, and a
         spacer collapses there, leaving them at the left. -->
    <div class="ml-auto flex items-center gap-(--spacing-sm)">
      <button
        type="button"
        :title="configStore.layoutFor(screenId) === 'grid' ? 'Switch to list' : 'Switch to grid'"
        class="rounded-sm p-(--spacing-xs) text-(--text-secondary)"
        @click="configStore.toggleLayout(screenId)"
      >
        <AppIcon
          :name="configStore.layoutFor(screenId) === 'grid' ? 'square.grid.2x2' : 'list.bullet'"
          size="sm"
        />
      </button>

      <button
        v-if="showAddButton"
        type="button"
        :title="addLabel"
        :aria-label="addLabel"
        class="rounded-sm border border-(--accent)/30 bg-(--accent-dim) p-(--spacing-xs) text-(--accent)"
        @click="$emit('add')"
      >
        <AppIcon
          name="plus"
          size="sm"
        />
      </button>
    </div>
  </div>
</template>
