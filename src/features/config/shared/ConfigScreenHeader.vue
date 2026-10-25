<script setup lang="ts">
// Ported from ConfigScreenHeader.swift:3-57. Purely static per-load content -- not itself
// filterable or actionable.
import AppIcon from '@/components/AppIcon.vue';

defineProps<{ icon: string; title: string; count: number; description: string }>();
</script>

<template>
  <div class="flex items-start gap-(--spacing-md) px-(--spacing-xxxl) py-(--spacing-lg)">
    <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-md bg-(--accent)/15">
      <AppIcon
        :name="icon"
        size="md"
        class="text-(--accent)"
      />
    </div>
    <div class="flex flex-col gap-(--spacing-xxs)">
      <div class="flex items-center gap-(--spacing-sm)">
        <h1 class="text-(length:--text-large) font-semibold">
          {{ title }}
        </h1>
        <span class="rounded-full bg-(--bg-elevated) px-(--spacing-sm) py-px text-(length:--text-tiny) text-(--text-tertiary)">{{ count }}</span>
      </div>
      <p class="text-(length:--text-small) text-(--text-tertiary)">
        {{ description }}
      </p>
    </div>
  </div>
</template>
