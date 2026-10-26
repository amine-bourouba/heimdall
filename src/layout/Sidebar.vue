<script setup lang="ts">
// Ported from SidebarView.swift: a vertical list of nav items with live counts. The Swift reference app splits
// `.main`/`.config` sections for visual grouping only (both render in the same VStack, back to
// back) -- `navigationItems.ts` (M2-06) already carries the one real order that matters, so no
// section split is reintroduced here.
//
// Not filtered on `available`: every item in the real catalog is `available: false` today (no
// feature past the router/placeholder stage exists yet), so filtering would render an empty,
// unusable sidebar. Nothing in this task's own AC asks for the filter either -- it names only
// "items and order", and each placeholder page already renders an honest "not built yet" state
// when visited, so showing every item (clickable, not disabled) is the more useful behaviour
// until individual features actually start flipping `available` to true.
import { onMounted, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { ipc, onDataChanged } from '@/ipc/client';
import type { Area, ProjectContext, SidebarCounts } from '@/ipc/bindings';
import { navigationItems } from '@/router/navigationItems';
import { usePrefsStore } from '@/stores/prefs';
import { useSearchStore } from '@/features/search/store';
import { useSessionStore } from '@/features/session/store';
import { useKeyboardNav } from '@/composables/useKeyboardNav';
import type { EscapeLayer } from '@/composables/useKeyboardNav';
import AppIcon from '@/components/AppIcon.vue';

// Only the areas with a real, file-driven count in `SidebarCounts` -- "models" has no Area at
// all (it is `provider.supportedModels.count` in the Swift reference app, a static built-in list, never loaded
// from disk), confirmed directly against `classify.rs`'s own `Area` enum rather than guessed.
const COUNTED_AREAS: Area[] = ['todos', 'history', 'commands', 'skills', 'plans', 'outputStyles', 'agents', 'hooks', 'mcp', 'plugins', 'memory'];

// `SidebarCounts`' own keys don't all match an `Area` variant's name 1:1 (`subAgents` vs `agents`).
const COUNT_KEY_BY_ID: Partial<Record<string, keyof SidebarCounts>> = {
  todos: 'todos',
  history: 'history',
  commands: 'commands',
  skills: 'skills',
  plans: 'plans',
  outputStyles: 'outputStyles',
  subAgents: 'subAgents',
  hooks: 'hooks',
  mcpServers: 'mcpServers',
  plugins: 'plugins',
  memory: 'memory',
  models: 'models',
};

const router = useRouter();
const route = useRoute();
const prefsStore = usePrefsStore();
const searchStore = useSearchStore();
const sessionStore = useSessionStore();
const counts = ref<SidebarCounts | null>(null);
const keyboardIndex = ref(0);

// KeyboardNavigationModifier.swift's own handleEscape() order (M3-19, the real wiring
// `useKeyboardNav.ts`'s own header comment used to say was still missing): session search, then
// tool filter, then config detail, then the selected session. The latter two are route-driven --
// Heimdall has no `activeConfigDetail`/`selectedSession` store flags, routing IS that state.
const escapeLayers: EscapeLayer[] = [
  {
    isActive: () => sessionStore.isSessionSearchActive,
    dismiss: () => {
      sessionStore.sessionSearchQuery = '';
      sessionStore.isSessionSearchActive = false;
    },
  },
  {
    isActive: () => sessionStore.isToolFilterActive,
    dismiss: () => {
      sessionStore.isToolFilterActive = false;
      sessionStore.activeToolFilters.clear();
    },
  },
  {
    isActive: () => route.name === 'config-detail',
    dismiss: () => void router.push({ name: 'config-list', params: { kind: route.params.kind } }),
  },
  {
    // Swift's own `selectedSession = nil` deselects the session but keeps the project context --
    // dropping only `sessionId` (optional on this same route) is the faithful equivalent, not a
    // full jump to the bare `/sessions` list, which would also lose `projectId`.
    isActive: () => route.name === 'session-detail' && !!route.params.sessionId,
    dismiss: () => void router.push({ name: 'session-detail', params: { projectId: route.params.projectId } }),
  },
];

function projectContext(): ProjectContext | null {
  // `currentProject` (the selected session's own project) has no Heimdall equivalent yet -- no
  // session-selection UI exists before M3, so the only real context today is a picked folder.
  return prefsStore.prefs?.configProjectPath ? { kind: 'pickedFolder' } : null;
}

async function refreshCounts() {
  counts.value = await ipc.getSidebarCounts(projectContext());
}

function countFor(id: string): number {
  const key = COUNT_KEY_BY_ID[id];
  return key && counts.value ? counts.value[key] : 0;
}

onMounted(() => {
  void refreshCounts();
  void onDataChanged(COUNTED_AREAS, () => void refreshCounts());
});

useKeyboardNav({
  escapeLayers,
  sidebarItemCount: () => navigationItems.length,
  sidebarIndex: keyboardIndex,
  // KeyboardNavigationModifier.swift:17-19's own guard (the composable's own doc comment on this
  // option): while the search overlay is open, its arrow keys drive its own list, not the
  // sidebar's hidden-behind-it one (M3-01).
  isSuppressed: () => searchStore.isOpen,
  onSidebarActivate: () => router.push(navigationItems[keyboardIndex.value].to),
});
</script>

<template>
  <nav
    aria-label="Primary"
    class="h-full w-[260px] min-w-[220px] max-w-[300px] overflow-y-auto bg-(--bg-sidebar) pt-(--titlebar-inset)"
  >
    <ul class="flex flex-col gap-(--spacing-xxs) p-(--spacing-md)">
      <li
        v-for="(item, index) in navigationItems"
        :key="item.id"
      >
        <RouterLink
          :to="item.to"
          active-class="bg-(--accent-dim) text-(--accent)"
          class="flex items-center justify-between gap-(--spacing-sm) rounded-sm px-(--spacing-sm) py-(--spacing-xs) text-(--text-secondary) text-(length:--text-caption-medium)"
          :class="{ 'ring-1 ring-(--accent)/50': keyboardIndex === index }"
        >
          <span class="flex items-center gap-(--spacing-sm)">
            <AppIcon
              :name="item.icon"
              size="sm"
            />
            {{ item.label }}
          </span>
          <span
            v-if="countFor(item.id) > 0"
            class="rounded-full bg-(--bg-card) px-(--spacing-xs) text-(--text-tertiary) text-(length:--text-tiny)"
          >
            {{ countFor(item.id) }}
          </span>
        </RouterLink>
      </li>
    </ul>
  </nav>
</template>
