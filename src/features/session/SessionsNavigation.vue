<script setup lang="ts">
// Ported from SessionsNavigationView.swift's `sessionsListPane` (lines 68-264). The detail pane
// (lines 266-342 onward) is a later task's job (M2-20+); `SessionsPage.vue` keeps a placeholder
// there for now. `listSearchQuery` is this view's own further local filter on top of the store's
// `filteredSortedProjects` (itself already filtered by `sidebarSearchQuery`) -- two layers,
// exactly matching the real Swift source (`filteredProjects` filters `appState.filteredSortedProjects`
// again), not a simplification introduced here.
import { computed, onMounted, ref } from 'vue';
import AppIcon from '@/components/AppIcon.vue';
import { fuzzyMatch } from '@/lib/fuzzyMatch';
import { usePrefsStore } from '@/stores/prefs';
import ProjectRow from './ProjectRow.vue';
import { useSessionStore, type ProjectSortOption } from './store';

const SORT_OPTIONS: { value: ProjectSortOption; label: string }[] = [
  { value: 'recentActivity', label: 'Recent Activity' },
  { value: 'name', label: 'Name' },
  { value: 'sessionCount', label: 'Session Count' },
];

const sessionStore = useSessionStore();
const prefsStore = usePrefsStore();

const listSearchQuery = ref('');
const collapsedProjects = ref(new Set<string>());
const isSortMenuOpen = ref(false);

const filteredProjects = computed(() => {
  const trimmed = listSearchQuery.value.trim();
  if (trimmed === '') return sessionStore.filteredSortedProjects;

  return sessionStore.filteredSortedProjects.flatMap((project) => {
    if (fuzzyMatch(project.name, trimmed)) return [project];
    const matching = project.sessions.filter((s) => fuzzyMatch(s.title, trimmed) ?? fuzzyMatch(s.id, trimmed));
    return matching.length > 0 ? [{ ...project, sessions: matching }] : [];
  });
});

const totalVisibleSessions = computed(() => filteredProjects.value.reduce((sum, p) => sum + p.sessions.length, 0));

function toggleCollapsed(projectId: string) {
  const next = new Set(collapsedProjects.value);
  if (next.has(projectId)) next.delete(projectId);
  else next.add(projectId);
  collapsedProjects.value = next;
}

async function toggleAgentSessions() {
  await prefsStore.update({ showAgentSessions: !prefsStore.prefs?.showAgentSessions });
}

function selectSort(option: ProjectSortOption) {
  sessionStore.projectSortOption = option;
  isSortMenuOpen.value = false;
}

onMounted(() => void sessionStore.load());
</script>

<template>
  <nav
    aria-label="Sessions"
    class="flex h-full flex-col bg-(--bg-sidebar)"
  >
    <div class="flex items-center gap-(--spacing-xs) px-(--spacing-md) py-(--spacing-sm)">
      <AppIcon
        name="rectangle.stack.fill"
        size="sm"
        class="text-(--text-primary)"
      />
      <span class="text-(length:--text-caption-medium) text-(--text-primary)">All Sessions</span>
      <span class="rounded-full bg-(--bg-card) px-(--spacing-xs) text-(length:--text-tiny) text-(--text-tertiary)">{{ totalVisibleSessions }}</span>

      <div class="flex-1" />

      <button
        type="button"
        :aria-pressed="prefsStore.prefs?.showAgentSessions ?? false"
        :title="prefsStore.prefs?.showAgentSessions ? 'Hide agent sessions' : 'Show agent sessions'"
        @click="toggleAgentSessions"
      >
        <AppIcon
          :name="prefsStore.prefs?.showAgentSessions ? 'person.2.wave.2' : 'person.2'"
          size="sm"
          :class="prefsStore.prefs?.showAgentSessions ? 'text-(--accent)' : 'text-(--text-tertiary)'"
        />
      </button>

      <AppIcon
        v-if="sessionStore.isLoadingMoreProjects"
        name="arrow.clockwise"
        size="sm"
        class="text-(--text-tertiary)"
      />
      <button
        v-else
        type="button"
        title="Refresh sessions"
        @click="sessionStore.load()"
      >
        <AppIcon
          name="arrow.clockwise"
          size="sm"
          class="text-(--text-tertiary)"
        />
      </button>

      <div
        class="relative"
        @focusout="isSortMenuOpen = false"
      >
        <button
          type="button"
          title="Sort projects"
          aria-haspopup="true"
          :aria-expanded="isSortMenuOpen"
          @click="isSortMenuOpen = !isSortMenuOpen"
        >
          <AppIcon
            name="arrow.up.arrow.down"
            size="sm"
            class="text-(--text-tertiary)"
          />
        </button>
        <ul
          v-if="isSortMenuOpen"
          role="menu"
          class="glass absolute right-0 top-full z-10 mt-(--spacing-xs) min-w-[160px] rounded-md p-(--spacing-xs)"
        >
          <li
            v-for="option in SORT_OPTIONS"
            :key="option.value"
          >
            <button
              type="button"
              role="menuitemradio"
              :aria-checked="sessionStore.projectSortOption === option.value"
              class="flex w-full items-center gap-(--spacing-xs) rounded-sm px-(--spacing-sm) py-(--spacing-xs) text-left text-(length:--text-caption)"
              @click="selectSort(option.value)"
            >
              <AppIcon
                v-if="sessionStore.projectSortOption === option.value"
                name="checkmark"
                size="sm"
              />
              {{ option.label }}
            </button>
          </li>
        </ul>
      </div>
    </div>

    <div class="flex items-center gap-(--spacing-sm) px-(--spacing-md) pb-(--spacing-sm)">
      <div class="flex flex-1 items-center gap-(--spacing-sm) rounded-sm bg-(--bg-card) px-(--spacing-sm) py-(--spacing-xs)">
        <AppIcon
          name="magnifyingglass"
          size="sm"
          class="text-(--text-tertiary)"
        />
        <input
          v-model="listSearchQuery"
          type="text"
          placeholder="Search sessions…"
          aria-label="Search sessions"
          class="flex-1 bg-transparent text-(length:--text-caption) text-(--text-primary) outline-none"
        >
        <button
          v-if="listSearchQuery"
          type="button"
          aria-label="Clear search"
          @click="listSearchQuery = ''"
        >
          <AppIcon
            name="xmark.circle.fill"
            size="sm"
            class="text-(--text-tertiary)"
          />
        </button>
      </div>
    </div>

    <div class="min-h-0 flex-1 overflow-y-auto">
      <div
        v-if="sessionStore.isLoadingProjects"
        class="flex h-full flex-col items-center justify-center gap-(--spacing-sm) text-(--text-tertiary)"
      >
        <p class="text-(length:--text-caption)">
          Loading sessions…
        </p>
      </div>
      <div
        v-else-if="filteredProjects.length === 0"
        class="flex h-full flex-col items-center justify-center gap-(--spacing-sm) text-(--text-tertiary)"
      >
        <AppIcon
          name="magnifyingglass"
          size="lg"
        />
        <p class="text-(length:--text-caption)">
          No sessions found
        </p>
      </div>
      <ul v-else>
        <li
          v-for="project in filteredProjects"
          :key="project.id"
        >
          <ProjectRow
            :project="project"
            :is-collapsed="collapsedProjects.has(project.id)"
            @toggle-collapse="toggleCollapsed(project.id)"
          />
        </li>
      </ul>
    </div>
  </nav>
</template>
