<script setup lang="ts">
// Ported from DebugLogView.swift (M3-08). Mounted only while `sessionStore.showDebugLogSessionId`
// is set (SessionDetail.vue's own `v-if`), so this component reads it directly rather than taking
// a prop -- the same convention `ExportOptionsDialog.vue`'s stub documents for the sibling dialog.
//
// One real, documented departure from a literal port: Swift's own `selectedLevels: Set<Level>` is
// a multi-select filter applied client-side over an already-loaded, unfiltered `entries` array.
// `get_debug_log_page` (confirmed directly in session_data.rs/debug_log.rs) instead takes a single
// `level: Option<LogLevel>` applied server-side -- an exact match, not a min-threshold -- and its
// own `total`/`offset` are counted within that filtered space. A multi-select UI has no faithful
// mapping onto that one-value contract, so the level filter here is single-select ("All" plus the
// three real levels), each choice re-fetching from offset 0 rather than filtering what is already
// loaded. Search stays client-side over whatever has been loaded so far, same as Swift -- there is
// no server-side message search to call instead.
import { computed, onMounted, onUnmounted, ref, watch } from 'vue';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import { ipc, onDataChanged } from '@/ipc/client';
import type { DebugLogEntry, LogLevel } from '@/ipc/bindings';
import AppIcon from '@/components/AppIcon.vue';
import { useSessionStore } from './store';

const sessionStore = useSessionStore();

const PAGE_SIZE = 200;
// debug_log.rs's own `load()` helper uses the same value to mean "every entry in the file".
const ALL_ENTRIES_LIMIT = 4_294_967_295;

const entries = ref<DebugLogEntry[]>([]);
const total = ref(0);
const isLoading = ref(true);
const isLoadingMore = ref(false);
const searchText = ref('');
const activeLevel = ref<LogLevel | null>(null);
const copied = ref(false);

let generation = 0;
async function load(sessionId: string) {
  const thisGeneration = ++generation;
  isLoading.value = true;
  const page = await ipc.getDebugLogPage(sessionId, 0, PAGE_SIZE, activeLevel.value);
  if (thisGeneration !== generation) return;
  entries.value = page.entries;
  total.value = page.total;
  isLoading.value = false;
}

async function loadMore() {
  const sessionId = sessionStore.showDebugLogSessionId;
  if (!sessionId || isLoadingMore.value || entries.value.length >= total.value) return;
  isLoadingMore.value = true;
  const page = await ipc.getDebugLogPage(sessionId, entries.value.length, PAGE_SIZE, activeLevel.value);
  entries.value.push(...page.entries);
  isLoadingMore.value = false;
}

watch(
  () => sessionStore.showDebugLogSessionId,
  (sessionId) => {
    if (sessionId) void load(sessionId);
  },
  { immediate: true },
);

function setLevel(level: LogLevel | null) {
  activeLevel.value = level;
  const sessionId = sessionStore.showDebugLogSessionId;
  if (sessionId) void load(sessionId);
}

function onScroll(event: Event) {
  const el = event.target as HTMLElement;
  if (el.scrollHeight - el.scrollTop - el.clientHeight < 200) void loadMore();
}

const filteredEntries = computed(() => {
  const q = searchText.value.trim().toLowerCase();
  if (!q) return entries.value;
  return entries.value.filter((e) => e.message.toLowerCase().includes(q));
});

// AC: "the list refreshes on data-changed" (same convention as memory/store.ts's own
// onDataChanged(['memory'], ...) -- here scoped to this component's own lifetime, since this is a
// sheet, not a store, and must stop listening once closed.
let unlisten: (() => void) | undefined;
onMounted(async () => {
  unlisten = await onDataChanged(['debugLogs'], () => {
    const sessionId = sessionStore.showDebugLogSessionId;
    if (sessionId) void load(sessionId);
  });
});
onUnmounted(() => unlisten?.());

function close() {
  sessionStore.showDebugLogSessionId = null;
}

function formatTime(iso: string): string {
  const d = new Date(iso);
  return `${d.toTimeString().slice(0, 8)}.${String(d.getMilliseconds()).padStart(3, '0')}`;
}

const LEVEL_CLASSES: Record<LogLevel, string> = {
  debug: 'text-(--text-tertiary) bg-(--bg-elevated)',
  warn: 'text-(--color-orange) bg-(--color-orange)/10',
  error: 'text-(--color-red) bg-(--color-red)/10',
};
const ROW_TINT: Record<LogLevel, string> = {
  debug: '',
  warn: 'bg-(--color-orange)/5',
  error: 'bg-(--color-red)/5',
};

// DebugLogView.swift's own copyFullLog(): always copies every entry, regardless of the active
// level filter or search query.
async function copyLog() {
  const sessionId = sessionStore.showDebugLogSessionId;
  if (!sessionId) return;
  const page = await ipc.getDebugLogPage(sessionId, 0, ALL_ENTRIES_LIMIT, null);
  const text = page.entries.map((e) => `${formatTime(e.timestamp)} [${e.level}] ${e.message}`).join('\n');
  await writeText(text);
  copied.value = true;
  setTimeout(() => (copied.value = false), 4000);
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') close();
}
</script>

<template>
  <Teleport to="body">
    <div
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/40"
      @click="close"
      @keydown="onKeydown"
    >
      <div
        role="dialog"
        aria-modal="true"
        aria-label="Debug Log"
        class="glass flex h-[70vh] w-200 max-w-[90vw] flex-col overflow-hidden rounded-md"
        @click.stop
      >
        <div class="flex shrink-0 items-center gap-(--spacing-md) border-b px-(--spacing-xl) py-(--spacing-lg)">
          <div class="flex h-9 w-9 shrink-0 items-center justify-center rounded-md bg-(--accent)/15">
            <AppIcon
              name="ladybug.fill"
              size="sm"
              class="text-(--accent)"
            />
          </div>
          <div class="flex flex-col">
            <h2 class="text-(length:--text-large) font-semibold">
              Debug Log
            </h2>
            <p class="text-(length:--text-tiny) text-(--text-tertiary)">
              {{ total }} {{ total === 1 ? 'entry' : 'entries' }}
            </p>
          </div>
          <div class="flex-1" />
          <button
            type="button"
            title="Copy Log"
            class="rounded-sm p-(--spacing-xs)"
            @click="copyLog"
          >
            <AppIcon
              :name="copied ? 'checkmark' : 'doc.on.doc'"
              size="sm"
              :class="copied ? 'text-(--color-green)' : 'text-(--text-tertiary)'"
            />
          </button>
          <button
            type="button"
            title="Close"
            class="rounded-sm p-(--spacing-xs)"
            @click="close"
          >
            <AppIcon
              name="xmark"
              size="sm"
              class="text-(--text-tertiary)"
            />
          </button>
        </div>

        <div class="flex shrink-0 items-center gap-(--spacing-sm) border-b bg-(--bg-card)/50 px-(--spacing-lg) py-(--spacing-sm)">
          <div class="flex min-w-0 max-w-75 flex-1 items-center gap-(--spacing-sm) rounded-sm bg-(--bg-elevated) px-(--spacing-sm) py-(--spacing-xs)">
            <AppIcon
              name="magnifyingglass"
              size="sm"
              class="shrink-0 text-(--text-tertiary)"
            />
            <input
              v-model="searchText"
              type="text"
              placeholder="Filter log entries…"
              aria-label="Filter log entries"
              class="min-w-0 flex-1 bg-transparent text-(length:--text-caption) outline-none"
            >
            <button
              v-if="searchText"
              type="button"
              title="Clear search"
              @click="searchText = ''"
            >
              <AppIcon
                name="xmark.circle.fill"
                size="sm"
                class="text-(--text-tertiary)"
              />
            </button>
          </div>

          <div class="flex-1" />

          <div
            role="radiogroup"
            aria-label="Level filter"
            class="flex shrink-0 gap-(--spacing-xs)"
          >
            <button
              v-for="level in [null, 'debug', 'warn', 'error'] as const"
              :key="level ?? 'all'"
              type="button"
              role="radio"
              :aria-checked="activeLevel === level"
              class="rounded-sm border px-(--spacing-sm) py-(--spacing-xxs) text-(length:--text-tiny) font-medium"
              :class="activeLevel === level ? 'border-(--accent)/30 bg-(--accent-dim) text-(--accent)' : 'border-(--border) text-(--text-tertiary)'"
              @click="setLevel(level)"
            >
              {{ level ? level.toUpperCase() : 'All' }}
            </button>
          </div>
        </div>

        <p
          v-if="isLoading"
          class="flex flex-1 items-center justify-center text-(--text-tertiary)"
        >
          Loading…
        </p>

        <div
          v-else-if="entries.length === 0"
          class="flex flex-1 flex-col items-center justify-center gap-(--spacing-sm) pt-20 text-center"
        >
          <AppIcon
            name="doc.text"
            size="xl"
            class="text-(--text-tertiary)/50"
          />
          <p class="font-medium text-(--text-secondary)">
            No debug log found
          </p>
          <p class="text-(length:--text-small) text-(--text-tertiary)">
            This session does not have a debug log file.
          </p>
        </div>

        <div
          v-else-if="filteredEntries.length === 0"
          class="flex flex-1 flex-col items-center justify-center gap-(--spacing-sm) pt-20 text-center"
        >
          <AppIcon
            name="line.3.horizontal.decrease"
            size="xl"
            class="text-(--text-tertiary)/50"
          />
          <p class="font-medium text-(--text-secondary)">
            No matching entries
          </p>
          <p class="text-(length:--text-small) text-(--text-tertiary)">
            Adjust your filters or search query.
          </p>
        </div>

        <div
          v-else
          class="flex-1 overflow-y-auto"
          @scroll="onScroll"
        >
          <div
            v-for="entry in filteredEntries"
            :key="entry.id"
            class="flex items-start gap-(--spacing-sm) px-(--spacing-lg) py-(--spacing-xxs)"
            :class="ROW_TINT[entry.level]"
          >
            <span class="w-22 shrink-0 text-right text-(length:--text-code-small) text-(--text-tertiary)">{{ formatTime(entry.timestamp) }}</span>
            <span
              class="w-11 shrink-0 rounded-xs text-center text-(length:--text-tiny)"
              :class="LEVEL_CLASSES[entry.level]"
            >{{ entry.level.toUpperCase() }}</span>
            <span class="min-w-0 flex-1 whitespace-pre-wrap text-(length:--text-code-small) text-(--text-secondary)">{{ entry.message }}</span>
          </div>

          <p
            v-if="isLoadingMore"
            class="py-(--spacing-md) text-center text-(length:--text-tiny) text-(--text-tertiary)"
          >
            Loading more…
          </p>
        </div>
      </div>
    </div>
  </Teleport>
</template>
