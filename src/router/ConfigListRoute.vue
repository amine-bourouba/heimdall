<script setup lang="ts">
// Dispatches /config/:kind to the right placeholder page by ConfigKind (task M2-06). Added
// beyond the task's own literal Outputs list: the AC names one generic route pattern for all
// four slug-addressable config kinds, which needs exactly this lookup to resolve to a real
// component -- not listed by name in Outputs, but required for the route to do anything at all.
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
