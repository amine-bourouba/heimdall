<script setup lang="ts">
// Ported from CardChrome.swift's `.cardChrome(isHovered:isSelected:)` view modifier -- a wrapper
// component here instead of a directive, since Vue has no exact equivalent of a chained
// SwiftUI view modifier.
withDefaults(
  defineProps<{
    isHovered?: boolean;
    isSelected?: boolean;
  }>(),
  { isHovered: false, isSelected: false },
);
</script>

<template>
  <div
    class="rounded-md border"
    :class="isHovered ? 'bg-(--bg-card-hover)' : 'bg-(--bg-card)'"
    :style="{ borderColor: isSelected ? 'var(--accent)' : isHovered ? 'var(--border-emphasis)' : 'var(--border)' }"
  >
    <slot />
  </div>
</template>
