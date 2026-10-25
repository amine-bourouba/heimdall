<script setup lang="ts">
// Ported from SessionTodosView.swift (M3-06). Self-contained, same reasoning as
// SessionFacetsCard.vue: fetches its own data from `sessionId`.
//
// `get_session_todos` can return more than one list per session -- confirmed directly in
// session_data.rs's own test (`get_session_todos_keeps_the_main_list_and_the_agent_list_apart`:
// a main list, `agentId: null`, plus a separate sub-agent list under the same session id).
// `SessionTodosView.swift` only ever renders one flat `[SessionTodo]` list, since the Swift
// reference app's own model has no such split to begin with -- there is no Swift precedent for rendering
// more than one, so each non-empty list gets its own card here, in the order Rust returns them
// (main first in every case seen), rather than merging them into one combined list that would
// obscure which todos belong to the sub-agent.
import { computed, ref, watch } from 'vue';
import { ipc } from '@/ipc/client';
import type { TodoList_Serialize as TodoList } from '@/ipc/bindings';
import AppIcon from '@/components/AppIcon.vue';

const props = defineProps<{ sessionId: string }>();

const lists = ref<TodoList[]>([]);
const expanded = ref<Set<number>>(new Set());

// Same staleness guard as SessionFacetsCard.vue's own (and search/store.ts's original): a fetch
// for a since-abandoned session must not overwrite the one for whichever session is open now.
let generation = 0;
async function load(sessionId: string) {
  const thisGeneration = ++generation;
  const result = await ipc.getSessionTodos(sessionId);
  if (thisGeneration !== generation) return;
  lists.value = result.filter((list) => list.items.length > 0);
  expanded.value = new Set(lists.value.map((_, index) => index));
}

watch(() => props.sessionId, load, { immediate: true });

function toggle(index: number) {
  if (expanded.value.has(index)) expanded.value.delete(index);
  else expanded.value.add(index);
}

function counts(list: TodoList) {
  return {
    completed: list.items.filter((i) => i.status === 'completed').length,
    inProgress: list.items.filter((i) => i.status === 'inProgress').length,
    pending: list.items.filter((i) => i.status === 'pending').length,
  };
}

const statusIcon = (status: string) => (status === 'completed' ? 'checkmark.circle.fill' : status === 'inProgress' ? 'circle.dotted' : 'circle');
const statusIconClass = (status: string) => (status === 'completed' ? 'text-(--color-green)' : status === 'inProgress' ? 'text-(--accent)' : 'text-(--text-tertiary)');

const totalCounts = computed(() => lists.value.map((l, i) => ({ list: l, index: i, counts: counts(l) })));
</script>

<template>
  <div
    v-for="{ list, index, counts: c } in totalCounts"
    :key="`${list.sessionId}:${list.agentId ?? ''}`"
    class="overflow-hidden rounded-md border bg-(--bg-card)"
  >
    <button
      type="button"
      class="flex w-full items-center gap-(--spacing-sm) px-(--spacing-lg) py-(--spacing-md) text-left"
      :aria-expanded="expanded.has(index)"
      :title="expanded.has(index) ? 'Collapse todos' : 'Expand todos'"
      @click="toggle(index)"
    >
      <AppIcon
        name="checklist"
        size="sm"
        class="text-(--accent)"
      />
      <span class="font-semibold text-(--text-secondary) text-(length:--text-small)">{{ list.agentId ? 'Todos · Sub-agent' : 'Todos' }}</span>
      <span
        v-if="c.completed > 0"
        class="rounded-xs bg-(--color-green)/10 px-(--spacing-xs) py-px text-(--color-green) text-(length:--text-micro)"
      >{{ c.completed }}</span>
      <span
        v-if="c.inProgress > 0"
        class="rounded-xs bg-(--accent-dim) px-(--spacing-xs) py-px text-(--accent) text-(length:--text-micro)"
      >{{ c.inProgress }}</span>
      <span
        v-if="c.pending > 0"
        class="rounded-xs bg-(--bg-elevated) px-(--spacing-xs) py-px text-(--text-tertiary) text-(length:--text-micro)"
      >{{ c.pending }}</span>
      <div class="flex-1" />
      <span class="rounded-xs bg-(--accent-dim) px-(--spacing-sm) py-(--spacing-xxs) text-(--accent) text-(length:--text-micro)">{{ list.items.length }}</span>
      <AppIcon
        name="chevron.right"
        size="sm"
        class="text-(--text-tertiary) transition-transform"
        :class="{ 'rotate-90': expanded.has(index) }"
      />
    </button>

    <div
      v-if="expanded.has(index)"
      class="flex flex-col border-t"
    >
      <div
        v-for="(item, itemIndex) in list.items"
        :key="itemIndex"
        class="flex items-start gap-(--spacing-md) px-(--spacing-lg) py-(--spacing-sm)"
        :class="{ 'border-t border-(--border)/15': itemIndex > 0 }"
      >
        <AppIcon
          :name="statusIcon(item.status)"
          size="sm"
          :class="statusIconClass(item.status)"
        />
        <div class="flex flex-col gap-(--spacing-xxs)">
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
</template>
