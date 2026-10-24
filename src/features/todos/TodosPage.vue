<script setup lang="ts">
// Ported from TodosOverviewView.swift (M3-05). Project/session cards are rendered inline, not as
// separate components -- this task's own Outputs list names only TodosPage.vue, the same
// scope-narrowing M3-02/M3-03/M3-04 already established. No grid/list layout toggle either: not
// named in this task's own Goal or AC (unlike M3-03's plans, which explicitly tests layout
// persistence), so one fixed vertical layout is used, matching M3-02's own precedent for the
// same situation.
import { computed, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import type { TodoList_Serialize as TodoList } from '@/ipc/bindings';
import AppIcon from '@/components/AppIcon.vue';
import FilterField from '@/components/FilterField.vue';
import ShimmerBlock from '@/components/ShimmerBlock.vue';
import { useTodosStore } from './store';

const store = useTodosStore();
const router = useRouter();

onMounted(() => void store.load());

const countText = computed(
  () => `${store.totalTodos} ${store.totalTodos === 1 ? 'todo' : 'todos'} · ${store.totalSessions} ${store.totalSessions === 1 ? 'session' : 'sessions'}`,
);

function goToSession(projectId: string | null, sessionId: string) {
  if (!projectId) return;
  void router.push({ name: 'session-detail', params: { projectId, sessionId } });
}
</script>

<template>
  <div class="flex h-full flex-col">
    <div class="flex items-center gap-(--spacing-md) border-b px-(--spacing-xxxl) py-(--spacing-xl)">
      <div class="flex h-9 w-9 items-center justify-center rounded-md bg-(--accent-dim)">
        <AppIcon
          name="checklist"
          class="text-(--accent)"
        />
      </div>
      <div class="flex flex-col gap-(--spacing-xxs)">
        <h1 class="text-(--text-primary) text-(length:--text-heading)">
          TODOs
        </h1>
        <span class="self-start rounded-full bg-(--bg-elevated) px-(--spacing-sm) py-(--spacing-xxs) text-(--text-tertiary) text-(length:--text-tiny)">{{ countText }}</span>
      </div>
      <div class="flex-1" />
      <FilterField
        v-model="store.query"
        placeholder="Find in TODOs…"
        class="w-50"
      />
    </div>

    <div class="flex-1 overflow-y-auto px-(--spacing-xxxl) py-(--spacing-lg)">
      <div
        v-if="store.isLoading"
        class="flex flex-col gap-(--spacing-md)"
      >
        <ShimmerBlock
          v-for="n in 5"
          :key="n"
          rounded="md"
          class="h-24"
        />
      </div>

      <div
        v-else-if="store.groups.length === 0"
        class="flex h-full flex-col items-center justify-center gap-(--spacing-xs) text-(--text-tertiary)"
      >
        <AppIcon
          name="checklist"
          size="lg"
        />
        <p>No TODOs found</p>
        <p class="text-(length:--text-small)">
          TODOs from Claude Code sessions will appear here
        </p>
      </div>

      <div
        v-else-if="store.filteredGroups.length === 0"
        class="flex h-full flex-col items-center justify-center gap-(--spacing-xs) text-(--text-tertiary)"
      >
        <AppIcon
          name="magnifyingglass"
          size="lg"
        />
        <p>No TODOs match "{{ store.query }}"</p>
      </div>

      <div
        v-else
        class="flex flex-col gap-(--spacing-lg)"
      >
        <div
          v-for="group in store.filteredGroups"
          :key="group.projectId ?? 'unknown'"
          class="flex flex-col gap-(--spacing-sm) rounded-md border bg-(--bg-card) p-(--spacing-lg)"
        >
          <div class="flex items-center gap-(--spacing-sm)">
            <p class="flex-1 text-(--text-primary) text-(length:--text-body-medium)">
              {{ group.projectName ?? 'Unknown project' }}
            </p>
            <span
              v-if="store.statusCountsFor(group.lists).completed > 0"
              class="rounded-sm bg-(--color-green)/10 px-(--spacing-xs) py-[1px] text-(--color-green) text-(length:--text-tiny)"
            >{{ store.statusCountsFor(group.lists).completed }}</span>
            <span
              v-if="store.statusCountsFor(group.lists).inProgress > 0"
              class="rounded-sm bg-(--accent-dim) px-(--spacing-xs) py-[1px] text-(--accent) text-(length:--text-tiny)"
            >{{ store.statusCountsFor(group.lists).inProgress }}</span>
            <span
              v-if="store.statusCountsFor(group.lists).pending > 0"
              class="rounded-sm bg-(--bg-elevated) px-(--spacing-xs) py-[1px] text-(--text-tertiary) text-(length:--text-tiny)"
            >{{ store.statusCountsFor(group.lists).pending }}</span>
          </div>

          <div
            v-for="list in (group.lists as TodoList[])"
            :key="`${list.sessionId}:${list.agentId ?? ''}`"
            class="rounded-sm border bg-(--bg-elevated) p-(--spacing-md)"
          >
            <div class="flex items-center gap-(--spacing-sm)">
              <AppIcon
                name="rectangle.stack"
                size="sm"
                class="text-(--accent)"
              />
              <span class="flex-1 truncate text-(--text-tertiary) text-(length:--text-tiny)">{{ list.sessionId }}</span>
              <span
                v-if="list.agentId"
                class="rounded-sm bg-(--bg-card) px-(--spacing-xs) py-[1px] text-(--text-tertiary) text-(length:--text-tiny)"
              >Agent</span>
              <button
                type="button"
                class="text-(--text-tertiary)"
                :disabled="!group.projectId"
                :aria-label="`Go to session ${list.sessionId}`"
                title="Go to session"
                @click="goToSession(group.projectId, list.sessionId)"
              >
                <AppIcon
                  name="arrow.right.circle"
                  size="sm"
                />
              </button>
            </div>

            <div class="mt-(--spacing-xs) flex flex-col gap-(--spacing-xs)">
              <div
                v-for="(item, index) in list.items"
                :key="index"
                class="flex items-start gap-(--spacing-sm)"
              >
                <AppIcon
                  :name="item.status === 'completed' ? 'checkmark.circle.fill' : item.status === 'inProgress' ? 'circle.dotted' : 'circle'"
                  size="sm"
                  :class="item.status === 'completed' ? 'text-(--color-green)' : item.status === 'inProgress' ? 'text-(--accent)' : 'text-(--text-tertiary)'"
                />
                <div class="flex flex-col">
                  <p
                    class="text-(length:--text-caption)"
                    :class="item.status === 'completed' ? 'text-(--text-tertiary) line-through' : 'text-(--text-primary)'"
                  >
                    {{ item.content }}
                  </p>
                  <p
                    v-if="item.status === 'inProgress'"
                    class="text-(--accent) text-(length:--text-tiny)"
                  >
                    {{ item.activeForm }}
                  </p>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
