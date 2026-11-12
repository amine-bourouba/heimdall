<script setup lang="ts">
// Ported from PluginsListView.swift (M3-13 read-only shell + M3-50 actions). No project picker
// (config-screens.md §1.2: "Not shown for... Plugins... no project scope at all") and no detail
// route (§1.7: ConfigItemDetailView is used only by Commands/Skills/OutputStyles) -- list_plugins
// itself takes no project argument at all, confirmed directly in bindings.ts.
//
// Actions (M3-50). The Enabled/Disabled pill is the toggle: Swift's pill opens a popover holding a
// single Enable/Disable button, an extra click for a reversible change, so here the pill itself
// flips the state (a `switch`, named "<plugin>: Enabled"). The trash button is Swift's "Remove
// plugin", and it does only what Swift's does: `remove_plugin` deletes the plugin's entry from
// `enabledPlugins` in settings.json and nothing else. The plugin stays installed
// (installed_plugins.json is untouched), so it is still listed here, as Disabled -- the
// confirmation and the toast say so rather than imply the files are gone. Confirmation is the
// webview's `window.confirm`, the decision recorded in the progress log for hook and sub-agent deletes.
import { computed, ref } from 'vue';
import { ipc } from '@/ipc/client';
import { fuzzyMatch } from '@/lib/fuzzyMatch';
import { formatRelativeTime } from '@/features/session/format';
import { useToastsStore } from '@/stores/toasts';
import { useConfigStore } from '../store';
import ConfigScreenHeader from '../shared/ConfigScreenHeader.vue';
import ConfigLayoutToolbar from '../shared/ConfigLayoutToolbar.vue';
import ConfigListLayout from '../shared/ConfigListLayout.vue';
import ConfigBadge from '../shared/ConfigBadge.vue';
import AppIcon from '@/components/AppIcon.vue';
import type { PluginInfo } from '@/ipc/bindings';

const SCREEN_ID = 'plugins';
const configStore = useConfigStore();
const toastsStore = useToastsStore();

const items = ref<PluginInfo[]>([]);
const isLoading = ref(true);
const filterQuery = ref('');
// One write at a time: a second click while a settings.json write is in flight must not queue another.
const isBusy = ref(false);

/** Re-reads the list in place (no skeleton), so a toggle does not flash the screen or lose the scroll position. */
async function refresh() {
  items.value = await ipc.listPlugins();
}

async function load() {
  isLoading.value = true;
  try {
    await refresh();
  } catch (error) {
    toastsStore.showError(error, 'Could not load plugins');
  } finally {
    isLoading.value = false;
  }
}
void load();

const filteredItems = computed<PluginInfo[]>(() => {
  const q = filterQuery.value.trim();
  if (q === '') return items.value;
  return items.value
    .map((item) => ({ item, score: fuzzyMatch(`${item.name} ${item.author}`, q)?.score ?? 0 }))
    .filter((r) => r.score > 0)
    .sort((a, b) => b.score - a.score)
    .map((r) => r.item);
});

// PluginsListView.swift:177-190's own installedAt parse: a non-ISO-8601 string (or none at all)
// means no "Installed X ago" line, not a fallback label.
function installedAgo(iso: string | null): string | null {
  if (!iso) return null;
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return null;
  return `Installed ${formatRelativeTime(iso)}`;
}

async function toggle(item: PluginInfo) {
  if (isBusy.value) return;
  isBusy.value = true;
  const enable = !item.enabled;
  try {
    await ipc.setPluginEnabled(item.key, enable);
    // Swift: "<name> enabled" as a success, "<name> disabled" as plain info.
    toastsStore.show(`${item.name} ${enable ? 'enabled' : 'disabled'}`, { style: enable ? 'success' : 'info' });
    await refresh();
  } catch (error) {
    toastsStore.showError(error, `Could not ${enable ? 'enable' : 'disable'} ${item.name}`);
  } finally {
    isBusy.value = false;
  }
}

async function remove(item: PluginInfo) {
  if (isBusy.value) return;
  const message = `Remove "${item.name}" from your enabled plugins in settings.json? The plugin stays installed, so it is still listed here as Disabled.`;
  if (!window.confirm(message)) return;
  isBusy.value = true;
  try {
    await ipc.removePlugin(item.key);
    toastsStore.show(`Removed ${item.name} from enabled plugins`, { style: 'info' });
    await refresh();
  } catch (error) {
    toastsStore.showError(error, `Could not remove ${item.name}`);
  } finally {
    isBusy.value = false;
  }
}

const emptyMessage = computed(() => (filterQuery.value ? `No plugins match "${filterQuery.value}"` : 'No plugins found'));
const emptyHint = computed(() => (filterQuery.value ? 'Try a different search term' : 'Install via claude plugin add'));
</script>

<template>
  <div class="flex h-full flex-col">
    <ConfigScreenHeader
      icon="puzzlepiece.fill"
      title="Plugins"
      :count="items.length"
      description="Plugins installed for Claude Code."
    />
    <ConfigLayoutToolbar
      v-model:filter-query="filterQuery"
      :screen-id="SCREEN_ID"
      filter-placeholder="Find in Plugins…"
    />
    <div class="flex-1 overflow-y-auto px-(--spacing-xxxl) py-(--spacing-lg)">
      <p class="mb-(--spacing-lg) rounded-md bg-(--bg-elevated) p-(--spacing-md) text-(length:--text-small) text-(--text-tertiary)">
        Plugins are installed via <code>claude plugin add</code> or by placing files under <code>~/.claude/plugins/</code>.
      </p>
      <ConfigListLayout
        :layout="configStore.layoutFor(SCREEN_ID)"
        :is-loading="isLoading"
        :items="filteredItems"
        empty-icon="puzzlepiece.fill"
        :empty-message="emptyMessage"
        :empty-hint="emptyHint"
      >
        <template #default="{ item }">
          <div class="flex flex-col gap-(--spacing-xs) rounded-md border bg-(--bg-card) p-(--spacing-lg)">
            <div class="flex items-start gap-(--spacing-sm)">
              <p class="min-w-0 flex-1 truncate text-(length:--text-body-medium) text-(--text-primary)">
                {{ item.name }}
              </p>
              <button
                type="button"
                role="switch"
                :aria-checked="item.enabled"
                :aria-label="`${item.name}: ${item.enabled ? 'Enabled' : 'Disabled'}`"
                :title="item.enabled ? 'Disable plugin' : 'Enable plugin'"
                :disabled="isBusy"
                class="flex shrink-0 items-center gap-(--spacing-xs) rounded-full px-(--spacing-sm) py-px text-(length:--text-tiny) disabled:opacity-50"
                :class="item.enabled ? 'bg-(--color-green)/10 text-(--color-green)' : 'bg-(--bg-elevated) text-(--text-tertiary)'"
                @click="toggle(item)"
              >
                <span
                  aria-hidden="true"
                  class="h-1.5 w-1.5 rounded-full"
                  :class="item.enabled ? 'bg-(--color-green)' : 'bg-(--text-tertiary)/50'"
                />
                {{ item.enabled ? 'Enabled' : 'Disabled' }}
              </button>
              <button
                type="button"
                :aria-label="`Remove ${item.name} from enabled plugins`"
                title="Remove from enabled plugins"
                :disabled="isBusy"
                class="shrink-0 rounded-sm p-(--spacing-xxs) text-(--text-tertiary) disabled:opacity-50"
                @click="remove(item)"
              >
                <AppIcon
                  name="trash"
                  size="sm"
                />
              </button>
            </div>
            <p
              v-if="item.author"
              class="text-(length:--text-small) text-(--text-tertiary)"
            >
              by {{ item.author }}
            </p>
            <div class="flex flex-wrap items-center gap-(--spacing-xs) text-(length:--text-tiny) text-(--text-tertiary)">
              <ConfigBadge
                v-if="item.version"
                :text="`v${item.version}`"
              />
              <span v-if="installedAgo(item.installedAt)">{{ installedAgo(item.installedAt) }}</span>
            </div>
          </div>
        </template>
      </ConfigListLayout>
    </div>
  </div>
</template>
