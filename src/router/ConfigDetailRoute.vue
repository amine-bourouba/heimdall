<script setup lang="ts">
// Dispatches /config/:kind/:scope/:slug the same way ConfigListRoute.vue dispatches the list
// route -- see that file's own header comment for why this exists beyond the task's literal
// Outputs list. The placeholder pages don't yet read :scope/:slug; the later tasks that replace
// them (docs/design/icon-map.md-style ownership noted in each feature's own index.ts) will.
import { computed } from 'vue';
import { useRoute } from 'vue-router';
import CommandsPage from '@/features/config/commands/CommandsPage.vue';
import SkillsPage from '@/features/config/skills/SkillsPage.vue';
import OutputStylesPage from '@/features/config/output-styles/OutputStylesPage.vue';
import SubAgentsPage from '@/features/config/subagents/SubAgentsPage.vue';

const pageByKind = {
  command: CommandsPage,
  skill: SkillsPage,
  outputStyle: OutputStylesPage,
  agent: SubAgentsPage,
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
