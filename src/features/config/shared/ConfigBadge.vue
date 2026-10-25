<script setup lang="ts">
// Ported from ConfigBadge.swift:3-39. `ConfigBadge` (plain pill) and `ConfigScopeBadge`
// (globe/folder/puzzle + label, by `ConfigItemScope`) are folded into one component -- every
// later screen needs both and this task's own Outputs list has room for exactly one file.
// `MCPServerSourceBadge` (:41-77) is MCP-specific and has no caller anywhere in this port yet
// (M3-14's own job) -- left for that task to add rather than building a variant nothing uses.
//
// `ConfigItemScope` has a third case, `builtIn`, that Swift's own `ConfigScope` (global/project
// only) has no equivalent for -- it exists because Heimdall folds Sub-agents' built-in agents
// (no file, no scope at all in Swift) into the same scope enum as everything else. Rendered here
// as its own badge rather than left unhandled.
import AppIcon from '@/components/AppIcon.vue';
import type { ConfigItemScope } from '@/ipc/bindings';

defineProps<{
  text?: string;
  scope?: ConfigItemScope;
  fgClass?: string;
  bgClass?: string;
}>();

const SCOPE_META: Record<ConfigItemScope, { label: string; icon: string; fg: string; bg: string }> = {
  global: { label: 'Global', icon: 'globe', fg: 'text-(--text-tertiary)', bg: 'bg-(--bg-elevated)' },
  project: { label: 'Project', icon: 'folder.fill', fg: 'text-(--color-green)', bg: 'bg-(--color-green)/10' },
  builtIn: { label: 'Built-in', icon: 'puzzlepiece.fill', fg: 'text-(--accent)', bg: 'bg-(--accent-dim)' },
};
</script>

<template>
  <span
    v-if="scope"
    class="flex items-center gap-(--spacing-xxs) rounded-full px-(--spacing-sm) py-px text-(length:--text-tiny)"
    :class="[SCOPE_META[scope].fg, SCOPE_META[scope].bg]"
  >
    <AppIcon
      :name="SCOPE_META[scope].icon"
      size="sm"
    />
    {{ SCOPE_META[scope].label }}
  </span>
  <span
    v-else
    class="rounded-full px-(--spacing-sm) py-px text-(length:--text-tiny)"
    :class="[fgClass ?? 'text-(--text-tertiary)', bgClass ?? 'bg-(--bg-elevated)']"
  >{{ text }}</span>
</template>
