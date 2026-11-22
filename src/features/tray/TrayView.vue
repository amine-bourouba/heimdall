<script setup lang="ts">
// Ported from MenuBarView.swift (M3-58): the tray window's whole content (320x440, `src-tauri/src/tray.rs`) --
// a header with stats, a search field, up to 10 recent sessions that open in the main window, and "Open Heimdall".
//
// This window has three commands and nothing else (`capabilities/tray.json`): `get_tray_snapshot`,
// `open_in_main_window` and `get_prefs`. So no toasts (errors show in the view), no events (it is never told
// that data changed) and no window API (Escape cannot hide it; losing focus already does, in Rust).
//
// The window is hidden and shown, never closed, so this page lives across many openings: whatever it loaded last
// time is stale the next. It re-reads the list and the theme every time it becomes visible.
//
// Deviations from the Swift view: the header shows sessions, messages and messages *today* (what
// `TraySnapshot` carries); Swift's tokens and cost need the analytics command, which this window cannot call.
// There is no app icon, and the title is "Heimdall" rather than the assistant's name.
import { onMounted, onUnmounted, ref, watch } from 'vue';
import { IpcError, ipc } from '@/ipc/client';
import { usePrefsStore } from '@/stores/prefs';
import { useThemeStore } from '@/stores/theme';
import AppIcon from '@/components/AppIcon.vue';
import FilterField from '@/components/FilterField.vue';
import { formatRelativeTime } from '@/features/session/format';
import type { NavTarget, TraySnapshot } from '@/ipc/bindings';

/** The same pause the search overlay waits after a keystroke: each call re-reads every project from disk. */
const TYPING_DEBOUNCE_MS = 200;

const prefsStore = usePrefsStore();
const themeStore = useThemeStore();

const snapshot = ref<TraySnapshot | null>(null);
const query = ref('');
const error = ref<string | null>(null);

/** An `IpcError` names its real cause; anything else gets the plain fallback (as the toasts store does). */
function messageOf(failure: unknown, fallback: string): string {
  return failure instanceof IpcError ? failure.message : fallback;
}

/** Not `formatLargeNumber` from the analytics charts: that module imports ECharts, which must stay out of this bundle. */
function formatCount(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1000) return `${(n / 1000).toFixed(1)}K`;
  return String(n);
}

// A later search supersedes an earlier one still in flight (the search store's own generation counter).
let generation = 0;
async function loadSnapshot() {
  const mine = ++generation;
  const trimmed = query.value.trim();
  try {
    const next = await ipc.getTraySnapshot(trimmed === '' ? null : trimmed);
    if (mine !== generation) return;
    snapshot.value = next;
    error.value = null;
  } catch (failure) {
    if (mine === generation) error.value = messageOf(failure, 'Could not load recent sessions');
  }
}

// A theme that cannot load leaves the default one; the message stays until the next list loads.
async function loadTheme() {
  try {
    await prefsStore.load();
    themeStore.applyToDocument();
  } catch (failure) {
    error.value = messageOf(failure, 'Could not load the theme');
  }
}

let refreshing: Promise<unknown> | null = null;
/** Focus and visibility both fire when the window is shown; one refresh serves both. */
function refresh() {
  refreshing ??= Promise.all([loadTheme(), loadSnapshot()]).finally(() => (refreshing = null));
}
function onShown() {
  if (document.visibilityState === 'visible') refresh();
}

let debounce: ReturnType<typeof setTimeout> | undefined;
watch(query, () => {
  clearTimeout(debounce);
  debounce = setTimeout(() => void loadSnapshot(), TYPING_DEBOUNCE_MS);
});

onMounted(() => {
  refresh();
  window.addEventListener('focus', onShown);
  document.addEventListener('visibilitychange', onShown);
});
onUnmounted(() => {
  clearTimeout(debounce);
  window.removeEventListener('focus', onShown);
  document.removeEventListener('visibilitychange', onShown);
});

async function open(target: NavTarget) {
  try {
    await ipc.openInMainWindow(target);
    error.value = null;
  } catch (failure) {
    error.value = messageOf(failure, 'Could not open the main window');
  }
}
</script>

<template>
  <div class="flex h-screen w-screen flex-col overflow-hidden bg-(--bg-card) text-(--text-primary)">
    <header class="shrink-0 border-b border-(--border)/30 px-(--spacing-lg) py-(--spacing-md)">
      <h1 class="text-(length:--text-caption) font-medium">
        Heimdall
      </h1>
      <div
        v-if="snapshot"
        class="mt-(--spacing-sm) flex gap-(--spacing-md)"
      >
        <div
          title="Total sessions"
          class="flex flex-col"
        >
          <span class="text-(length:--text-caption) font-medium">{{ formatCount(snapshot.totalSessions) }}</span>
          <span class="text-(length:--text-tiny) text-(--text-tertiary)">sessions</span>
        </div>
        <div
          title="Total messages"
          class="flex flex-col"
        >
          <span class="text-(length:--text-caption) font-medium">{{ formatCount(snapshot.totalMessages) }}</span>
          <span class="text-(length:--text-tiny) text-(--text-tertiary)">messages</span>
        </div>
        <div
          title="Messages today"
          class="flex flex-col"
        >
          <span class="text-(length:--text-caption) font-medium">{{ formatCount(snapshot.todayMessages) }}</span>
          <span class="text-(length:--text-tiny) text-(--text-tertiary)">today</span>
        </div>
      </div>
    </header>

    <div class="shrink-0 border-b border-(--border)/30">
      <FilterField
        v-model="query"
        placeholder="Search recent sessions…"
      />
    </div>

    <div class="min-h-0 flex-1 overflow-y-auto p-(--spacing-sm)">
      <p
        v-if="error"
        role="alert"
        class="px-(--spacing-sm) py-(--spacing-xs) text-(length:--text-small) text-(--color-red)"
      >
        {{ error }}
      </p>
      <template v-if="snapshot">
        <ul
          v-if="snapshot.recent.length > 0"
          class="flex flex-col gap-(--spacing-xxs)"
        >
          <li
            v-for="item in snapshot.recent"
            :key="item.session.id"
          >
            <button
              type="button"
              :title="item.session.title"
              class="flex w-full items-center gap-(--spacing-sm) rounded-sm px-(--spacing-sm) py-(--spacing-xs) text-left hover:bg-(--bg-elevated)"
              @click="open({ kind: 'session', projectId: item.projectId, sessionId: item.session.id })"
            >
              <AppIcon
                name="text.bubble"
                size="sm"
                class="w-4.5 shrink-0 text-(--text-tertiary)"
              />
              <span class="min-w-0 flex-1">
                <span class="block truncate text-(length:--text-caption) font-medium">{{ item.session.title }}</span>
                <span class="block truncate text-(length:--text-tiny) text-(--text-tertiary)">{{ item.projectName }}</span>
              </span>
              <span class="shrink-0 text-(length:--text-tiny) text-(--text-tertiary)">{{ formatRelativeTime(item.session.startedAt) }}</span>
            </button>
          </li>
        </ul>
        <div
          v-else
          class="flex flex-col items-center gap-(--spacing-sm) p-(--spacing-xl) text-(--text-tertiary)"
        >
          <AppIcon
            name="text.bubble"
            size="lg"
          />
          <p class="text-(length:--text-caption)">
            {{ query.trim() === '' ? 'No recent sessions' : 'No matching sessions' }}
          </p>
        </div>
      </template>
      <p
        v-else-if="!error"
        class="p-(--spacing-xl) text-center text-(length:--text-caption) text-(--text-tertiary)"
      >
        Loading…
      </p>
    </div>

    <button
      type="button"
      title="Open the main window"
      class="flex shrink-0 items-center justify-between border-t border-(--border)/30 px-(--spacing-lg) py-(--spacing-md) text-(length:--text-caption) font-medium hover:bg-(--bg-elevated)"
      @click="open({ kind: 'screen', screen: 'sessions' })"
    >
      Open Heimdall
      <AppIcon
        name="arrow.up.forward"
        size="sm"
        class="text-(--text-tertiary)"
      />
    </button>
  </div>
</template>
