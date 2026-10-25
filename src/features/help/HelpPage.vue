<script setup lang="ts">
// Ported from HelpView.swift (M3-17). A routed page (the same port-plan.md P-6 treatment M3-16
// gave Settings), closed via "Done" (`useNavigation.goBack`), not a sheet -- "Show All" dispatches
// the real 'shortcuts' action instead of owning a local `showShortcuts` boolean the way Swift's
// view does, since ShortcutHelp.vue (mounted once in AppShell.vue) already owns that overlay's
// open state.
import { BINDINGS } from '@/composables/useShortcuts';
import { useAppActions } from '@/composables/useAppActions';
import { useNavigation } from '@/composables/useNavigation';
import AppIcon from '@/components/AppIcon.vue';
import ShortcutLabel from '@/components/ShortcutLabel.vue';

const SHORTCUT_LABELS: Record<string, string> = {
  search: 'Universal Search',
  back: 'Navigate Back',
  forward: 'Navigate Forward',
  toolFilter: 'Toggle Tool Filter',
};

const FEATURES = [
  { title: 'Session Browser', description: 'Browse and search conversation history grouped by project' },
  { title: 'Session Detail', description: 'Rich timeline with markdown, code diffs, and tool output' },
  { title: 'Universal Search', description: 'Search across sessions, commands, skills, and more' },
  { title: 'Configuration', description: 'Manage commands, skills, MCP servers, models, and plugins' },
  { title: 'Live Reload', description: 'Auto-refreshes when session files change on disk' },
  { title: 'Editor Integration', description: 'Open files directly in VS Code, Cursor, Xcode, or Zed' },
];

const { goBack } = useNavigation();
const { dispatch } = useAppActions();
</script>

<template>
  <div class="flex h-full flex-col">
    <div class="flex shrink-0 items-center gap-(--spacing-md) border-b px-(--spacing-xxxl) py-(--spacing-lg)">
      <h1 class="flex-1 text-(length:--text-large) font-semibold">
        Heimdall Help
      </h1>
      <button
        type="button"
        title="Done"
        class="rounded-sm border border-(--accent)/30 bg-(--accent-dim) px-(--spacing-lg) py-(--spacing-xs) text-(--accent)"
        @click="goBack"
      >
        Done
      </button>
    </div>

    <div class="flex-1 overflow-y-auto px-(--spacing-xxxl) py-(--spacing-xl)">
      <div class="flex max-w-150 flex-col gap-(--spacing-xxl)">
        <div class="flex items-center gap-(--spacing-md)">
          <AppIcon
            name="questionmark.circle.fill"
            size="lg"
            class="text-(--accent)"
          />
          <p class="text-(--text-secondary)">
            A cross-platform companion for Claude Code.
          </p>
        </div>

        <div class="flex flex-col gap-(--spacing-md)">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-(--spacing-sm)">
              <AppIcon
                name="keyboard"
                size="sm"
                class="text-(--accent)"
              />
              <h2 class="font-semibold">
                Keyboard Shortcuts
              </h2>
            </div>
            <button
              type="button"
              title="Show all keyboard shortcuts"
              class="text-(length:--text-small) text-(--accent)"
              @click="dispatch('shortcuts')"
            >
              Show All
            </button>
          </div>
          <div class="flex flex-col gap-(--spacing-xs) rounded-md border bg-(--bg-card) p-(--spacing-lg)">
            <div
              v-for="binding in BINDINGS.filter((b) => b.action in SHORTCUT_LABELS)"
              :key="binding.action"
              class="flex items-center justify-between"
            >
              <span class="text-(--text-secondary)">{{ SHORTCUT_LABELS[binding.action] }}</span>
              <ShortcutLabel :key-label="binding.key" />
            </div>
          </div>
        </div>

        <div class="flex flex-col gap-(--spacing-md)">
          <div class="flex items-center gap-(--spacing-sm)">
            <AppIcon
              name="star"
              size="sm"
              class="text-(--accent)"
            />
            <h2 class="font-semibold">
              Features
            </h2>
          </div>
          <div class="flex flex-col gap-(--spacing-md) rounded-md border bg-(--bg-card) p-(--spacing-lg)">
            <div
              v-for="feature in FEATURES"
              :key="feature.title"
              class="flex items-start gap-(--spacing-sm)"
            >
              <AppIcon
                name="checkmark.circle.fill"
                size="sm"
                class="mt-0.5 shrink-0 text-(--color-green)"
              />
              <div class="flex flex-col">
                <span class="font-medium">{{ feature.title }}</span>
                <span class="text-(length:--text-small) text-(--text-tertiary)">{{ feature.description }}</span>
              </div>
            </div>
          </div>
        </div>

        <div class="flex flex-col gap-(--spacing-md)">
          <div class="flex items-center gap-(--spacing-sm)">
            <AppIcon
              name="arrow.right.circle"
              size="sm"
              class="text-(--accent)"
            />
            <h2 class="font-semibold">
              Getting Started
            </h2>
          </div>
          <div class="flex flex-col gap-(--spacing-sm) rounded-md border bg-(--bg-card) p-(--spacing-lg) text-(length:--text-small) text-(--text-secondary)">
            <p>Heimdall reads session transcripts from <strong>~/.claude/projects/</strong>. Use Claude Code CLI normally — Heimdall will automatically discover and display your sessions.</p>
            <p class="flex flex-wrap items-center gap-(--spacing-xxs)">
              Configure your preferred text editor and terminal in <strong>Settings</strong> (<ShortcutLabel key-label="," />) to enable one-click file opening and command re-runs.
            </p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
