<script setup lang="ts">
// One row, shared by every list this overlay renders (empty-state SCREENS/RECENT, grouped search
// hits, quick actions) -- each caller maps its own shape down to these primitive props rather
// than this component knowing about SearchHit/NavigationItem/RecentSession itself.
//
// `highlights` are UTF-16 code-unit offsets into `title` (docs/spec/search-overlay.md §2.4,
// `SearchHit.highlights`) -- JS strings are already UTF-16 internally, so `title.slice(start,
// end)` on these offsets directly is correct with no decoding step, unlike the Swift side this
// was ported from (grapheme-cluster indexed, no such shortcut there).
import { computed } from 'vue';
import AppIcon from '@/components/AppIcon.vue';

const props = withDefaults(
  defineProps<{
    icon: string;
    title: string;
    subtitle?: string | null;
    highlights?: [number, number][];
    trailingCount?: number | null;
    selected?: boolean;
  }>(),
  { subtitle: null, highlights: () => [], trailingCount: null, selected: false },
);

const segments = computed(() => {
  if (props.highlights.length === 0) return [{ text: props.title, highlighted: false }];
  const sorted = [...props.highlights].sort((a, b) => a[0] - b[0]);
  const out: { text: string; highlighted: boolean }[] = [];
  let cursor = 0;
  for (const [start, end] of sorted) {
    if (start > cursor) out.push({ text: props.title.slice(cursor, start), highlighted: false });
    out.push({ text: props.title.slice(start, end), highlighted: true });
    cursor = end;
  }
  if (cursor < props.title.length) out.push({ text: props.title.slice(cursor), highlighted: false });
  return out;
});
</script>

<template>
  <div
    role="option"
    :aria-selected="selected"
    class="flex items-center gap-(--spacing-sm) rounded-sm px-(--spacing-sm) py-(--spacing-xs)"
    :class="selected ? 'bg-(--accent-dim) text-(--accent)' : 'text-(--text-secondary)'"
  >
    <AppIcon
      :name="icon"
      size="sm"
      class="shrink-0"
    />
    <span class="flex min-w-0 flex-1 flex-col">
      <span class="truncate">
        <template
          v-for="(segment, index) in segments"
          :key="index"
        >
          <mark
            v-if="segment.highlighted"
            class="rounded-xs bg-(--accent-dim) text-(--accent)"
          >{{ segment.text }}</mark>
          <template v-else>{{ segment.text }}</template>
        </template>
      </span>
      <span
        v-if="subtitle"
        class="truncate text-(--text-tertiary) text-(length:--text-small)"
      >{{ subtitle }}</span>
    </span>
    <span
      v-if="trailingCount"
      class="shrink-0 rounded-full bg-(--bg-card) px-(--spacing-xs) text-(--text-tertiary) text-(length:--text-tiny)"
    >{{ trailingCount }}</span>
    <span
      v-if="selected"
      class="shrink-0 text-(--text-tertiary) text-(length:--text-small)"
      aria-hidden="true"
    >↵</span>
  </div>
</template>
