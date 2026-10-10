<script setup lang="ts">
// Ported from ProjectSessionsView.swift (lines 1-181): shown in the detail pane when a project,
// but no specific session, is selected. Takes `project` as a prop (matching the real
// `ProjectSessionsView(project:)` signature) rather than reading `sessionStore.currentProject`
// itself -- wiring this into `SessionsPage.vue`'s own detail-pane state machine (no selection /
// project selected / session selected / loading) is a later integration task's job (flagged as a
// follow-up by M2-15, still open), not this one's; this component only needs a project to render.
//
// `filterQuery` is this component's own third, independent search layer -- on top of M2-14's
// `sidebarSearchQuery` and M2-15's own `listSearchQuery`, exactly matching the real Swift source
// (`ProjectSessionsView`'s own local `@State filterQuery`, unrelated to either). The reveal
// stagger animation (`shimmerReveal`, 50ms sleep then an 0.4s ease-out, delays capped at
// 0.28s/0.27s) is not ported: it is a purely cosmetic mount transition with no accessibility or
// correctness implication, and no shared "shimmer reveal" primitive exists yet in this port to
// reuse -- building one from scratch for a single screen's own entrance animation is out of
// proportion to this task's own AC, which only asks the spec to cover both layouts and sub-agent
// sessions.
import { computed, ref } from 'vue';
import AppIcon from '@/components/AppIcon.vue';
import { fuzzyMatch } from '@/lib/fuzzyMatch';
import { usePrefsStore } from '@/stores/prefs';
import SessionCard from './SessionCard.vue';
import type { ProjectWithSessions } from './store';

const props = defineProps<{ project: ProjectWithSessions }>();

const prefsStore = usePrefsStore();
const filterQuery = ref('');

const layout = computed(() => prefsStore.prefs?.sessionLayout ?? 'grid');

const filteredSessions = computed(() => {
  const q = filterQuery.value.trim();
  if (q === '') return props.project.sessions;

  return props.project.sessions
    .map((session) => {
      const best = Math.max(
        fuzzyMatch(session.title, q)?.score ?? 0,
        fuzzyMatch(session.preview ?? '', q)?.score ?? 0,
        fuzzyMatch(session.id, q)?.score ?? 0,
      );
      return { session, best };
    })
    .filter(({ best }) => best > 0)
    .sort((a, b) => b.best - a.best)
    .map(({ session }) => session);
});

async function toggleLayout() {
  await prefsStore.update({ sessionLayout: layout.value === 'grid' ? 'list' : 'grid' });
}
</script>

<template>
  <div class="flex h-full flex-col bg-(--bg-app)">
    <div class="border-b border-(--border) px-(--spacing-xxxl) py-(--spacing-xl)">
      <div class="flex items-center gap-(--spacing-md)">
        <div class="flex h-9 w-9 items-center justify-center rounded-md bg-(--color-green)/15">
          <AppIcon
            name="folder.fill"
            class="text-(--color-green)"
          />
        </div>
        <div class="flex flex-col gap-(--spacing-xxs)">
          <h1 class="text-(length:--text-heading) text-(--text-primary)">
            {{ project.name }}
          </h1>
          <span class="self-start rounded-full bg-(--bg-elevated) px-(--spacing-sm) py-(--spacing-xxs) text-(length:--text-tiny) text-(--text-tertiary)">
            {{ project.sessions.length }} {{ project.sessions.length === 1 ? 'session' : 'sessions' }}
          </span>
        </div>
      </div>
      <p class="mt-(--spacing-sm) text-(length:--text-caption) text-(--text-secondary)">
        Claude Code sessions for this project
      </p>
    </div>

    <div class="flex items-center justify-end gap-(--spacing-sm) border-b border-(--border) px-(--spacing-xxl) py-(--spacing-sm)">
      <input
        v-model="filterQuery"
        type="text"
        placeholder="Find in Sessions…"
        aria-label="Find in Sessions"
        class="w-[200px] rounded-sm bg-(--bg-card) px-(--spacing-sm) py-(--spacing-xs) text-(length:--text-caption) text-(--text-primary) outline-none"
      >
      <button
        type="button"
        title="Toggle layout"
        @click="toggleLayout"
      >
        <AppIcon
          :name="layout === 'grid' ? 'list.bullet' : 'square.grid.2x2'"
          size="sm"
          class="text-(--text-tertiary)"
        />
      </button>
    </div>

    <div class="min-h-0 flex-1 overflow-y-auto px-(--spacing-xxl) py-(--spacing-lg)">
      <div
        v-if="filteredSessions.length === 0 && filterQuery.trim() !== ''"
        class="flex h-full flex-col items-center justify-center gap-(--spacing-sm) text-(--text-tertiary)"
      >
        <AppIcon
          name="magnifyingglass"
          size="lg"
        />
        <p class="text-(length:--text-caption)">
          No sessions match "{{ filterQuery }}"
        </p>
        <p class="text-(length:--text-tiny)">
          Try a different search term
        </p>
      </div>

      <div
        v-else-if="layout === 'grid'"
        class="grid grid-cols-2 gap-(--spacing-lg)"
      >
        <SessionCard
          v-for="session in filteredSessions"
          :key="session.id"
          :session="session"
          layout="grid"
        />
      </div>
      <div
        v-else
        class="flex flex-col gap-(--spacing-md)"
      >
        <SessionCard
          v-for="session in filteredSessions"
          :key="session.id"
          :session="session"
          layout="list"
        />
      </div>
    </div>
  </div>
</template>
