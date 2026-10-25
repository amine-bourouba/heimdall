<script setup lang="ts">
// Ported from SessionFacetsCard.swift and SessionFacets.swift:50-91 (M3-06). Self-contained:
// fetches its own data from `sessionId` rather than going through session/store.ts (not in this
// task's own Outputs list), matching the plans/memory feature stores' own precedent of a
// component owning its own fetch when nothing else needs the same data.
//
// FlowLayout.swift's hand-rolled wrapping layout (`:256-307`) has no equivalent needed here: CSS
// `flex-wrap` already does exactly that natively.
import { computed, ref, watch } from 'vue';
import { ipc } from '@/ipc/client';
import AppIcon from '@/components/AppIcon.vue';
import MarkdownView from '@/lib/markdown/MarkdownView.vue';

const props = defineProps<{ sessionId: string }>();

type SessionFacets = NonNullable<Awaited<ReturnType<typeof ipc.getSessionFacets>>>;

const facets = ref<SessionFacets | null>(null);
const isExpanded = ref(true);

// A generation token, same reasoning as search/store.ts's own staleness guard: switching to a
// second session before the first one's fetch resolves must not let that stale response
// overwrite the newer session's (now-empty-so-far) state once it lands late.
let generation = 0;
async function load(sessionId: string) {
  const thisGeneration = ++generation;
  const result = await ipc.getSessionFacets(sessionId);
  if (thisGeneration !== generation) return;
  facets.value = result;
  isExpanded.value = true;
}

watch(() => props.sessionId, load, { immediate: true });

// Ported from SessionFacets.swift:56-90. `_LABELS` cover the Swift switch's own named cases;
// anything else falls through to the same `_`-to-space, title-cased default every one of Swift's
// own label getters shares.
function titleCase(value: string): string {
  return value.replace(/_/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase());
}

const OUTCOME_LABELS: Record<string, string> = {
  success: 'Success',
  partially_achieved: 'Partial',
  unclear_from_transcript: 'Unclear',
  failure: 'Failed',
};
// Same `text-(--color-x) bg-(--color-x)/10` pair already used for every other colored badge in
// this codebase (e.g. SessionCard.vue's model/token pills) -- a class pair, not a raw color
// value, so the opacity modifier stays a plain Tailwind utility instead of a hand-built
// color-mix() string.
const OUTCOME_CLASSES: Record<string, string> = {
  success: 'text-(--color-green) bg-(--color-green)/10',
  partially_achieved: 'text-(--color-orange) bg-(--color-orange)/10',
  failure: 'text-(--color-red) bg-(--color-red)/10',
};

const HELPFULNESS_LABELS: Record<string, string> = {
  very_helpful: 'Very Helpful',
  slightly_helpful: 'Slightly Helpful',
  not_helpful: 'Not Helpful',
  harmful: 'Harmful',
};
const HELPFULNESS_CLASSES: Record<string, string> = {
  very_helpful: 'text-(--color-green) bg-(--color-green)/10',
  slightly_helpful: 'text-(--accent) bg-(--accent-dim)',
  not_helpful: 'text-(--color-orange) bg-(--color-orange)/10',
  harmful: 'text-(--color-red) bg-(--color-red)/10',
};
const DEFAULT_BADGE_CLASSES = 'text-(--text-tertiary) bg-(--bg-elevated)';

const outcomeLabel = computed(() => (facets.value ? OUTCOME_LABELS[facets.value.outcome] ?? titleCase(facets.value.outcome) : ''));
const outcomeClasses = computed(() => (facets.value ? OUTCOME_CLASSES[facets.value.outcome] ?? DEFAULT_BADGE_CLASSES : ''));
const helpfulnessLabel = computed(() =>
  facets.value ? HELPFULNESS_LABELS[facets.value.claudeHelpfulness] ?? titleCase(facets.value.claudeHelpfulness) : '',
);
const helpfulnessClasses = computed(() => (facets.value ? HELPFULNESS_CLASSES[facets.value.claudeHelpfulness] ?? DEFAULT_BADGE_CLASSES : ''));
const sessionTypeLabel = computed(() => (facets.value ? titleCase(facets.value.sessionType) : ''));

function sortedByCountDesc(counts: Record<string, number>): { name: string; count: number }[] {
  return Object.entries(counts)
    .filter(([, count]) => count > 0)
    .sort((a, b) => b[1] - a[1])
    .map(([name, count]) => ({ name, count }));
}

const goalCategories = computed(() => (facets.value ? sortedByCountDesc(facets.value.goalCategories) : []));
const frictionItems = computed(() => (facets.value ? sortedByCountDesc(facets.value.frictionCounts) : []));
const totalFriction = computed(() => frictionItems.value.reduce((sum, f) => sum + f.count, 0));
</script>

<template>
  <div
    v-if="facets"
    class="overflow-hidden rounded-md border bg-(--bg-card)"
  >
    <button
      type="button"
      class="flex w-full items-center gap-(--spacing-sm) px-(--spacing-lg) py-(--spacing-md) text-left"
      :aria-expanded="isExpanded"
      :title="isExpanded ? 'Collapse AI summary' : 'Expand AI summary'"
      @click="isExpanded = !isExpanded"
    >
      <AppIcon
        name="sparkles"
        size="sm"
        class="text-(--color-purple)"
      />
      <span class="font-semibold text-(--text-secondary) text-(length:--text-small)">AI Summary</span>
      <span
        class="rounded-xs px-(--spacing-sm) py-(--spacing-xxs) text-(length:--text-micro)"
        :class="outcomeClasses"
      >{{ outcomeLabel }}</span>
      <span
        class="rounded-xs px-(--spacing-sm) py-(--spacing-xxs) text-(length:--text-micro)"
        :class="helpfulnessClasses"
      >{{ helpfulnessLabel }}</span>
      <div class="flex-1" />
      <span class="rounded-xs bg-(--bg-elevated) px-(--spacing-sm) py-(--spacing-xxs) text-(--text-tertiary) text-(length:--text-micro)">{{ sessionTypeLabel }}</span>
      <AppIcon
        name="chevron.right"
        size="sm"
        class="text-(--text-tertiary) transition-transform"
        :class="{ 'rotate-90': isExpanded }"
      />
    </button>

    <div
      v-if="isExpanded"
      class="flex flex-col gap-(--spacing-md) border-t px-(--spacing-lg) py-(--spacing-lg)"
    >
      <MarkdownView
        v-if="facets.briefSummary"
        :content="facets.briefSummary"
      />

      <div
        v-if="facets.underlyingGoal"
        class="flex flex-col gap-(--spacing-xs)"
      >
        <span class="font-semibold text-(--text-tertiary) text-(length:--text-micro)">Goal</span>
        <p class="text-(--text-primary) text-(length:--text-caption)">
          {{ facets.underlyingGoal }}
        </p>
      </div>

      <div
        v-if="goalCategories.length > 0"
        class="flex flex-wrap gap-(--spacing-xs)"
      >
        <span
          v-for="category in goalCategories"
          :key="category.name"
          class="flex items-center gap-(--spacing-xxs) rounded-xs bg-(--accent-dim) px-(--spacing-sm) py-(--spacing-xxs) text-(--accent) text-(length:--text-micro)"
        >
          {{ titleCase(category.name) }}
          <span
            v-if="category.count > 1"
            class="opacity-70"
          >{{ category.count }}</span>
        </span>
      </div>

      <div
        v-if="totalFriction > 0"
        class="flex flex-col gap-(--spacing-xs)"
      >
        <span class="flex items-center gap-(--spacing-xs) font-semibold text-(--text-tertiary) text-(length:--text-micro)">
          <AppIcon
            name="exclamationmark.triangle"
            size="sm"
            class="text-(--color-orange)"
          />
          Friction
        </span>
        <div class="flex flex-wrap gap-(--spacing-xs)">
          <span
            v-for="item in frictionItems"
            :key="item.name"
            class="flex items-center gap-(--spacing-xxs) rounded-xs bg-(--color-orange)/10 px-(--spacing-sm) py-(--spacing-xxs) text-(--color-orange) text-(length:--text-micro)"
          >
            {{ titleCase(item.name) }}
            <span
              v-if="item.count > 1"
              class="opacity-70"
            >{{ item.count }}</span>
          </span>
        </div>
        <p
          v-if="facets.frictionDetail"
          class="text-(--text-tertiary) text-(length:--text-tiny)"
        >
          {{ facets.frictionDetail }}
        </p>
      </div>
    </div>
  </div>
</template>
