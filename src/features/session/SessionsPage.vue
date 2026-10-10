<!-- Ported from SessionsNavigationView.swift's own `body` (`HStack` of list pane + divider +
     detail pane, lines 53-66). This task (M2-15) owns only the list pane (`SessionsNavigation.vue`);
     the detail pane's own state machine (no selection / project selected / session selected /
     loading / timeline) is M2-16's (project sessions) and M2-20's (conversation view) job --
     kept here as the same minimal placeholder the stub rule asks for, not a guess at logic those
     tasks haven't built yet. -->
<script setup lang="ts">
import SessionsNavigation from './SessionsNavigation.vue';
</script>

<template>
  <div class="flex h-full">
    <div class="h-full w-[280px] min-w-[220px] max-w-[380px]">
      <SessionsNavigation />
    </div>
    <main class="flex flex-1 items-center justify-center text-(--text-tertiary)">
      <p>Select a session from the list to view its content.</p>
    </main>
  </div>
</template>
