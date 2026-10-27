<script setup lang="ts">
// Ported from AnalyticsShimmerView.swift:1-106 (docs/spec/analytics.md §8): header, 8 card
// placeholders (same 4-col grid as the real stat cards), three single chart placeholders at
// heights 220/240/200, then two side-by-side placeholders at height 280. Reuses ShimmerBlock.vue
// (M2-07) rather than hand-rolling new shimmer CSS, the same composition every other feature's own
// skeleton (e.g. ConfigSkeleton.vue) already uses.
import ShimmerBlock from '@/components/ShimmerBlock.vue';
</script>

<template>
  <div class="flex flex-col gap-(--spacing-lg)">
    <ShimmerBlock
      class="h-8 w-50"
      rounded="sm"
    />
    <div class="grid grid-cols-4 gap-(--spacing-md)">
      <ShimmerBlock
        v-for="n in 8"
        :key="n"
        class="h-27.5"
        rounded="md"
      />
    </div>
    <ShimmerBlock
      v-for="h in [220, 240, 200]"
      :key="h"
      :style="{ height: `${h}px` }"
      rounded="md"
    />
    <div class="grid grid-cols-2 gap-(--spacing-md)">
      <ShimmerBlock
        class="h-70"
        rounded="md"
      />
      <ShimmerBlock
        class="h-70"
        rounded="md"
      />
    </div>
  </div>
</template>
