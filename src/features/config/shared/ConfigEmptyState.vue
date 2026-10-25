<script setup lang="ts">
// Ported from ConfigEmptyState.swift:3-30. One component, two flavours ("no items on disk" vs
// "no filter matches") -- both are just different icon/message/hint props from the caller, this
// component doesn't need to know which.
import AppIcon from '@/components/AppIcon.vue';

defineProps<{ icon: string; message: string; hint: string }>();
</script>

<template>
  <div class="flex h-full flex-col items-center justify-center gap-(--spacing-sm) text-center">
    <AppIcon
      :name="icon"
      size="xl"
      class="text-(--text-tertiary)/50"
    />
    <p class="font-medium text-(--text-secondary)">
      {{ message }}
    </p>
    <p class="rounded-full bg-(--bg-elevated) px-(--spacing-md) py-(--spacing-xs) text-(length:--text-tiny) text-(--text-tertiary)">
      {{ hint }}
    </p>
  </div>
</template>
