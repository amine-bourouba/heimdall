<script setup lang="ts">
// Dispatches /config/:kind to the right placeholder page by ConfigKind (task M2-06). Added
// beyond the task's own literal Outputs list: the AC names one generic route pattern for all
// four slug-addressable config kinds, which needs exactly this lookup to resolve to a real
// component -- not listed by name in Outputs, but required for the route to do anything at all.
//
// `defineAsyncComponent`, not a static import (M3-12 necessary ripple): CommandsPage.vue/
// SkillsPage.vue/OutputStylesPage.vue went from trivial one-line placeholders to real pages
// pulling in ConfigItemDetail.vue -> MarkdownView.vue -> Shiki once M3-12 landed, and this file
// is itself statically imported from routes.ts -- unlike every other route there, which already
// lazy-loads its own page component. A static import here would drag the now-heavy config pages
// (and Shiki with them) into whatever chunk routes.ts belongs to; confirmed directly by `node
// scripts/check-bundle-size.mjs` failing on several Shiki per-language chunks with a static
// import here, and passing clean once converted to this lazy form.
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
