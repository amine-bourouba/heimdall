<script setup lang="ts">
// Ported from MCPServersListView.swift (M3-14). Read-only: no "+" (wizard, M3-48), no per-card
// pencil/braces/folder/trash (M3-45/M3-48) -- `server.source === 'user'` would gate those in
// Swift, but none of the four actions exist here yet, so nothing is rendered at all rather than
// shown disabled.
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
import ConfigScreenHeader from '../shared/ConfigScreenHeader.vue';
import ConfigLayoutToolbar from '../shared/ConfigLayoutToolbar.vue';
import ConfigListLayout from '../shared/ConfigListLayout.vue';
import ConfigBadge from '../shared/ConfigBadge.vue';
import McpStatusIndicator from './McpStatusIndicator.vue';
import type { McpServer, McpServerStatus } from '@/ipc/bindings';

const SCREEN_ID = 'mcpServers';
const configStore = useConfigStore();

const servers = ref<McpServer[]>([]);
const statuses = ref<Record<string, McpServerStatus>>({});
const isLoading = ref(true);
const filterQuery = ref('');
const expandedTools = ref<Set<string>>(new Set());

async function load() {
  isLoading.value = true;
  const project = configStore.selectedProject;
  const [serverList, statusMap] = await Promise.all([ipc.listMcpServers(project), ipc.getMcpStatuses(project)]);
  servers.value = serverList.value;
  statuses.value = statusMap;
  isLoading.value = false;
}
watch(() => configStore.selectedProject, load, { deep: true, immediate: true });

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
  </div>
</template>
