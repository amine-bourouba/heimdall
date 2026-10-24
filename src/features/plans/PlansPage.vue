<script setup lang="ts">
// Ported from PlansListView.swift (M3-02). Plan cards are rendered inline here, not as a
// separate component: this task's own Outputs list names only PlansPage.vue and PlanDetail.vue,
// no third "PlanCard.vue" (unlike M3-02's own HistoryRow.vue). Delete (Swift's own `deletePlan`/
// `deleteConfigItem`) is not ported: not in this task's Goal or AC, and `deleteConfigItem` has no
// Output file here to call it from.
import { computed, onMounted } from 'vue';
import type { ConfigItemSummary } from '@/ipc/bindings';
import AppIcon from '@/components/AppIcon.vue';
import FilterField from '@/components/FilterField.vue';
import ShimmerBlock from '@/components/ShimmerBlock.vue';
import { usePlansStore } from './store';
import PlanDetail from './PlanDetail.vue';

const store = usePlansStore();

onMounted(() => void store.load());

const countText = computed(() => `${store.plans.length} ${store.plans.length === 1 ? 'plan' : 'plans'}`);

function openPlan(plan: ConfigItemSummary) {
  void store.open(plan.ref);
}
</script>

<template>
  <PlanDetail
    v-if="store.selected"
    :plan="store.selected"
    @back="store.close()"
  />

  <div
    v-else
    class="flex h-full flex-col"
  >
    <div class="flex items-center gap-(--spacing-md) border-b px-(--spacing-xxxl) py-(--spacing-xl)">
      <div class="flex h-9 w-9 items-center justify-center rounded-md bg-(--accent-dim)">
        <AppIcon
          name="list.bullet.clipboard.fill"
          class="text-(--accent)"
        />
      </div>
      <div class="flex flex-col gap-(--spacing-xxs)">
        <h1 class="text-(--text-primary) text-(length:--text-heading)">
          Plans
        </h1>
        <span class="self-start rounded-full bg-(--bg-elevated) px-(--spacing-sm) py-(--spacing-xxs) text-(--text-tertiary) text-(length:--text-tiny)">{{ countText }}</span>
      </div>
      <div class="flex-1" />
      <FilterField
        v-model="store.query"
        placeholder="Find in Plans…"
        class="w-50"
      />
      <button
        type="button"
        class="shrink-0 text-(--text-tertiary)"
        title="Toggle layout"
        aria-label="Toggle layout"
        @click="store.toggleLayout()"
      >
        <AppIcon :name="store.layout === 'grid' ? 'list.bullet' : 'square.grid.2x2'" />
      </button>
    </div>

    <div class="flex-1 overflow-y-auto px-(--spacing-xxxl) py-(--spacing-lg)">
      <div
        v-if="store.isLoading"
        class="grid gap-(--spacing-md)"
        :class="store.layout === 'grid' ? 'grid-cols-2' : 'grid-cols-1'"
      >
        <ShimmerBlock
          v-for="n in 6"
          :key="n"
          rounded="md"
          class="h-20"
        />
      </div>

      <div
        v-else-if="store.plans.length === 0"
        class="flex h-full flex-col items-center justify-center gap-(--spacing-xs) text-(--text-tertiary)"
      >
        <AppIcon
          name="list.bullet.clipboard.fill"
          size="lg"
        />
        <p>No plans found</p>
        <p class="text-(length:--text-small)">
          ~/.claude/plans/
        </p>
      </div>

      <div
        v-else-if="store.filteredPlans.length === 0"
        class="flex h-full flex-col items-center justify-center gap-(--spacing-xs) text-(--text-tertiary)"
      >
        <AppIcon
          name="magnifyingglass"
          size="lg"
        />
        <p>No plans match "{{ store.query }}"</p>
      </div>

      <div
        v-else
        class="grid gap-(--spacing-md)"
        :class="store.layout === 'grid' ? 'grid-cols-2' : 'grid-cols-1'"
      >
        <button
          v-for="plan in store.filteredPlans"
          :key="`${plan.ref.kind}:${plan.ref.slug}`"
          type="button"
          class="rounded-md border bg-(--bg-card) p-(--spacing-lg) text-left hover:bg-(--bg-card-hover)"
          @click="openPlan(plan)"
        >
          <p class="text-(--text-primary) text-(length:--text-body-medium)">
            {{ plan.name }}
          </p>
          <p class="mt-(--spacing-xs) text-(--text-tertiary) text-(length:--text-tiny)">
            {{ plan.ref.slug }}.md
          </p>
        </button>
      </div>
    </div>
  </div>
</template>
