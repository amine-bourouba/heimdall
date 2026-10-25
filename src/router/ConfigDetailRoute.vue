<script setup lang="ts">
// Dispatches /config/:kind/:scope/:slug the same way ConfigListRoute.vue dispatches the list
// route -- see that file's own header comment for why this exists beyond the task's literal
// Outputs list, and for why these are lazy `defineAsyncComponent`s rather than static imports
// (M3-12 necessary ripple: the Shiki-pulling MarkdownView chain these pages now reach makes a
// static import here a real bundle-size regression, confirmed directly).
import { computed, defineAsyncComponent } from 'vue';
import { useRoute } from 'vue-router';

const pageByKind = {
  command: defineAsyncComponent(() => import('@/features/config/commands/CommandsPage.vue')),
  skill: defineAsyncComponent(() => import('@/features/config/skills/SkillsPage.vue')),
  outputStyle: defineAsyncComponent(() => import('@/features/config/output-styles/OutputStylesPage.vue')),
  agent: defineAsyncComponent(() => import('@/features/config/subagents/SubAgentsPage.vue')),
} as const;

const route = useRoute();
const page = computed(() => pageByKind[route.params.kind as keyof typeof pageByKind]);
</script>

<template>
  <component
    :is="page"
    v-if="page"
  />
  <p
    v-else
    class="text-(--text-tertiary)"
  >
    Unknown config kind: {{ route.params.kind }}
  </p>
</template>
