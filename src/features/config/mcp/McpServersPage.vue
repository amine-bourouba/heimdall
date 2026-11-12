<script setup lang="ts">
// Ported from MCPServersListView.swift (M3-14 list, M3-48 actions). The toolbar "+" opens the setup
// wizard, and a server from the global `~/.claude.json` (`source === 'user'`, the only place the writer
// touches; Swift gates its actions the same way) gets Edit (pencil: the wizard) and Remove (trash). Open
// in editor and Show in Finder (braces and folder in Swift) come with M3-55.
//
// Remove shows no `window.confirm`: `remove_mcp_server` raises its own native confirmation naming the
// server and what goes with it, and answers `cancelled` if it is declined (then nothing happens here).
// It removes the server and its `mcp__<name>__*` permissions from settings.json in one command.
//
// AC1: "env and header values are never displayed (only keys)." `McpServer.envKeys`/`headerKeys`
// are `string[]` -- confirmed directly in bindings.ts and mcp.rs's own header comment ("No
// response here carries a secret value... `env`/`headers` maps themselves [are] `#[serde(skip)]`")
// -- the redaction already happened before this file exists; there is no value to accidentally
// render even if this screen tried to.
//
// `McpServerSource` has 5 values here (user/local/project/plugin/cloudIntegration), not Swift's 3
// (user/cloudIntegration/plugin) -- config-screens.md §11's own open question, resolved: `user`
// and Heimdall's finer `local`/`project` all render the plain scope badge (ConfigBadge.vue,
// reused rather than duplicated into a new MCPServerSourceBadge component M3-11 deliberately left
// unbuilt for this task), `plugin`/`cloudIntegration` get their own badge text/colour inline below
// -- the same 3 visual variants Swift's own UI actually shows, just resolved from 5 source values
// instead of 3.
import { computed, ref, watch } from 'vue';
import { useConfigStore } from '../store';
import { ipc } from '@/ipc/client';
import { fuzzyMatch } from '@/lib/fuzzyMatch';
import { useToastsStore } from '@/stores/toasts';
import AppIcon from '@/components/AppIcon.vue';
import ConfigScreenHeader from '../shared/ConfigScreenHeader.vue';
import ConfigLayoutToolbar from '../shared/ConfigLayoutToolbar.vue';
import ConfigListLayout from '../shared/ConfigListLayout.vue';
import ConfigBadge from '../shared/ConfigBadge.vue';
import McpStatusIndicator from './McpStatusIndicator.vue';
import McpSetupWizard from './McpSetupWizard.vue';
import type { McpServer, McpServerStatus } from '@/ipc/bindings';

const SCREEN_ID = 'mcpServers';
const configStore = useConfigStore();
const toastsStore = useToastsStore();

const servers = ref<McpServer[]>([]);
const statuses = ref<Record<string, McpServerStatus>>({});
const isLoading = ref(true);
const filterQuery = ref('');
const expandedTools = ref<Set<string>>(new Set());

/** Re-reads the list in place (no skeleton), so a save or a remove does not flash the screen. */
async function refresh() {
  const project = configStore.selectedProject;
  const [serverList, statusMap] = await Promise.all([ipc.listMcpServers(project), ipc.getMcpStatuses(project)]);
  servers.value = serverList.value;
  statuses.value = statusMap;
}

async function load() {
  isLoading.value = true;
  try {
    await refresh();
  } catch (error) {
    toastsStore.showError(error, 'Could not load MCP servers');
  } finally {
    isLoading.value = false;
  }
}
watch(() => configStore.selectedProject, load, { deep: true, immediate: true });

// --- Actions ---------------------------------------------------------------------------------

const wizardOpen = ref(false);
const editing = ref<McpServer | null>(null);
// One write at a time: a second click while a ~/.claude.json write is in flight must not queue another.
const isBusy = ref(false);

/** Only a server from the global `~/.claude.json` can be written (config::write::mcp). */
const canWrite = (server: McpServer) => server.source === 'user';
/** The names the wizard must not reuse: the keys of that same file's `mcpServers`. */
const existingNames = computed(() => servers.value.filter(canWrite).map((s) => s.rawName));

function openAdd() {
  editing.value = null;
  wizardOpen.value = true;
}

function openEdit(server: McpServer) {
  editing.value = server;
  wizardOpen.value = true;
}

function closeWizard() {
  wizardOpen.value = false;
}

async function onSaved() {
  try {
    await refresh();
  } catch (error) {
    toastsStore.showError(error, 'Could not reload MCP servers');
  }
}

async function remove(server: McpServer) {
  if (isBusy.value) return;
  isBusy.value = true;
  try {
    const outcome = await ipc.removeMcpServer(server.rawName);
    if (outcome === 'cancelled') return; // declined in the native confirmation: nothing happened
    toastsStore.show(outcome === 'written' ? `Removed ${server.name}` : `${server.name} was already gone`, { style: 'info' });
    await refresh();
  } catch (error) {
    toastsStore.showError(error, 'Could not remove the server');
  } finally {
    isBusy.value = false;
  }
}

const filteredServers = computed<McpServer[]>(() => {
  const q = filterQuery.value.trim();
  if (q === '') return servers.value;
  return servers.value
    .map((server) => ({ server, score: fuzzyMatch(`${server.name} ${server.tools.join(' ')}`, q)?.score ?? 0 }))
    .filter((r) => r.score > 0)
    .sort((a, b) => b.score - a.score)
    .map((r) => r.server);
});

function transportLabel(server: McpServer): string | null {
  if (server.type) return server.type.toUpperCase();
  if (server.url) return 'HTTP';
  if (server.command) return 'STDIO';
  return null;
}

function connectionInfo(server: McpServer): string {
  return server.url ?? [server.command, ...server.args].filter(Boolean).join(' ');
}

const SOURCE_META: Partial<Record<McpServer['source'], { text: string; fg: string; bg: string }>> = {
  plugin: { text: '🧩 Built-in', fg: 'text-(--color-blue)', bg: 'bg-(--color-blue)/10' },
  cloudIntegration: { text: '☁ claude.ai', fg: 'text-(--color-purple)', bg: 'bg-(--color-purple)/10' },
};

function toggleTools(serverId: string) {
  if (expandedTools.value.has(serverId)) expandedTools.value.delete(serverId);
  else expandedTools.value.add(serverId);
}
function visibleTools(server: McpServer): string[] {
  return expandedTools.value.has(server.id) ? server.tools : server.tools.slice(0, 5);
}

const emptyMessage = computed(() => (filterQuery.value ? `No MCP servers match "${filterQuery.value}"` : 'No MCP servers found'));
const emptyHint = computed(() => (filterQuery.value ? 'Try a different search term' : 'Add one with claude mcp add, or edit ~/.claude.json directly'));
</script>

<template>
  <div class="flex h-full flex-col">
    <ConfigScreenHeader
      icon="powerplug.fill"
      title="MCP Servers"
      :count="servers.length"
      description="Model Context Protocol servers available to Claude Code."
    />
    <ConfigLayoutToolbar
      v-model:filter-query="filterQuery"
      :screen-id="SCREEN_ID"
      filter-placeholder="Find in MCP Servers…"
      show-project-picker
      show-add-button
      add-label="Add MCP server"
      @add="openAdd"
    />
    <div class="flex-1 overflow-y-auto px-(--spacing-xxxl) py-(--spacing-lg)">
      <p class="mb-(--spacing-lg) rounded-md bg-(--bg-elevated) p-(--spacing-md) text-(length:--text-small) text-(--text-tertiary)">
        Add servers with <code>claude mcp add</code>, or by editing <code>~/.claude.json</code> directly.
      </p>
      <ConfigListLayout
        :layout="configStore.layoutFor(SCREEN_ID)"
        :is-loading="isLoading"
        :items="filteredServers"
        empty-icon="powerplug.fill"
        :empty-message="emptyMessage"
        :empty-hint="emptyHint"
      >
        <template #default="{ item: server }">
          <div class="flex flex-col gap-(--spacing-xs) rounded-md border bg-(--bg-card) p-(--spacing-lg)">
            <div class="flex items-center gap-(--spacing-sm)">
              <McpStatusIndicator :status="statuses[server.id] ?? 'unknown'" />
              <p class="min-w-0 flex-1 truncate text-(length:--text-body-medium) text-(--text-primary)">
                {{ server.name }}
              </p>
              <ConfigBadge
                v-if="transportLabel(server)"
                :text="transportLabel(server)!"
              />
              <template v-if="canWrite(server)">
                <button
                  type="button"
                  title="Edit server"
                  :aria-label="`Edit ${server.name}`"
                  :disabled="isBusy"
                  class="shrink-0 rounded-sm p-(--spacing-xxs) text-(--text-tertiary) disabled:opacity-50"
                  @click="openEdit(server)"
                >
                  <AppIcon
                    name="pencil"
                    size="sm"
                  />
                </button>
                <button
                  type="button"
                  title="Remove server"
                  :aria-label="`Remove ${server.name}`"
                  :disabled="isBusy"
                  class="shrink-0 rounded-sm p-(--spacing-xxs) text-(--text-tertiary) disabled:opacity-50"
                  @click="remove(server)"
                >
                  <AppIcon
                    name="trash"
                    size="sm"
                  />
                </button>
              </template>
            </div>
            <p
              v-if="connectionInfo(server)"
              class="truncate text-(length:--text-code-small) text-(--text-tertiary)"
            >
              {{ connectionInfo(server) }}
            </p>
            <div class="flex flex-wrap items-center gap-(--spacing-xs)">
              <ConfigBadge
                v-if="SOURCE_META[server.source]"
                :text="SOURCE_META[server.source]!.text"
                :fg-class="SOURCE_META[server.source]!.fg"
                :bg-class="SOURCE_META[server.source]!.bg"
              />
              <ConfigBadge
                v-else
                :scope="server.scope"
              />
            </div>
            <div
              v-if="server.tools.length > 0"
              class="flex flex-wrap items-center gap-(--spacing-xs)"
            >
              <ConfigBadge
                v-for="tool in visibleTools(server)"
                :key="tool"
                :text="tool"
              />
              <button
                v-if="server.tools.length > 5"
                type="button"
                class="text-(length:--text-tiny) text-(--accent)"
                @click="toggleTools(server.id)"
              >
                {{ expandedTools.has(server.id) ? 'Show less' : `+${server.tools.length - 5} more` }}
              </button>
            </div>
          </div>
        </template>
      </ConfigListLayout>
    </div>
    <McpSetupWizard
      v-if="wizardOpen"
      :editing="editing"
      :existing-names="existingNames"
      :on-close="closeWizard"
      :on-saved="onSaved"
    />
  </div>
</template>
