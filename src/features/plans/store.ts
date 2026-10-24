// Ported from PlansListView.swift/PlanDetailView.swift (M3-03). Plans go through the generic
// config-item commands (M1-44), not a dedicated Plan type/command on the IPC side: `listConfigItems('plan', null)`
// (plans are "global only", no per-project half -- config_read.rs's own comment) returns
// `ConfigItemSummary[]`, which deliberately carries no content/description for plans
// (`plan_summary` in config_read.rs always sends `description: String::new()`) -- unlike the
// Swift reference app's own eagerly-loaded `Plan.content`, there is no snippet to show or search in the list
// here, only the name; the full body only exists once a plan is opened (`getConfigItem`).
import { computed, ref } from 'vue';
import { defineStore } from 'pinia';
import { ipc } from '@/ipc/client';
import type { ConfigItemDetail, ConfigItemSummary, ConfigRef, Layout } from '@/ipc/bindings';
import { fuzzyMatch } from '@/lib/fuzzyMatch';
import { usePrefsStore } from '@/stores/prefs';

/** `navigationItems.ts`'s own id for this screen -- the key `configLayouts` is persisted under. */
const SCREEN_ID = 'plans';

export const usePlansStore = defineStore('plans', () => {
  const prefsStore = usePrefsStore();

  const plans = ref<ConfigItemSummary[]>([]);
  const isLoading = ref(true);
  const query = ref('');
  const selected = ref<ConfigItemDetail | null>(null);
  const isLoadingDetail = ref(false);

  // AC: "Layout choice persists per screen in configLayouts." Read fresh from the one Rust-owned
  // Prefs struct each time, not cached locally -- the same convention ProjectSessions.vue's own
  // single-screen `sessionLayout` already uses.
  const layout = computed<Layout>(() => prefsStore.prefs?.configLayouts?.[SCREEN_ID] ?? 'grid');

  const filteredPlans = computed<ConfigItemSummary[]>(() => {
    const q = query.value.trim();
    if (q === '') return plans.value;
    return plans.value
      .map((plan) => ({ plan, score: fuzzyMatch(plan.name, q)?.score ?? 0 }))
      .filter((r) => r.score > 0)
      .sort((a, b) => b.score - a.score)
      .map((r) => r.plan);
  });

  async function load() {
    isLoading.value = true;
    plans.value = (await ipc.listConfigItems('plan', null)).value;
    isLoading.value = false;
  }

  // `configLayouts` is a map, replaced wholesale by `update_prefs` (prefs.rs's own `apply_patch`:
  // `prefs.config_layouts = v`, not a per-key merge) -- spreading the current map here before
  // overriding this screen's own key is what keeps every other screen's saved layout intact.
  async function setLayout(next: Layout) {
    const current = prefsStore.prefs?.configLayouts ?? {};
    await prefsStore.update({ configLayouts: { ...current, [SCREEN_ID]: next } });
  }

  function toggleLayout() {
    return setLayout(layout.value === 'grid' ? 'list' : 'grid');
  }

  async function open(ref: ConfigRef) {
    isLoadingDetail.value = true;
    selected.value = null;
    selected.value = await ipc.getConfigItem(ref);
    isLoadingDetail.value = false;
  }

  function close() {
    selected.value = null;
  }

  return { plans, isLoading, query, selected, isLoadingDetail, layout, filteredPlans, load, setLayout, toggleLayout, open, close };
});
