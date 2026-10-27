<script setup lang="ts">
// Ported from AnalyticsDateRange (docs/spec/analytics.md §2): 4 presets (week/month/quarter/all)
// plus a custom range. `isCustomRangePresented`/`customStartDate`/`customEndDate` were a
// deliberate M3-31 scope cut from the store itself (no Swift test exercises them, they're
// popover-local UI state with no IPC write) -- this component owns that local popover state and
// writes `analyticsStore.selectedDateRange` directly on confirm, the same direct-mutation idiom
// `sessionStore.sessionSearchQuery` already uses elsewhere in this port.
//
// Open question §12.1 in analytics.md: the Swift reference app's own segmented picker never
// highlights anything while a custom range is active (its preset list excludes `.custom`
// entirely). Reproduced as-is here too -- a 5th, always-dim "Custom" segment would be a new
// affordance this task's own AC doesn't ask for.
import { computed, ref } from 'vue';
import { useAnalyticsStore, parseStatsDate } from '../store';
import AppIcon from '@/components/AppIcon.vue';

const store = useAnalyticsStore();

const PRESETS: { id: 'week' | 'month' | 'quarter' | 'all'; label: string }[] = [
  { id: 'week', label: '7d' },
  { id: 'month', label: '30d' },
  { id: 'quarter', label: '90d' },
  { id: 'all', label: 'All' },
];

const isPopoverOpen = ref(false);
const draftStart = ref('');
const draftEnd = ref('');

const activePreset = computed(() => (store.selectedDateRange.kind === 'custom' ? null : store.selectedDateRange.kind));

function selectPreset(id: (typeof PRESETS)[number]['id']) {
  store.selectedDateRange = { kind: id };
  isPopoverOpen.value = false;
}

function openPopover() {
  draftStart.value = '';
  draftEnd.value = '';
  isPopoverOpen.value = true;
}

function applyCustomRange() {
  if (!draftStart.value || !draftEnd.value) return;
  // `new Date(aYyyyMmDdString)` parses as *UTC* midnight -- misaligned against the store's own
  // local-midnight day comparisons (`parseStatsDate`'s own header comment explains why), which
  // would silently exclude the custom range's own start or end day depending on the user's UTC
  // offset direction. `<input type="date">`'s value is already a plain `yyyy-MM-dd` string, so
  // the store's own local-component parser applies directly, no new logic needed.
  store.selectedDateRange = { kind: 'custom', start: parseStatsDate(draftStart.value), end: parseStatsDate(draftEnd.value) };
  isPopoverOpen.value = false;
}
</script>

<template>
  <div class="relative flex items-center gap-(--spacing-xs)">
    <div
      role="radiogroup"
      aria-label="Date range"
      class="flex gap-(--spacing-xxs) rounded-sm border bg-(--bg-elevated) p-(--spacing-xxs)"
    >
      <button
        v-for="preset in PRESETS"
        :key="preset.id"
        type="button"
        role="radio"
        :aria-checked="activePreset === preset.id"
        class="rounded-sm px-(--spacing-sm) py-(--spacing-xxs) text-(length:--text-small)"
        :class="activePreset === preset.id ? 'bg-(--accent-dim) text-(--accent)' : 'text-(--text-secondary)'"
        @click="selectPreset(preset.id)"
      >
        {{ preset.label }}
      </button>
    </div>
    <button
      type="button"
      title="Custom range"
      class="flex items-center gap-(--spacing-xxs) rounded-sm border px-(--spacing-sm) py-(--spacing-xxs) text-(length:--text-small)"
      :class="store.isCustomRange ? 'bg-(--accent-dim) text-(--accent)' : 'text-(--text-secondary)'"
      @click="openPopover"
    >
      <AppIcon
        name="calendar"
        size="sm"
      />
      Custom
    </button>

    <div
      v-if="isPopoverOpen"
      class="glass absolute top-full right-0 z-10 mt-(--spacing-xs) flex w-60 flex-col gap-(--spacing-sm) rounded-md p-(--spacing-md)"
    >
      <label class="flex flex-col gap-(--spacing-xxs) text-(length:--text-small) text-(--text-secondary)">
        Start
        <input
          v-model="draftStart"
          type="date"
          class="rounded-sm border bg-(--bg-card) px-(--spacing-sm) py-(--spacing-xs) text-(--text-primary)"
        >
      </label>
      <label class="flex flex-col gap-(--spacing-xxs) text-(length:--text-small) text-(--text-secondary)">
        End
        <input
          v-model="draftEnd"
          type="date"
          class="rounded-sm border bg-(--bg-card) px-(--spacing-sm) py-(--spacing-xs) text-(--text-primary)"
        >
      </label>
      <button
        type="button"
        class="rounded-sm border border-(--accent)/30 bg-(--accent-dim) px-(--spacing-md) py-(--spacing-xs) text-(--accent) text-(length:--text-small)"
        :disabled="!draftStart || !draftEnd"
        @click="applyCustomRange"
      >
        Apply
      </button>
    </div>
  </div>
</template>
