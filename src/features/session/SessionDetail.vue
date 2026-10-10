<script lang="ts">
// A genuine module-level singleton, deliberately in a plain (non-setup) `<script>` block: every
// top-level binding inside `<script setup>` actually lives inside the compiled `setup()` function
// and so is re-created per component *instance*, not per module -- confirmed the hard way, see
// the scroll-restoration comment below. This Map needs real module scope to survive the `:key`-
// forced remount between two different sessions, and this is the one place Vue SFCs let a plain
// `<script>` block share a binding with the same file's own `<script setup>` block.
export const scrollPositions = new Map<string, number>();
</script>

<script setup lang="ts">
// Ported from SessionDetailView.swift's `sessionHeader`/`filteredMessages`/`toolFilterBar`/
// `messagesList`/`BubbleRow` (docs/spec/session-views.md §5). Two real, documented departures
// from a literal port:
//  - Virtualization: Swift's `LazyVStack` lets SwiftUI measure every row's intrinsic size for
//    free; the web has no such thing, so TanStack Virtual's own `measureElement` -- bound as each
//    row's element `ref` -- does the job instead. It wraps a `ResizeObserver` per row, so a
//    block's own expand/collapse *and* a font-scale change (applied as a CSS var on `<html>`,
//    which resizes every row's text) both re-measure automatically, with no bespoke "recompute"
//    code needed for either (this task's AC1, in full).
//  - Pagination: Swift's own trigger is an `.onAppear` sentinel row at the bottom of a fixed
//    `visibleCount` window (+20 per trigger, session-views.md §1/§5). There is no `visibleCount`
//    concept here at all -- the virtualizer already renders only what's on screen regardless of
//    how many messages are loaded, so "near the end" is just "the last virtual item's index is
//    close to the end of what IS loaded," checked directly against `session.detail.messageCount`
//    to decide whether another real `get_session_messages` page exists to fetch.
// Scroll restoration (AC4) has no Swift equivalent at all (confirmed directly: SessionDetailView
// never persists an offset) -- a genuine addition this port makes, not a port of existing
// behaviour, kept in the real module-level `scrollPositions` Map above (not `sessionCache`-style:
// that one's singleton-ness comes from Pinia calling `defineStore`'s setup function once per
// store id, a different mechanism from plain module scope).
import { computed, nextTick, onMounted, onUnmounted, ref } from 'vue';
import { useVirtualizer } from '@tanstack/vue-virtual';
import AppIcon from '@/components/AppIcon.vue';
import TextBlock from './blocks/TextBlock.vue';
import ThinkingBlock from './blocks/ThinkingBlock.vue';
import ToolBlock from './blocks/ToolBlock.vue';
import ConversationContextCard from './ConversationContextCard.vue';
import SessionToolbar from './SessionToolbar.vue';
import DebugLogSheet from './DebugLogSheet.vue';
import ExportOptionsDialog from './ExportOptionsDialog.vue';
import { useSessionStore, type LoadedSession } from './store';
import { formatModel, formatRelativeTime, formatTokenCount, totalTokens } from './format';
import { useAppActions } from '@/composables/useAppActions';
import type { ContentBlock, Message, ToolResult } from '@/ipc/bindings';

const props = defineProps<{ session: LoadedSession }>();

const sessionStore = useSessionStore();
const scrollContainer = ref<HTMLElement | null>(null);

// session-views.md §5 "Filtering": text search and tool filter are independent, both applied to
// `session.messages`. `_index` is the message's real position (used for the turn number and for
// "keep the message immediately before a tool match too"), kept stable across filtering.
const indexedMessages = computed(() => props.session.messages.map((message, _index) => ({ message, _index })));

function messageTextContent(message: Message): string {
  return message.blocks
    .filter((b): b is ContentBlock & { kind: 'text' } => b.kind === 'text')
    .map((b) => b.text)
    .join(' ');
}

const searchQuery = computed(() => sessionStore.sessionSearchQuery.trim().toLowerCase());

const filteredMessages = computed(() => {
  const all = indexedMessages.value;
  const textOk = (m: Message) => searchQuery.value === '' || messageTextContent(m).toLowerCase().includes(searchQuery.value);

  if (sessionStore.activeToolFilters.size === 0) {
    return all.filter((m) => textOk(m.message));
  }

  const toolOk = (m: Message) => m.blocks.some((b) => b.kind === 'toolUse' && sessionStore.activeToolFilters.has(b.name));
  const keep = new Set<number>();
  all.forEach(({ message, _index }) => {
    if (toolOk(message)) {
      keep.add(_index);
      // session-views.md §5: "plus the immediately preceding *user* message (for context)".
      if (_index > 0 && all[_index - 1].message.role === 'user') keep.add(_index - 1);
    }
  });
  return all.filter((m) => keep.has(m._index) && textOk(m.message));
});

// session-views.md §5: "A [toolUseId: ToolResult] lookup built once from all session.messages
// (not just filtered/visible ones) so tool results still resolve under a filter."
const toolResultsByUseId = computed(() => {
  const map = new Map<string, ToolResult>();
  for (const message of props.session.messages) {
    for (const block of message.blocks) {
      if (block.kind === 'toolResult') map.set(block.toolUseId, block);
    }
  }
  return map;
});

function visibleBlocks(message: Message): ContentBlock[] {
  return message.blocks.filter((b) => {
    if (b.kind === 'toolResult') return false; // rendered alongside its toolUse, not on its own
    if (b.kind === 'toolUse' && sessionStore.activeToolFilters.size > 0) return sessionStore.activeToolFilters.has(b.name);
    return true;
  });
}

const rowVirtualizer = useVirtualizer(
  computed(() => ({
    count: filteredMessages.value.length,
    getScrollElement: () => scrollContainer.value,
    estimateSize: () => 160,
    overscan: 6,
    getItemKey: (index: number) => filteredMessages.value[index]?.message.id ?? index,
  })),
);

let loadingMore = false;
async function maybeLoadMore() {
  if (loadingMore) return;
  if (props.session.messages.length >= props.session.detail.messageCount) return;
  const items = rowVirtualizer.value.getVirtualItems();
  const last = items[items.length - 1];
  if (!last || last.index < filteredMessages.value.length - 3) return;
  loadingMore = true;
  try {
    await sessionStore.loadMoreMessages();
  } finally {
    loadingMore = false;
  }
}

function onScroll() {
  void maybeLoadMore();
}

const { on } = useAppActions();
let offToolFilter: (() => void) | undefined;

function toggleToolNameFilter(name: string) {
  if (sessionStore.activeToolFilters.has(name)) sessionStore.activeToolFilters.delete(name);
  else sessionStore.activeToolFilters.add(name);
}

function clearToolFilters() {
  sessionStore.activeToolFilters.clear();
}

onMounted(async () => {
  offToolFilter = on('toolFilter', () => {
    sessionStore.isToolFilterActive = !sessionStore.isToolFilterActive;
    if (!sessionStore.isToolFilterActive) sessionStore.activeToolFilters.clear();
  });

  await nextTick();
  const saved = scrollPositions.get(props.session.id);
  // Goes through the DOM directly (not `rowVirtualizer.value.scrollToOffset`): the virtualizer's
  // own scroll-element wiring is itself a reactive effect that resolves one tick after this
  // callback starts, so calling its method here raced ahead of it and silently no-opped
  // (confirmed empirically -- the first version of this restore never actually moved the scroll
  // position). Setting `scrollTop` and firing the same native `scroll` event a real scroll would
  // dispatch reaches the virtualizer through the one path it is definitely already listening on
  // (`maybeLoadMore`'s own trigger below, and the save side on unmount, use the identical path).
  if (saved != null && scrollContainer.value) {
    scrollContainer.value.scrollTop = saved;
    scrollContainer.value.dispatchEvent(new Event('scroll'));
  }
  void maybeLoadMore();
});

onUnmounted(() => {
  offToolFilter?.();
  scrollPositions.set(props.session.id, rowVirtualizer.value.scrollOffset ?? 0);
});

</script>

<template>
  <div class="flex h-full flex-col">
    <div class="shrink-0 border-b border-(--border)/30 px-(--spacing-xxxl) py-(--spacing-lg)">
      <div class="flex items-center gap-(--spacing-sm)">
        <div class="flex h-9 w-9 shrink-0 items-center justify-center rounded-md bg-(--accent)/10">
          <AppIcon
            name="rectangle.stack"
            size="sm"
            class="text-(--accent)"
          />
        </div>
        <h1 class="line-clamp-2 text-(length:--text-large) font-semibold">
          {{ session.detail.summary.title }}
        </h1>
        <span
          v-if="session.detail.summary.model"
          class="shrink-0 rounded-full bg-(--bg-elevated) px-(--spacing-sm) py-px text-(length:--text-tiny)"
        >{{ formatModel(session.detail.summary.model) }}</span>
        <span
          v-if="totalTokens(session.detail.summary.tokens) > 0"
          class="shrink-0 rounded-full bg-(--bg-elevated) px-(--spacing-sm) py-px text-(length:--text-tiny)"
        >{{ formatTokenCount(totalTokens(session.detail.summary.tokens)) }} tokens</span>
      </div>
      <p class="mt-(--spacing-xs) text-(length:--text-small) text-(--text-tertiary)">
        {{ formatRelativeTime(session.detail.summary.startedAt) }} · {{ session.detail.summary.turnCount }} turn(s)
      </p>
      <SessionToolbar
        :summary="session.detail.summary"
        class="mt-(--spacing-md)"
      />
      <ConversationContextCard
        v-if="session.detail.context && session.detail.context.series.length > 0"
        :context="session.detail.context"
        class="mt-(--spacing-md)"
      />
    </div>

    <DebugLogSheet v-if="sessionStore.showDebugLogSessionId" />
    <ExportOptionsDialog v-if="sessionStore.isShowingExportOptions" />

    <div
      v-if="sessionStore.isToolFilterActive"
      class="shrink-0 overflow-x-auto border-b border-(--border)/30 px-(--spacing-lg) py-(--spacing-sm)"
    >
      <div class="flex items-center gap-(--spacing-xs)">
        <button
          v-for="name in session.detail.toolNames"
          :key="name"
          type="button"
          class="shrink-0 rounded-full border px-(--spacing-sm) py-px text-(length:--text-tiny)"
          :class="sessionStore.activeToolFilters.has(name) ? 'border-(--accent) bg-(--accent)/10 text-(--accent)' : 'border-(--border) text-(--text-secondary)'"
          @click="toggleToolNameFilter(name)"
        >
          {{ name }}
        </button>
        <button
          v-if="sessionStore.activeToolFilters.size > 0"
          type="button"
          class="shrink-0 text-(length:--text-tiny) text-(--text-tertiary)"
          @click="clearToolFilters"
        >
          Clear
        </button>
      </div>
    </div>

    <div
      ref="scrollContainer"
      class="flex-1 overflow-y-auto px-(--spacing-xxxl) py-(--spacing-lg)"
      @scroll="onScroll"
    >
      <p
        v-if="filteredMessages.length === 0"
        class="flex h-full items-center justify-center text-(--text-tertiary)"
      >
        No messages to display
      </p>
      <div
        v-else
        :style="{ position: 'relative', height: `${rowVirtualizer.getTotalSize()}px` }"
      >
        <div
          v-for="item in rowVirtualizer.getVirtualItems()"
          :key="String(item.key)"
          :ref="(el) => rowVirtualizer.measureElement(el as Element)"
          :data-index="item.index"
          class="absolute left-0 w-full pb-(--spacing-lg)"
          :style="{ transform: `translateY(${item.start}px)` }"
        >
          <div
            v-if="filteredMessages[item.index].message.role !== 'system'"
            class="flex"
            :class="filteredMessages[item.index].message.role === 'user' ? 'justify-end' : 'justify-start'"
          >
            <div class="flex w-3/4 gap-(--spacing-sm)">
              <div
                v-if="filteredMessages[item.index].message.role === 'assistant'"
                class="flex h-7 w-7 shrink-0 items-center justify-center rounded-sm bg-(--accent)/10"
              >
                <AppIcon
                  name="sparkles"
                  size="sm"
                  class="text-(--accent)"
                />
              </div>
              <div
                class="min-w-0 flex-1 rounded-md border border-(--border) p-(--spacing-lg)"
                :class="filteredMessages[item.index].message.role === 'user' ? 'bg-(--bg-elevated)' : 'bg-(--bg-card)'"
              >
                <div class="mb-(--spacing-sm) flex items-baseline gap-(--spacing-xs) text-(length:--text-tiny) text-(--text-tertiary)">
                  <span class="font-semibold">{{ filteredMessages[item.index].message.role === 'user' ? 'You' : 'Claude' }}</span>
                  <span v-if="filteredMessages[item.index].message.timestamp">{{ formatRelativeTime(filteredMessages[item.index].message.timestamp!) }}</span>
                  <span>#{{ filteredMessages[item.index]._index + 1 }}</span>
                </div>

                <div class="flex flex-col gap-(--spacing-sm)">
                  <template
                    v-for="block in visibleBlocks(filteredMessages[item.index].message)"
                    :key="block.id"
                  >
                    <div v-if="block.kind === 'text'">
                      <div
                        v-if="searchQuery"
                        class="whitespace-pre-wrap text-(length:--text-code) text-(--text-secondary)"
                      >
                        <template
                          v-for="(segment, i) in block.text.split(new RegExp(`(${searchQuery.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')})`, 'i'))"
                          :key="i"
                        >
                          <mark
                            v-if="segment.toLowerCase() === searchQuery"
                            class="rounded-xs bg-(--accent)/30"
                          >{{ segment }}</mark>
                          <template v-else>
                            {{ segment }}
                          </template>
                        </template>
                      </div>
                      <TextBlock
                        v-else
                        :text="block.text"
                        :system="block.system"
                      />
                    </div>
                    <ThinkingBlock
                      v-else-if="block.kind === 'thinking'"
                      :text="block.text"
                    />
                    <ToolBlock
                      v-else-if="block.kind === 'toolUse'"
                      :tool="block"
                      :result="toolResultsByUseId.get(block.toolUseId)"
                    />
                  </template>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
