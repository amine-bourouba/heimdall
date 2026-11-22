<script setup lang="ts">
// Ported from SessionToolbarContent.swift (docs/spec/session-views.md §6). This task's own Goal
// names exactly six things to build: copy resume, copy session id, file-history toggle,
// debug-log opener (when a log exists), export, and expand-all -- the search field is also built
// (it is `SessionToolbarFilterField`, squarely this same Swift file, and M2-20 already built the
// search/highlight *behaviour* it drives with nothing to actually type a query into). Terminal
// launch and "Show in Finder"/reveal are explicitly out of scope (this task's AC2, M3-55's own
// job) -- so is "Copy File Name": Heimdall never exposes a raw file path to the webview at all
// (commands take ids, never paths, AI_INSTRUCTIONS.md), so "copy session id" (this task's own
// Goal wording, not Swift's "copy file name") copies `summary.id` directly, the natural
// architecture-correct adaptation. Delete (session-views.md §6 item 10) was left for whichever task
// owns it: M3-51 added it (the trash button at the end, with Rust's native confirmation). Close
// (item 11) still is not here.
//
// `claude --resume <id>` (AC1) is hard-coded, not built from a `claudeCodePath` preference --
// the AC's own wording is an exact, literal string, not "whatever CLI the user configured".
import { ref } from 'vue';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import AppIcon from '@/components/AppIcon.vue';
import { useLauncher } from '@/composables/useLauncher';
import { useSessionStore } from './store';
import { useDeleteActions } from './useDeleteActions';
import type { SessionSummary } from '@/ipc/bindings';

const props = defineProps<{ summary: SessionSummary }>();

const sessionStore = useSessionStore();
const { isBusy: isDeleting, deleteSession } = useDeleteActions();
const launcher = useLauncher();

const resumeCopied = ref(false);
const idCopied = ref(false);

// M3-55 (SessionToolbarContent.swift:105-124): with `openTerminalOnBash` on, and a terminal that exists on
// this OS, resuming also opens the preferred terminal at the project. Rust puts the same
// `claude --resume <id>` on the clipboard again; the copy above stays so the button's own feedback and
// the literal text do not depend on the terminal call succeeding.
async function copyResume() {
  await writeText(`claude --resume ${props.summary.id}`);
  resumeCopied.value = true;
  setTimeout(() => (resumeCopied.value = false), 4000);
  if (launcher.opensTerminalOnCopy.value) void launcher.openTerminal(props.summary.projectId, props.summary.id);
}

function revealSession() {
  void launcher.reveal({ kind: 'session', projectId: props.summary.projectId, sessionId: props.summary.id });
}

async function copySessionId() {
  await writeText(props.summary.id);
  idCopied.value = true;
  setTimeout(() => (idCopied.value = false), 4000);
}

function toggleFileHistory() {
  sessionStore.isShowingFileHistory = !sessionStore.isShowingFileHistory;
}

function openDebugLog() {
  sessionStore.showDebugLogSessionId = props.summary.id;
}

function openExport() {
  sessionStore.isShowingExportOptions = true;
}

function toggleExpandAll() {
  sessionStore.allBlocksExpanded = !sessionStore.allBlocksExpanded;
}

function onSearchInput(value: string) {
  sessionStore.sessionSearchQuery = value;
  sessionStore.isSessionSearchActive = value !== '';
}

function clearSearch() {
  onSearchInput('');
}
</script>

<template>
  <div
    role="toolbar"
    aria-label="Session actions"
    class="flex flex-wrap items-center gap-(--spacing-xs)"
  >
    <div class="relative flex items-center">
      <AppIcon
        name="magnifyingglass"
        size="sm"
        class="pointer-events-none absolute left-2 text-(--text-tertiary)"
      />
      <input
        type="text"
        placeholder="Find in session…"
        class="w-[200px] rounded-sm border border-(--border) bg-(--bg-elevated) py-(--spacing-xs) pl-7 pr-7 text-(length:--text-small) text-(--text-primary)"
        :value="sessionStore.sessionSearchQuery"
        @input="onSearchInput(($event.target as HTMLInputElement).value)"
      >
      <button
        v-if="sessionStore.sessionSearchQuery"
        type="button"
        aria-label="Clear search"
        class="absolute right-2 text-(--text-tertiary)"
        @click="clearSearch"
      >
        <AppIcon
          name="xmark.circle.fill"
          size="sm"
        />
      </button>
    </div>

    <button
      type="button"
      :title="resumeCopied ? 'Copied' : launcher.opensTerminalOnCopy.value ? `Resume (copy and open ${launcher.terminalName.value})` : 'Resume'"
      class="flex items-center gap-(--spacing-xs) rounded-sm p-(--spacing-xs) text-(--text-secondary)"
      @click="copyResume"
    >
      <AppIcon
        :name="resumeCopied ? 'checkmark' : 'arrow.uturn.forward'"
        size="sm"
        :class="resumeCopied ? 'text-(--color-green)' : ''"
      />
      <span class="text-(length:--text-small)">{{ resumeCopied ? 'Copied' : 'Resume' }}</span>
    </button>

    <button
      type="button"
      title="Copy session ID"
      class="rounded-sm p-(--spacing-xs) text-(--text-secondary)"
      @click="copySessionId"
    >
      <AppIcon
        :name="idCopied ? 'checkmark' : 'doc.on.doc'"
        size="sm"
        :class="idCopied ? 'text-(--color-green)' : ''"
      />
    </button>

    <button
      type="button"
      :title="`Show in ${launcher.fileManager.value}`"
      :aria-label="`Show in ${launcher.fileManager.value}`"
      class="rounded-sm p-(--spacing-xs) text-(--text-secondary)"
      @click="revealSession"
    >
      <AppIcon
        name="folder"
        size="sm"
      />
    </button>

    <button
      type="button"
      title="File History"
      :disabled="!summary.hasFileHistory"
      class="rounded-sm p-(--spacing-xs) text-(--text-secondary) disabled:opacity-40"
      :class="sessionStore.isShowingFileHistory ? 'text-(--accent)' : ''"
      @click="toggleFileHistory"
    >
      <AppIcon
        name="clock.arrow.2.circlepath"
        size="sm"
      />
    </button>

    <button
      type="button"
      title="Export Session"
      class="rounded-sm p-(--spacing-xs) text-(--text-secondary)"
      @click="openExport"
    >
      <AppIcon
        name="square.and.arrow.up"
        size="sm"
      />
    </button>

    <button
      v-if="summary.hasDebugLog"
      type="button"
      title="Debug Log"
      class="rounded-sm p-(--spacing-xs) text-(--text-secondary)"
      @click="openDebugLog"
    >
      <AppIcon
        name="ladybug"
        size="sm"
      />
    </button>

    <button
      type="button"
      :title="sessionStore.allBlocksExpanded ? 'Collapse All' : 'Expand All'"
      class="rounded-sm p-(--spacing-xs) text-(--text-secondary)"
      @click="toggleExpandAll"
    >
      <AppIcon
        :name="sessionStore.allBlocksExpanded ? 'arrow.down.right.and.arrow.up.left' : 'arrow.up.left.and.arrow.down.right'"
        size="sm"
      />
    </button>

    <button
      type="button"
      title="Delete session"
      :disabled="isDeleting"
      class="rounded-sm p-(--spacing-xs) text-(--text-secondary) disabled:opacity-50"
      @click="deleteSession(summary.id)"
    >
      <AppIcon
        name="trash"
        size="sm"
      />
    </button>
  </div>
</template>
