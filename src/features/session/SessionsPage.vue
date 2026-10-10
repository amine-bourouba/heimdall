<!-- Ported from SessionsNavigationView.swift's own `body` (`HStack` of list pane + divider +
     detail pane, lines 53-66) and `detailPane`'s state machine (session-views.md §3): no
     selection -> empty state; project selected, no session -> ProjectSessions (M2-16); session
     selected -> SessionSkeleton while loading, else SessionDetail (both M2-20's own job, finally
     closing the gap M2-16's own comment here left for it). The timeline/file-history branches
     are not built (SessionGroup grouping and isShowingFileHistory have no store state at all --
     M2-15/M2-14's own documented reductions).

     The route <-> store sync below exists only because SessionDetail's scroll-restore (M2-20
     AC4) needs real browser history entries to restore "on back/forward" at all: selecting a
     session has never touched the URL (every click handler calls the store directly), so
     M2-06's own `/sessions/:projectId/:sessionId?` route -- built, never driven -- would
     otherwise stay permanently unused. The store stays the single place selection actually
     happens (click handlers are untouched); this component only mirrors it into the URL and
     mirrors the URL back on a route change the store didn't cause itself (back/forward, a typed
     URL, a deep link). -->
<script setup lang="ts">
import { watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import SessionsNavigation from './SessionsNavigation.vue';
import ProjectSessions from './ProjectSessions.vue';
import SessionSkeleton from './SessionSkeleton.vue';
import SessionDetail from './SessionDetail.vue';
import { useSessionStore } from './store';

const sessionStore = useSessionStore();
const route = useRoute();
const router = useRouter();

// Store -> route: mirror the current selection into the URL so it becomes a real history entry.
watch(
  () => [sessionStore.selectedProjectId, sessionStore.selectedSession?.id] as const,
  ([projectId, sessionId]) => {
    if (route.name !== 'sessions' && route.name !== 'session-detail') return;
    if (route.params.projectId === projectId && route.params.sessionId === (sessionId ?? undefined)) return;
    if (!projectId) {
      void router.push({ name: 'sessions' });
    } else {
      void router.push({ name: 'session-detail', params: { projectId, sessionId } });
    }
  },
);

// Route -> store: catches everything the store itself didn't cause -- back/forward, a typed URL,
// a deep link -- and is a no-op whenever the store already matches (so the watcher above never
// bounces back into this one).
watch(
  () => [route.params.projectId as string | undefined, route.params.sessionId as string | undefined] as const,
  ([projectId, sessionId]) => {
    if (sessionStore.selectedProjectId === (projectId ?? null) && sessionStore.selectedSession?.id === sessionId) return;
    if (!projectId) {
      sessionStore.deselectSession();
      return;
    }
    if (sessionId) void sessionStore.selectSession(projectId, sessionId);
    else sessionStore.deselectSession();
  },
  { immediate: true },
);
</script>

<template>
  <div class="flex h-full">
    <div class="h-full w-[280px] min-w-[220px] max-w-[380px]">
      <SessionsNavigation />
    </div>
    <main class="flex-1 overflow-hidden">
      <SessionSkeleton v-if="sessionStore.isLoadingSession" />
      <SessionDetail
        v-else-if="sessionStore.selectedSession"
        :key="sessionStore.selectedSession.id"
        :session="sessionStore.selectedSession"
      />
      <ProjectSessions
        v-else-if="sessionStore.currentProject"
        :project="sessionStore.currentProject"
      />
      <div
        v-else
        class="flex h-full items-center justify-center text-(--text-tertiary)"
      >
        <p>Select a session from the list to view its content.</p>
      </div>
    </main>
  </div>
</template>
