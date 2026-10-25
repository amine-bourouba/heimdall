<script setup lang="ts">
// Ported from ConfigSkeletonView.swift:3-100. Grid: 2 columns x 3 cards; list: 4 cards; each card
// mimics a title bar, two body lines and two badge chips. Reuses ShimmerBlock.vue (M2-04) rather
// than hand-rolling new shimmer CSS.
import type { Layout } from '@/ipc/bindings';
import ShimmerBlock from '@/components/ShimmerBlock.vue';

defineProps<{ layout: Layout }>();
</script>

<template>
  <div :class="layout === 'grid' ? 'grid grid-cols-2 gap-(--spacing-md)' : 'flex flex-col gap-(--spacing-sm)'">
    <div
      v-for="n in layout === 'grid' ? 6 : 4"
      :key="n"
      class="flex flex-col gap-(--spacing-sm) rounded-md border border-(--border) p-(--spacing-lg)"
    >
      <ShimmerBlock
        class="h-4 w-1/2"
        rounded="sm"
      />
      <ShimmerBlock
        class="h-3 w-full"
        rounded="xs"
      />
      <ShimmerBlock
        class="h-3 w-4/5"
        rounded="xs"
      />
      <div class="flex gap-(--spacing-xs)">
        <ShimmerBlock
          class="h-5 w-16"
          rounded="sm"
        />
        <ShimmerBlock
          class="h-5 w-16"
          rounded="sm"
        />
      </div>
    </div>
  </div>
</template>
