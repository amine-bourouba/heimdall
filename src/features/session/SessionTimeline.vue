<script setup lang="ts">
// Ported from SessionTimelineView.swift -- a Gantt chart of one session group's parent + its
// sub-agents over time, NOT a tool-call list (this task's own Goal prose says "lists tool calls",
// but the one named Input file it actually cites is this Gantt view; read directly rather than
// guessed, per this session's own established practice of trusting the real source over a task
// file's own gloss when the two disagree).
//
// `SessionGroup` (the parent/agents grouping `entries` is built from) does not exist anywhere in
// Heimdall -- confirmed directly: no such type in the IPC contract, and M2-15 already documented
// not building the equivalent grouping logic. So this component takes already-built `entries`
// as a prop rather than a `SessionGroup`/`Session[]` -- real, working Gantt-chart rendering, just
// with no real caller yet to produce grouped entries from a session's own data (flagged in
// Follow-ups, same shape as several other "built but not yet fed real data" pieces this session).
//
// No chart library: ECharts' Gantt support needs its `custom` series' `renderItem` API, real
// setup cost for a view with no real caller and no acceptance criterion gating it at all (this
// task's one AC is about the context card's sparkline, not this file) -- plain positioned `div`s
// over the known start/end range is the simplest thing that is still a real, correct Gantt bar.
import { computed } from 'vue';
import AppIcon from '@/components/AppIcon.vue';

export interface TimelineEntry {
  id: string;
  projectId: string;
  label: string;
  startedAt: string;
  endedAt: string | null;
  isParent: boolean;
  agentType: string | null;
}

const props = defineProps<{ entries: TimelineEntry[] }>();
defineEmits<{ select: [entry: TimelineEntry] }>();

const ROW_HEIGHT = 36;

const range = computed(() => {
  const starts = props.entries.map((e) => new Date(e.startedAt).getTime());
  const ends = props.entries.map((e) => new Date(e.endedAt ?? Date.now()).getTime());
  const min = Math.min(...starts);
  const max = Math.max(...ends);
  return { min, span: Math.max(max - min, 1) };
});

// The chart row reserves its first 35% for the label; the bar lives in the remaining 65%,
// scaled by how far through the whole group's time range this entry's own span falls.
const LABEL_WIDTH_PERCENT = 35;
const BAR_AREA_PERCENT = 100 - LABEL_WIDTH_PERCENT;

function barStyle(entry: TimelineEntry): { left: string; width: string } {
  const start = new Date(entry.startedAt).getTime();
  const end = new Date(entry.endedAt ?? Date.now()).getTime();
  const left = LABEL_WIDTH_PERCENT + ((start - range.value.min) / range.value.span) * BAR_AREA_PERCENT;
  const width = Math.max(((end - start) / range.value.span) * BAR_AREA_PERCENT, 0.5);
  return { left: `${left}%`, width: `${width}%` };
}

function colorFor(entry: TimelineEntry): string {
  if (entry.isParent) return 'var(--accent)';
  switch (entry.agentType?.toLowerCase()) {
    case 'explore':
      return 'var(--color-blue)';
    case 'plan':
      return 'var(--color-green)';
    default:
      return 'var(--color-purple)';
  }
}

const agentCount = computed(() => props.entries.filter((e) => !e.isParent).length);
</script>

<template>
  <div class="flex h-full flex-col bg-(--bg-app)">
    <div class="flex items-center gap-(--spacing-sm) border-b border-(--border)/30 px-(--spacing-lg) py-(--spacing-md)">
      <AppIcon
        name="chart.xyaxis.line"
        size="sm"
        class="text-(--text-primary)"
      />
      <span class="text-(length:--text-body-medium) text-(--text-primary)">Session Timeline</span>
      <span class="rounded-full bg-(--color-purple)/15 px-(--spacing-sm) py-px text-(length:--text-tiny) text-(--color-purple)">{{ agentCount }} agents</span>
      <div class="flex-1" />
      <slot name="trailing" />
    </div>

    <div class="flex-1 overflow-y-auto p-(--spacing-lg)">
      <div
        class="rounded-md border border-(--border) bg-(--bg-card) p-(--spacing-lg)"
        :style="{ height: `${Math.max(entries.length * ROW_HEIGHT, 120)}px` }"
      >
        <div
          v-for="entry in entries"
          :key="entry.id"
          class="relative mb-1 h-6 text-(length:--text-tiny) text-(--text-tertiary)"
        >
          <span class="absolute left-0 top-0 max-w-[30%] truncate">{{ entry.label }}</span>
          <button
            type="button"
            class="absolute top-0 h-5 rounded-xs"
            :style="{ ...barStyle(entry), backgroundColor: colorFor(entry) }"
            :title="entry.label"
            @click="$emit('select', entry)"
          />
        </div>
      </div>

      <div class="mt-(--spacing-md) flex gap-(--spacing-md) text-(length:--text-tiny) text-(--text-tertiary)">
        <span class="flex items-center gap-1"><span class="h-2 w-2 rounded-full bg-(--accent)" />Parent</span>
        <span class="flex items-center gap-1"><span class="h-2 w-2 rounded-full bg-(--color-blue)" />Explore</span>
        <span class="flex items-center gap-1"><span class="h-2 w-2 rounded-full bg-(--color-green)" />Plan</span>
        <span class="flex items-center gap-1"><span class="h-2 w-2 rounded-full bg-(--color-purple)" />Other</span>
      </div>
    </div>
  </div>
</template>
