<script setup lang="ts">
// Ported from SessionsNavigationView.swift's `SessionsProjectSection`/`SessionsListRow`
// (lines 347-444, 629-774). The agent-session *grouping* (`SessionGroup`, `SessionGroupRow`,
// `AgentSessionRow` -- indented child rows under a collapsible parent) is deliberately not
// ported: it is a separate model (`Models/SessionGroup.swift`) outside this task's own Inputs
// list, and this task's AC only asks the spec to cover sorting/search/the agent toggle/keyboard
// selection, none of which need the grouped layout specifically. With `showAgentSessions` on,
// every session (including sidechains) renders as a flat row instead, still carrying the same
// sidechain icon `SessionsListRow` itself always showed regardless of grouping.
import AppIcon from '@/components/AppIcon.vue';
import { formatModel, formatRelativeTime, formatTokenCount, totalTokens } from './format';
import { useSessionStore } from './store';
import type { ProjectWithSessions } from './store';

const props = defineProps<{
  project: ProjectWithSessions;
  isCollapsed: boolean;
}>();

defineEmits<{ 'toggle-collapse': [] }>();

const sessionStore = useSessionStore();

function selectProject() {
  sessionStore.selectedSession = null;
  sessionStore.selectedProjectId = props.project.id;
}

function selectSession(sessionId: string) {
  if (sessionStore.selectedSession?.id === sessionId) return;
  void sessionStore.selectSession(props.project.id, sessionId);
}
</script>

<template>
  <div>
    <div class="flex items-center gap-(--spacing-xs) px-(--spacing-md) py-(--spacing-sm)">
      <button
        type="button"
        :aria-label="isCollapsed ? 'Expand project' : 'Collapse project'"
        :aria-expanded="!isCollapsed"
        @click="$emit('toggle-collapse')"
      >
        <AppIcon
          name="chevron.right"
          size="sm"
          class="text-(--text-tertiary) transition-transform"
          :class="{ 'rotate-90': !isCollapsed }"
        />
      </button>
      <button
        type="button"
        class="flex flex-1 items-center gap-(--spacing-xs) text-left"
        :title="project.name"
        @click="selectProject"
      >
        <AppIcon
          name="shippingbox"
          size="sm"
          class="text-(--text-tertiary)"
        />
        <span class="flex-1 truncate text-(length:--text-subheading) text-(--text-primary)">{{ project.name }}</span>
        <span class="rounded-full bg-(--bg-card) px-(--spacing-xs) text-(length:--text-tiny) text-(--text-tertiary)">{{ project.sessions.length }}</span>
      </button>
    </div>

    <ul v-if="!isCollapsed">
      <li
        v-for="session in project.sessions"
        :key="session.id"
      >
        <button
          type="button"
          class="flex w-full flex-col gap-(--spacing-xxs) py-(--spacing-sm) pr-(--spacing-md) pl-(--spacing-lg) text-left"
          :class="sessionStore.selectedSession?.id === session.id ? 'bg-(--accent-dim)' : ''"
          :title="session.title"
          @click="selectSession(session.id)"
        >
          <div class="flex items-center gap-(--spacing-xs)">
            <AppIcon
              v-if="session.isSidechain"
              name="cpu"
              size="sm"
              class="text-(--color-purple)"
            />
            <span
              class="line-clamp-2 flex-1 text-(length:--text-body-medium)"
              :class="sessionStore.selectedSession?.id === session.id ? 'text-(--accent)' : 'text-(--text-primary)'"
            >{{ session.title }}</span>
            <span class="shrink-0 text-(length:--text-tiny) text-(--text-tertiary)">{{ formatRelativeTime(session.startedAt) }}</span>
          </div>

          <div class="flex items-center gap-(--spacing-xs)">
            <span
              v-if="session.model"
              class="rounded-full bg-(--accent-dim) px-(--spacing-xs) text-(length:--text-tiny) text-(--accent)"
            >{{ formatModel(session.model) }}</span>
            <span
              v-if="totalTokens(session.tokens) > 0"
              class="rounded-full bg-(--color-blue)/15 px-(--spacing-xs) text-(length:--text-tiny) text-(--color-blue)"
            >{{ formatTokenCount(totalTokens(session.tokens)) }} tk</span>
            <span class="text-(length:--text-tiny) text-(--text-tertiary)">{{ session.turnCount }} {{ session.turnCount === 1 ? 'turn' : 'turns' }}</span>
          </div>

          <p
            v-if="session.preview"
            class="line-clamp-2 text-(length:--text-tiny) text-(--text-tertiary)"
          >
            {{ session.preview }}
          </p>
        </button>
      </li>
    </ul>
  </div>
</template>
