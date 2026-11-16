<script setup lang="ts">
// Ported from HistoryListView.swift (M3-02): header, search, date-bucketed list, paging. The
// project-filter picker and grid/list toggle are not in M3-02's or M3-51's Outputs/AC -- left for a
// later pass if ever asked for, not an oversight.
//
// M3-51 adds the "Clear old prompts" menu (Swift's `clearMenu`, HistoryListView.swift:557-575: four
// ages, disabled with no entries). The confirmation is Rust's native one (`clear_history_older_than`
// names the number of days); a declined one comes back `cancelled` and nothing happens, nothing is
// said. Rust does not say how many entries went, so the toast does not either (Swift's did).
import { nextTick, onMounted, ref } from 'vue';
import AppIcon from '@/components/AppIcon.vue';
import FilterField from '@/components/FilterField.vue';
import ShimmerBlock from '@/components/ShimmerBlock.vue';
import { useToastsStore } from '@/stores/toasts';
import { HISTORY_GROUP_TITLES, useHistoryStore } from './store';
import HistoryRow from './HistoryRow.vue';

/** HistoryListView.swift:526-531 and `clearOptionLabel` (:386-393). */
const CLEAR_OPTIONS = [
  { days: 14, label: 'Older than 2 weeks', phrase: '2 weeks' },
  { days: 30, label: 'Older than 1 month', phrase: '1 month' },
  { days: 90, label: 'Older than 3 months', phrase: '3 months' },
  { days: 180, label: 'Older than 6 months', phrase: '6 months' },
] as const;

const store = useHistoryStore();
const toastsStore = useToastsStore();

const isMenuOpen = ref(false);
const isClearing = ref(false);
const menuRoot = ref<HTMLElement | null>(null);

function closeMenu() {
  isMenuOpen.value = false;
}

// Close only when focus leaves the whole menu. Closing on any focusout removed the item before the
// click that caused it landed (clicking an item first moves focus off the toggle): AppMenu.vue's fix.
function onFocusOut(event: FocusEvent) {
  if (!menuRoot.value?.contains(event.relatedTarget as Node | null)) closeMenu();
}

async function toggleMenu() {
  isMenuOpen.value = !isMenuOpen.value;
  if (!isMenuOpen.value) return;
  await nextTick();
  menuRoot.value?.querySelector<HTMLElement>('[role="menuitem"]')?.focus();
}

function onMenuKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    // This menu's own Escape: the app's layered Escape must not also act on the page behind it.
    event.stopPropagation();
    closeMenu();
    menuRoot.value?.querySelector<HTMLElement>('[aria-haspopup]')?.focus();
    return;
  }
  if (event.key !== 'ArrowDown' && event.key !== 'ArrowUp') return;
  event.preventDefault();
  event.stopPropagation(); // not the sidebar's arrow navigation
  const items = Array.from(menuRoot.value?.querySelectorAll<HTMLElement>('[role="menuitem"]') ?? []);
  const at = items.indexOf(document.activeElement as HTMLElement);
  const next = event.key === 'ArrowDown' ? (at + 1) % items.length : (at - 1 + items.length) % items.length;
  items[next]?.focus();
}

async function clearOlderThan(option: (typeof CLEAR_OPTIONS)[number]) {
  closeMenu();
  if (isClearing.value) return;
  isClearing.value = true;
  try {
    const outcome = await store.clearOlderThan(option.days);
    if (outcome === 'written') toastsStore.show(`Deleted prompts older than ${option.phrase}`, { style: 'info' });
    else if (outcome === 'unchanged') toastsStore.show(`No prompts older than ${option.phrase}`, { style: 'info' });
  } catch (error) {
    toastsStore.showError(error, 'Could not clear the history');
  } finally {
    isClearing.value = false;
  }
}

onMounted(() => void store.load());
</script>

<template>
  <div class="flex h-full flex-col">
    <div class="flex items-center gap-(--spacing-md) border-b px-(--spacing-xxxl) py-(--spacing-xl)">
      <div class="flex h-9 w-9 items-center justify-center rounded-md bg-(--accent-dim)">
        <AppIcon
          name="clock.arrow.circlepath"
          class="text-(--accent)"
        />
      </div>
      <div class="flex flex-col gap-(--spacing-xxs)">
        <h1 class="text-(--text-primary) text-(length:--text-heading)">
          History
        </h1>
        <span class="self-start rounded-full bg-(--bg-elevated) px-(--spacing-sm) py-(--spacing-xxs) text-(--text-tertiary) text-(length:--text-tiny)">
          {{ store.filteredEntries.length === store.total ? store.total : `${store.filteredEntries.length} of ${store.total}` }}
          {{ store.total === 1 ? 'prompt' : 'prompts' }}
        </span>
      </div>
      <div class="flex-1" />
      <FilterField
        v-model="store.query"
        placeholder="Search prompts…"
        class="w-50"
      />
      <div
        ref="menuRoot"
        class="relative"
        @focusout="onFocusOut"
        @keydown="onMenuKeydown"
      >
        <button
          type="button"
          title="Clear old prompts"
          aria-label="Clear old prompts"
          aria-haspopup="menu"
          :aria-expanded="isMenuOpen"
          :disabled="store.entries.length === 0 || isClearing"
          class="rounded-sm p-(--spacing-xs) text-(--text-tertiary) disabled:opacity-40"
          @click="toggleMenu"
        >
          <AppIcon name="trash" />
        </button>
        <ul
          v-if="isMenuOpen"
          role="menu"
          aria-label="Clear history"
          class="glass absolute right-0 top-full z-10 mt-(--spacing-xs) min-w-[200px] rounded-md p-(--spacing-xs)"
        >
          <li
            v-for="option in CLEAR_OPTIONS"
            :key="option.days"
          >
            <button
              type="button"
              role="menuitem"
              class="w-full rounded-sm px-(--spacing-sm) py-(--spacing-xs) text-left text-(length:--text-caption)"
              @click="clearOlderThan(option)"
            >
              {{ option.label }}
            </button>
          </li>
        </ul>
      </div>
    </div>

    <div class="flex-1 overflow-y-auto px-(--spacing-xxxl) py-(--spacing-lg)">
      <div
        v-if="store.isLoading"
        class="flex flex-col gap-(--spacing-sm)"
      >
        <ShimmerBlock
          v-for="n in 6"
          :key="n"
          rounded="md"
          class="h-20"
        />
      </div>

      <div
        v-else-if="store.entries.length === 0"
        class="flex h-full flex-col items-center justify-center gap-(--spacing-xs) text-(--text-tertiary)"
      >
        <AppIcon
          name="clock.arrow.circlepath"
          size="lg"
        />
        <p>No history found</p>
        <p class="text-(length:--text-small)">
          ~/.claude/history.jsonl
        </p>
      </div>

      <div
        v-else-if="store.filteredEntries.length === 0"
        class="flex h-full flex-col items-center justify-center gap-(--spacing-xs) text-(--text-tertiary)"
      >
        <AppIcon
          name="magnifyingglass"
          size="lg"
        />
        <p>No prompts match your search</p>
      </div>

      <template v-else>
        <div
          v-for="group in store.groupedEntries"
          :key="group.group"
          class="flex flex-col gap-(--spacing-sm)"
        >
          <div class="flex items-center gap-(--spacing-sm) pt-(--spacing-md) pb-(--spacing-xs)">
            <h2 class="text-(--text-tertiary) text-(length:--text-tiny)">
              {{ HISTORY_GROUP_TITLES[group.group].toUpperCase() }}
            </h2>
            <div class="h-px flex-1 bg-(--border)" />
          </div>
          <HistoryRow
            v-for="entry in group.entries"
            :key="entry.id"
            :entry="entry"
            :query="store.query"
          />
        </div>

        <div
          v-if="store.hasMore"
          class="flex justify-center pt-(--spacing-lg)"
        >
          <button
            type="button"
            class="rounded-sm bg-(--bg-elevated) px-(--spacing-md) py-(--spacing-sm) text-(--text-secondary) text-(length:--text-small)"
            :disabled="store.isLoadingMore"
            @click="store.loadMore()"
          >
            {{ store.isLoadingMore ? 'Loading…' : 'Load more' }}
          </button>
        </div>
      </template>
    </div>
  </div>
</template>
