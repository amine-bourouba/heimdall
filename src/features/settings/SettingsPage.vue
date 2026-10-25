<script setup lang="ts">
// Ported from SettingsView.swift (M3-16). Four of its five tabs (General, Appearance, Viewer,
// Memory) -- the Menu Bar tab and launch-at-login toggle come later (AC3, M3-58/M3-60). A routed
// page (port-plan.md P-6: "Settings... becomes an in-app view instead of a separate OS window"),
// not a sheet/overlay -- closed via "Done" (`useNavigation.goBack`, the same native-history back
// Mod+[ already uses), not Escape or a backdrop click.
//
// Every control here writes through an already-built mechanism, not new Rust work: `themeStore`
// (M2-05) owns colorTheme/fontScale; `memoryStore.addFolder`/`removeFolder` (M3-04) already drive
// the identical "Extra Folders" list Swift's own Memory tab shows; everything else is a plain
// `PrefsPatch` field written via `prefsStore.update`.
import { computed, onMounted, ref } from 'vue';
import { usePrefsStore } from '@/stores/prefs';
import { useThemeStore } from '@/stores/theme';
import { usePlatformStore } from '@/stores/platform';
import { useMemoryStore } from '@/features/memory/store';
import { useNavigation } from '@/composables/useNavigation';
import AppIcon from '@/components/AppIcon.vue';
import ThemePicker from './ThemePicker.vue';
import type { PreferredEditor, PreferredTerminal, TargetOs } from '@/ipc/bindings';

const prefsStore = usePrefsStore();
const themeStore = useThemeStore();
const platformStore = usePlatformStore();
const memoryStore = useMemoryStore();
const { goBack } = useNavigation();

onMounted(() => void memoryStore.load());

type TabId = 'general' | 'appearance' | 'viewer' | 'memory';
const TABS: { id: TabId; label: string; icon: string }[] = [
  { id: 'general', label: 'General', icon: 'gearshape' },
  { id: 'appearance', label: 'Appearance', icon: 'paintpalette' },
  { id: 'viewer', label: 'Viewer', icon: 'eye' },
  { id: 'memory', label: 'Memory', icon: 'brain.head.profile' },
];
const activeTab = ref<TabId>('general');

// Ported from prefs.rs's own PreferredEditor::options/PreferredTerminal::options (ADR 0003) --
// no IPC command exposes these, so the per-OS filtering is replicated here exactly, not
// re-derived from scratch (AC2: "Editor and terminal lists show only options valid for the
// running OS").
const EDITOR_LABELS: Record<PreferredEditor, string> = { vscode: 'VS Code', cursor: 'Cursor', xcode: 'Xcode', zed: 'Zed' };
function editorOptions(os: TargetOs | null): PreferredEditor[] {
  return (['vscode', 'cursor', 'xcode', 'zed'] as const).filter((e) => e !== 'xcode' || os === 'macos');
}

const TERMINAL_LABELS: Record<PreferredTerminal, string> = {
  terminal: 'Terminal',
  iterm2: 'iTerm2',
  warp: 'Warp',
  ghostty: 'Ghostty',
  kitty: 'Kitty',
  alacritty: 'Alacritty',
  windowsTerminal: 'Windows Terminal',
  powershell: 'PowerShell',
  cmd: 'Command Prompt',
  gnomeTerminal: 'GNOME Terminal',
  konsole: 'Konsole',
  system: 'System default',
};
function terminalOptions(os: TargetOs | null): PreferredTerminal[] {
  switch (os) {
    case 'macos':
      return ['terminal', 'iterm2', 'warp', 'ghostty', 'kitty', 'alacritty'];
    case 'windows':
      return ['windowsTerminal', 'powershell', 'cmd'];
    case 'linux':
      return ['gnomeTerminal', 'konsole', 'kitty', 'alacritty', 'system'];
    default:
      return [];
  }
}

const fontScalePercent = computed(() => Math.round(themeStore.fontScale * 100));
const customFolders = computed(() => memoryStore.sources.filter((s) => s.kind === 'folder'));

// Named handlers, not inline `($event.target as X).y` casts in the template: Vue's own template
// expression parser rejected a TS `as` cast used as an object-literal property value here
// (confirmed directly -- every inline attempt failed with the same "Unexpected token as" parse
// error, including the single-cast ones, not just the doubled `.value as PreferredEditor` case).
function onTextEditorChange(event: Event) {
  void prefsStore.update({ textEditor: (event.target as HTMLSelectElement).value as PreferredEditor });
}
function onPreferredTerminalChange(event: Event) {
  void prefsStore.update({ preferredTerminal: (event.target as HTMLSelectElement).value as PreferredTerminal });
}
function onOpenTerminalOnBashChange(event: Event) {
  void prefsStore.update({ openTerminalOnBash: (event.target as HTMLInputElement).checked });
}
function onClaudeCodePathChange(event: Event) {
  void prefsStore.update({ claudeCodePath: (event.target as HTMLInputElement).value || null });
}
function onShowAnimationsChange(event: Event) {
  void prefsStore.update({ showAnimations: (event.target as HTMLInputElement).checked });
}
function onWrapCodeLinesChange(event: Event) {
  void prefsStore.update({ wrapCodeLines: (event.target as HTMLInputElement).checked });
}
function onAutoExpandBlocksChange(event: Event) {
  void prefsStore.update({ autoExpandBlocks: (event.target as HTMLInputElement).checked });
}
function onParseMarkdownChange(event: Event) {
  void prefsStore.update({ parseMarkdownInResults: (event.target as HTMLInputElement).checked });
}
</script>

<template>
  <div class="flex h-full flex-col">
    <div class="flex shrink-0 items-center gap-(--spacing-md) border-b px-(--spacing-xxxl) py-(--spacing-lg)">
      <h1 class="flex-1 text-(length:--text-large) font-semibold">
        Settings
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

    <div
      role="tablist"
      aria-label="Settings tabs"
      class="flex shrink-0 gap-(--spacing-xs) border-b px-(--spacing-xxxl) py-(--spacing-sm)"
    >
      <button
        v-for="tab in TABS"
        :key="tab.id"
        type="button"
        role="tab"
        :title="tab.label"
        :aria-selected="activeTab === tab.id"
        class="flex items-center gap-(--spacing-xs) rounded-sm px-(--spacing-md) py-(--spacing-xs) text-(length:--text-caption)"
        :class="activeTab === tab.id ? 'bg-(--accent-dim) text-(--accent)' : 'text-(--text-secondary)'"
        @click="activeTab = tab.id"
      >
        <AppIcon
          :name="tab.icon"
          size="sm"
        />
        {{ tab.label }}
      </button>
    </div>

    <div class="flex-1 overflow-y-auto px-(--spacing-xxxl) py-(--spacing-xl)">
      <div
        v-if="activeTab === 'general'"
        class="flex max-w-150 flex-col gap-(--spacing-lg)"
      >
        <label class="flex items-center justify-between gap-(--spacing-lg)">
          <span class="text-(--text-secondary)">Default Editor</span>
          <select
            :value="prefsStore.prefs?.textEditor ?? 'vscode'"
            class="rounded-sm border bg-(--bg-elevated) px-(--spacing-sm) py-(--spacing-xs)"
            @change="onTextEditorChange"
          >
            <option
              v-for="editor in editorOptions(platformStore.os)"
              :key="editor"
              :value="editor"
            >
              {{ EDITOR_LABELS[editor] }}
            </option>
          </select>
        </label>

        <label class="flex items-center justify-between gap-(--spacing-lg)">
          <span class="text-(--text-secondary)">Terminal Application</span>
          <select
            :value="prefsStore.prefs?.preferredTerminal ?? 'terminal'"
            class="rounded-sm border bg-(--bg-elevated) px-(--spacing-sm) py-(--spacing-xs)"
            @change="onPreferredTerminalChange"
          >
            <option
              v-for="terminal in terminalOptions(platformStore.os)"
              :key="terminal"
              :value="terminal"
            >
              {{ TERMINAL_LABELS[terminal] }}
            </option>
          </select>
        </label>

        <label class="flex items-center justify-between gap-(--spacing-lg)">
          <span class="text-(--text-secondary)">Open terminal when copying bash commands</span>
          <input
            type="checkbox"
            class="accent-(--accent)"
            :checked="prefsStore.prefs?.openTerminalOnBash ?? false"
            @change="onOpenTerminalOnBashChange"
          >
        </label>

        <label class="flex flex-col gap-(--spacing-xs)">
          <span class="text-(--text-secondary)">Claude Code Path</span>
          <input
            type="text"
            placeholder="Searches PATH when empty"
            class="rounded-sm border bg-(--bg-elevated) px-(--spacing-sm) py-(--spacing-xs) text-(length:--text-code-small)"
            :value="prefsStore.prefs?.claudeCodePath ?? ''"
            @change="onClaudeCodePathChange"
          >
        </label>
      </div>

      <div
        v-else-if="activeTab === 'appearance'"
        class="flex max-w-150 flex-col gap-(--spacing-lg)"
      >
        <div class="flex flex-col gap-(--spacing-sm)">
          <span class="text-(--text-secondary)">Theme</span>
          <ThemePicker />
        </div>

        <label class="flex items-center justify-between gap-(--spacing-lg) border-t pt-(--spacing-lg)">
          <span class="text-(--text-secondary)">Message streaming animations</span>
          <input
            type="checkbox"
            class="accent-(--accent)"
            :checked="prefsStore.prefs?.showAnimations ?? true"
            @change="onShowAnimationsChange"
          >
        </label>

        <div class="flex items-center justify-between gap-(--spacing-lg) border-t pt-(--spacing-lg)">
          <span class="text-(--text-secondary)">Font Size</span>
          <div class="flex items-center gap-(--spacing-sm)">
            <button
              type="button"
              title="Decrease font size"
              class="flex h-7 w-7 items-center justify-center rounded-sm bg-(--bg-elevated)"
              @click="themeStore.decreaseFontScale()"
            >
              <AppIcon
                name="minus"
                size="sm"
              />
            </button>
            <span class="w-12 text-center tabular-nums">{{ fontScalePercent }}%</span>
            <button
              type="button"
              title="Increase font size"
              class="flex h-7 w-7 items-center justify-center rounded-sm bg-(--bg-elevated)"
              @click="themeStore.increaseFontScale()"
            >
              <AppIcon
                name="plus"
                size="sm"
              />
            </button>
            <button
              type="button"
              title="Reset font size to 100%"
              class="text-(length:--text-tiny) text-(--accent) disabled:text-(--text-tertiary)"
              :disabled="themeStore.fontScale === 1.0"
              @click="themeStore.resetFontScale()"
            >
              Reset
            </button>
          </div>
        </div>
      </div>

      <div
        v-else-if="activeTab === 'viewer'"
        class="flex max-w-150 flex-col gap-(--spacing-lg)"
      >
        <label class="flex items-center justify-between gap-(--spacing-lg)">
          <span class="text-(--text-secondary)">Wrap lines in code blocks</span>
          <input
            type="checkbox"
            class="accent-(--accent)"
            :checked="prefsStore.prefs?.wrapCodeLines ?? true"
            @change="onWrapCodeLinesChange"
          >
        </label>

        <label class="flex items-center justify-between gap-(--spacing-lg)">
          <span class="text-(--text-secondary)">Expand tool blocks automatically</span>
          <input
            type="checkbox"
            class="accent-(--accent)"
            :checked="prefsStore.prefs?.autoExpandBlocks ?? true"
            @change="onAutoExpandBlocksChange"
          >
        </label>

        <label class="flex items-center justify-between gap-(--spacing-lg)">
          <span class="text-(--text-secondary)">Render markdown in tool results</span>
          <input
            type="checkbox"
            class="accent-(--accent)"
            :checked="prefsStore.prefs?.parseMarkdownInResults ?? true"
            @change="onParseMarkdownChange"
          >
        </label>
      </div>

      <div
        v-else
        class="flex max-w-150 flex-col gap-(--spacing-sm)"
      >
        <span class="text-(--text-secondary)">Extra Folders</span>
        <p
          v-if="customFolders.length === 0"
          class="text-(length:--text-small) text-(--text-tertiary)"
        >
          No extra folders yet. Add one to show its <code>.md</code> files in the Memory tab, alongside your project memory.
        </p>
        <div
          v-for="folder in customFolders"
          :key="folder.id"
          class="flex items-center gap-(--spacing-sm)"
        >
          <AppIcon
            name="folder"
            size="sm"
            class="shrink-0 text-(--text-tertiary)"
          />
          <span
            class="min-w-0 flex-1 truncate text-(length:--text-caption)"
            :title="folder.label"
          >{{ folder.label }}</span>
          <button
            type="button"
            title="Remove folder"
            @click="memoryStore.removeFolder(folder.id)"
          >
            <AppIcon
              name="minus.circle.fill"
              size="sm"
              class="text-(--text-tertiary)"
            />
          </button>
        </div>
        <button
          type="button"
          class="mt-(--spacing-xs) flex w-fit items-center gap-(--spacing-xs) rounded-sm border border-(--accent)/30 bg-(--accent-dim) px-(--spacing-md) py-(--spacing-xs) text-(--accent)"
          @click="memoryStore.addFolder()"
        >
          <AppIcon
            name="plus"
            size="sm"
          />
          Add Folder…
        </button>
      </div>
    </div>
  </div>
</template>
