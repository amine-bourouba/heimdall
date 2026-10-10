<script setup lang="ts">
// Ported from SessionSkeletonView.swift: a header placeholder plus 6 alternating bubble
// placeholders (odd index = assistant, getting an extra tool-block-shaped rect at index 1 and 3).
// Swift's own `shimmer(cornerRadius:)` sweep has no direct Heimdall port -- Tailwind's own
// `animate-pulse` is the native-equivalent "loading shimmer" utility already available, used
// directly rather than hand-rolling the sweep animation (no new CSS needed).
const BUBBLES = Array.from({ length: 6 }, (_, i) => i);

// SessionSkeletonView.swift:150-166 -- per-bubble line-width ratios, by index.
const USER_RATIOS: Record<number, number[]> = { 0: [0.85, 0.65], 2: [0.75, 0.55], 4: [0.7, 0.8, 0.4] };
const ASSISTANT_RATIOS: Record<number, number[]> = { 1: [0.9, 0.85, 0.5], 3: [0.85, 0.95, 0.65], 5: [0.8, 0.7] };

function lineRatios(index: number): number[] {
  if (index % 2 === 0) return USER_RATIOS[index] ?? [0.9, 0.6];
  return ASSISTANT_RATIOS[index] ?? [0.9, 0.75, 0.55];
}
</script>

<template>
  <div class="flex h-full flex-col overflow-hidden">
    <div class="flex items-center justify-between gap-(--spacing-lg) border-b border-(--border)/30 px-(--spacing-xxxl) py-(--spacing-lg)">
      <div class="flex min-w-0 flex-col gap-(--spacing-sm)">
        <div class="h-4 w-[340px] animate-pulse rounded-xs bg-(--bg-card)" />
        <div class="flex items-center gap-(--spacing-sm)">
          <div class="h-[18px] w-[100px] animate-pulse rounded-xs bg-(--bg-card)" />
          <div class="h-[18px] w-[110px] animate-pulse rounded-xs bg-(--bg-card)" />
          <div class="h-3.5 w-[130px] animate-pulse rounded-xs bg-(--bg-card)" />
        </div>
      </div>
      <div class="flex shrink-0 gap-(--spacing-sm)">
        <div
          v-for="w in [80, 110, 120, 70]"
          :key="w"
          class="h-7 animate-pulse rounded-sm bg-(--bg-card)"
          :style="{ width: `${w}px` }"
        />
      </div>
    </div>

    <div class="flex-1 overflow-hidden px-(--spacing-xxxl) py-(--spacing-lg)">
      <div
        v-for="index in BUBBLES"
        :key="index"
        class="mb-(--spacing-lg) flex"
        :class="index % 2 === 0 ? 'justify-end' : 'justify-start'"
      >
        <div
          class="w-3/4 rounded-md border border-(--border) p-(--spacing-lg)"
          :class="index % 2 === 0 ? 'bg-(--bg-elevated)' : 'bg-(--bg-card)'"
        >
          <div class="mb-(--spacing-sm) flex items-center gap-(--spacing-sm)">
            <div class="h-3 w-7 animate-pulse rounded-xs bg-(--bg-code)" />
            <div class="h-2.5 w-[50px] animate-pulse rounded-xs bg-(--bg-code)" />
          </div>
          <div
            v-for="(ratio, i) in lineRatios(index)"
            :key="i"
            class="mb-(--spacing-xs) h-3 animate-pulse rounded-xs bg-(--bg-code)"
            :style="{ width: `${ratio * 100}%` }"
          />
          <div
            v-if="index % 2 === 1 && (index === 1 || index === 3)"
            class="mt-(--spacing-sm) h-11 w-full animate-pulse rounded-md bg-(--bg-code)"
          />
        </div>
      </div>
    </div>
  </div>
</template>
