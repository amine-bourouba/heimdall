<script setup lang="ts">
// Ported from SearchOverlayView.swift (docs/spec/search-overlay.md, M3-01). Router/session-store
// access lives here, not in store.ts (see that file's own header comment): `store.activate()`
// returns a plain descriptor, `perform()` below is the one place that turns it into a real
// navigation.
import { computed, nextTick, onUnmounted, ref, watch } from 'vue';
import { useRouter } from 'vue-router';
import type { SearchCategory } from '@/ipc/bindings';
import { useSearchStore, type QuickAction, type SearchActivation } from './store';
import { useSessionStore } from '@/features/session/store';
import { useAppActions } from '@/composables/useAppActions';
import SearchResultRow from './SearchResultRow.vue';
import AppIcon from '@/components/AppIcon.vue';
import ShimmerBlock from '@/components/ShimmerBlock.vue';

// §1's table, display label + the same icon `navigationItems.ts` uses for the 13 categories that
// are also sidebar screens -- the other 3 (debugLogs/facets/fileChanges) have no sidebar
// equivalent, so they get their own already-mapped icon (src/components/iconMap.ts).
const CATEGORY_META: Record<SearchCategory, { label: string; icon: string }> = {
  sessions: { label: 'Sessions', icon: 'rectangle.stack.fill' },
  debugLogs: { label: 'Debug Logs', icon: 'ladybug.fill' },
  facets: { label: 'AI Summaries', icon: 'sparkles' },
  fileChanges: { label: 'File Changes', icon: 'doc.text.magnifyingglass' },
  todos: { label: 'Todos', icon: 'checklist' },
  history: { label: 'History', icon: 'clock.arrow.circlepath' },
  commands: { label: 'Commands', icon: 'apple.terminal.fill' },
  skills: { label: 'Skills', icon: 'bolt.fill' },
  plans: { label: 'Plans', icon: 'list.bullet.clipboard.fill' },
  memory: { label: 'Memory', icon: 'brain.head.profile.fill' },
  mcpServers: { label: 'MCP Servers', icon: 'powerplug.fill' },
  plugins: { label: 'Plugins', icon: 'puzzlepiece.fill' },
  outputStyles: { label: 'Output Styles', icon: 'speaker.wave.3.fill' },
  hooks: { label: 'Hooks', icon: 'arrow.triangle.branch' },
  models: { label: 'Models', icon: 'brain.fill' },
  subAgents: { label: 'Sub-agents', icon: 'person.2.fill' },
};

const store = useSearchStore();
const sessionStore = useSessionStore();
const router = useRouter();
const { on } = useAppActions();

const inputRef = ref<HTMLInputElement>();
const rowRefs = ref<HTMLElement[]>([]);

// §7: flat index of every rendered row, in on-screen order, for scroll-into-view only -- keyboard
// selection math itself stays in the store (`moveSelection`/`activate`), this is purely a DOM
// concern the store has no business knowing about.
function setRowRef(el: unknown, index: number) {
  if (el instanceof HTMLElement) rowRefs.value[index] = el;
}

const unregisterShortcut = on('search', () => store.toggleOpen());
onUnmounted(unregisterShortcut);

watch(
  () => store.isOpen,
  async (open) => {
    rowRefs.value = [];
    if (!open) return;
    await nextTick();
    inputRef.value?.focus();
  },
);

// §6: "ScrollViewReader auto-scrolls the selected row into view on selectedIndex change."
watch(
  () => store.selectedIndex,
  async (index) => {
    await nextTick();
    rowRefs.value[index]?.scrollIntoView({ block: 'nearest' });
  },
);

function perform(activation: SearchActivation) {
  switch (activation.kind) {
    case 'route':
      void router.push(activation.to);
      break;
    case 'session':
      if (activation.debugLog) sessionStore.showDebugLogSessionId = activation.sessionId;
      void router.push({ name: 'session-detail', params: { projectId: activation.projectId, sessionId: activation.sessionId } });
      break;
    case 'none':
      break;
  }
  store.close();
}

function activateSelected() {
  perform(store.activate());
}

/** A click on a row not currently selected still has to go through the exact same resolution
 * the keyboard path uses (§6: "Click on a result... Same as pressing Enter with that row
 * selected") -- selecting it first keeps `store.activate()` the one place that logic lives. */
function activateIndex(index: number) {
  store.selectedIndex = index;
  activateSelected();
}

function activateQuickAction(action: QuickAction) {
  perform(store.activateQuickAction(action));
}

// §6: plain arrow/Enter/Escape only; Tab is swallowed below to keep focus trapped in the input
// (the overlay's own selection model never uses real DOM focus movement, so there is nothing for
// Tab to usefully do here, matching the Swift reference app's AppKit keyDown monitor's own scope).
function onKeydown(event: KeyboardEvent) {
  switch (event.key) {
    case 'ArrowDown':
      store.moveSelection(1);
      event.preventDefault();
      break;
    case 'ArrowUp':
      store.moveSelection(-1);
      event.preventDefault();
      break;
    case 'Enter':
      activateSelected();
      event.preventDefault();
      // Stopped below, with Escape, not here individually -- see this switch's own closing
      // comment for why every case needs it, not just these two.
      break;
    case 'Escape':
      store.close();
      event.preventDefault();
      break;
    case 'Tab':
      event.preventDefault();
      break;
  }
  // A real, previously-undiscovered bug (M3-19): none of the cases above stopped propagation, so
  // every one of these keys also reached `useKeyboardNav.ts`'s own document-level listener
  // (`Sidebar.vue`'s `isSuppressed: () => searchStore.isOpen` exists specifically to prevent
  // that) -- harmless for Arrow/Tab, which never touch `isOpen`, but Enter and Escape both call
  // `store.close()` *synchronously* before this event finishes bubbling, so by the time
  // `useKeyboardNav.ts` checked `isSuppressed()` the overlay already looked closed and its own
  // Enter/Escape handling ran too: pressing Enter on a search result double-navigated (the real
  // target, then immediately overwritten by the sidebar's own Enter-activates-highlighted-item
  // handling). Confirmed directly with a real `router.push` trace showing both navigations firing
  // on the one keystroke. A modal overlay owning its own keyboard input should not leak any of it
  // to the app's global handler regardless of `isOpen` timing, so every handled key stops
  // propagation here, not just the two that happened to expose the race.
  event.stopPropagation();
}

// Refocuses the input on any blur while open -- the simplest trap that matches this overlay's
// own keyboard model (no Tab-cycling between rows, only arrow/Enter against `selectedIndex`).
function onInputBlur() {
  if (store.isOpen) void nextTick(() => inputRef.value?.focus());
}

const emptyRowCount = computed(() => store.emptyScreens.length + store.recentSessions.length);
const showShimmer = computed(() => store.isSearching && store.flatResults.length === 0 && !store.isEmptyQuery);

// §7: "group 0's results occupy indices [0, count0), group 1's occupy [count0, count0+count1)" --
// precomputed once per render instead of an `.indexOf()` scan per row in the template.
const groupOffsets = computed<number[]>(() => {
  const offsets: number[] = [];
  let running = 0;
  for (const group of store.groups) {
    offsets.push(running);
    running += group.hits.length;
  }
  return offsets;
});
</script>

<template>
  <Teleport to="body">
    <div
      v-if="store.isOpen"
      class="fixed inset-0 z-50 flex items-start justify-center bg-black/40 pt-[12vh]"
      @click="store.close()"
      @keydown="onKeydown"
    >
      <div
        ref="panelRef"
        role="dialog"
        aria-modal="true"
        aria-label="Search"
        class="glass flex max-h-[70vh] w-150 flex-col overflow-hidden rounded-md"
        @click.stop
      >
        <div class="flex items-center gap-(--spacing-sm) border-b px-(--spacing-md) py-(--spacing-sm)">
          <AppIcon
            name="magnifyingglass"
            class="shrink-0 text-(--text-tertiary)"
          />
          <input
            ref="inputRef"
            v-model="store.query"
            type="text"
            placeholder="Search everything…"
            aria-label="Search"
            class="min-w-0 flex-1 bg-transparent text-(--text-primary) outline-none"
            @blur="onInputBlur"
          >
          <div
            role="radiogroup"
            aria-label="Match mode"
            class="flex shrink-0 gap-(--spacing-xxs) text-(length:--text-small)"
          >
            <button
              v-for="m in (['fuzzy', 'exact'] as const)"
              :key="m"
              type="button"
              role="radio"
              :aria-checked="store.mode === m"
              class="rounded-sm px-(--spacing-xs)"
              :class="store.mode === m ? 'bg-(--accent-dim) text-(--accent)' : 'text-(--text-tertiary)'"
              @click="store.setMode(m)"
            >
              {{ m === 'fuzzy' ? 'Fuzzy' : 'Exact' }}
            </button>
          </div>
        </div>

        <div
          role="listbox"
          aria-label="Search results"
          class="flex-1 overflow-y-auto p-(--spacing-sm)"
        >
          <div
            v-if="showShimmer"
            class="flex flex-col gap-(--spacing-xs)"
          >
            <ShimmerBlock
              v-for="n in 5"
              :key="n"
              rounded="sm"
              class="h-9"
            />
          </div>

          <template v-else-if="store.isEmptyQuery">
            <p
              v-if="emptyRowCount === 0"
              class="p-(--spacing-md) text-(--text-tertiary)"
            >
              Nothing to search yet.
            </p>
            <template v-else>
              <p
                v-if="store.emptyScreens.length > 0"
                class="px-(--spacing-sm) pt-(--spacing-xs) pb-(--spacing-xxs) text-(--text-tertiary) text-(length:--text-tiny)"
              >
                SCREENS
              </p>
              <div
                v-for="(item, index) in store.emptyScreens"
                :key="item.id"
                :ref="(el) => setRowRef(el, index)"
                @click="activateIndex(index)"
              >
                <SearchResultRow
                  :icon="item.icon"
                  :title="item.label"
                  :trailing-count="store.sidebarCounts?.[item.id as keyof typeof store.sidebarCounts] ?? null"
                  :selected="store.selectedIndex === index"
                />
              </div>

              <p
                v-if="store.recentSessions.length > 0"
                class="px-(--spacing-sm) pt-(--spacing-sm) pb-(--spacing-xxs) text-(--text-tertiary) text-(length:--text-tiny)"
              >
                RECENT
              </p>
              <div
                v-for="(recent, i) in store.recentSessions"
                :key="recent.session.id"
                :ref="(el) => setRowRef(el, store.emptyScreens.length + i)"
                @click="activateIndex(store.emptyScreens.length + i)"
              >
                <SearchResultRow
                  icon="rectangle.stack.fill"
                  :title="recent.session.title"
                  :subtitle="recent.projectName"
                  :selected="store.selectedIndex === store.emptyScreens.length + i"
                />
              </div>
            </template>
          </template>

          <template v-else>
            <p
              v-if="store.groups.length === 0 && !store.isSearching"
              class="p-(--spacing-md) text-(--text-tertiary)"
            >
              No results.
            </p>
            <template
              v-for="(group, groupIndex) in store.groups"
              :key="group.category"
            >
              <p class="px-(--spacing-sm) pt-(--spacing-sm) pb-(--spacing-xxs) text-(--text-tertiary) text-(length:--text-tiny)">
                {{ CATEGORY_META[group.category].label.toUpperCase() }}
              </p>
              <div
                v-for="(hit, hitIndex) in group.hits"
                :key="hitIndex"
                :ref="(el) => setRowRef(el, groupOffsets[groupIndex] + hitIndex)"
                @click="activateIndex(groupOffsets[groupIndex] + hitIndex)"
              >
                <SearchResultRow
                  :icon="CATEGORY_META[group.category].icon"
                  :title="hit.title"
                  :subtitle="hit.subtitle"
                  :highlights="hit.highlights"
                  :selected="store.selectedIndex === groupOffsets[groupIndex] + hitIndex"
                />
              </div>
              <template v-if="group.category === 'commands' || group.category === 'mcpServers'">
                <div
                  v-for="quick in store.quickActions.filter((q) => q.group === group.category)"
                  :key="quick.label"
                  @click="activateQuickAction(quick)"
                >
                  <SearchResultRow
                    :icon="quick.icon"
                    :title="quick.label"
                    :subtitle="quick.subtitle"
                  />
                </div>
              </template>
            </template>
          </template>
        </div>
      </div>
    </div>
  </Teleport>
</template>
