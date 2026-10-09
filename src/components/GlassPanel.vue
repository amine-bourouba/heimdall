<script setup lang="ts">
// Ported from GlassBackground.swift: a shape wrapper around the `.glass` utility (M2-02's
// main.css), which already carries the real backdrop-filter/@supports fallback
// (docs/design/webkitgtk-fallbacks.md) -- this component only adds the shape.
withDefaults(
  defineProps<{
    shape?: 'rect' | 'circle' | 'capsule';
  }>(),
  { shape: 'rect' },
);
</script>

<template>
  <div
    class="glass"
    :class="{ 'rounded-md': shape === 'rect', 'rounded-full': shape === 'circle' || shape === 'capsule' }"
  >
    <slot />
  </div>
</template>
