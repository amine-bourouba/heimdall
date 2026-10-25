<script setup lang="ts">
// Ported from ExportOptionsView.swift (M3-10). Mounted only while
// `sessionStore.isShowingExportOptions` is true (SessionDetail.vue's own `v-if`); that flag alone
// carries no session, so `projectId`/`sessionId` come in as props from there, the same convention
// FileHistory.vue (M3-07) already established for a dialog whose owning store flag is a bare
// boolean/id with nothing else attached.
import { ref } from 'vue';
import { ipc } from '@/ipc/client';
import type { ExportOptions } from '@/ipc/bindings';
import AppIcon from '@/components/AppIcon.vue';
import { useSessionStore } from './store';

const props = defineProps<{ projectId: string; sessionId: string }>();

const sessionStore = useSessionStore();

// session_markdown.rs's own `Default for ExportOptions` (confirmed directly, matches
// SessionExporter.swift:7-12's own defaults exactly): timestamps and tool results on by default,
// thinking and token usage off.
const options = ref<ExportOptions>({ timestamps: true, toolResults: true, thinking: false, tokenUsage: false });
const copied = ref(false);

function close() {
  sessionStore.isShowingExportOptions = false;
}

async function copyMarkdown() {
  await ipc.exportSession(props.projectId, props.sessionId, options.value, 'clipboard');
  copied.value = true;
  setTimeout(() => (copied.value = false), 1500);
}

// Unlike ExportOptionsView.swift's own exportMarkdown(), which always dismisses regardless of
// whether the save panel was confirmed (Swift's own presentMarkdownSavePanel has no return value
// to check at all) -- export_session returns a real WriteOutcome, so this only closes on
// "written", leaving the dialog open on "cancelled" rather than silently discarding the user's
// chosen options along with a dialog they may just want to retry.
async function exportToFile() {
  const outcome = await ipc.exportSession(props.projectId, props.sessionId, options.value, 'file');
  if (outcome === 'written') close();
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') close();
}
</script>

<template>
  <Teleport to="body">
    <div
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/40"
      @click="close"
      @keydown="onKeydown"
    >
      <div
        role="dialog"
        aria-modal="true"
        aria-label="Export Session"
        class="glass flex w-75 flex-col gap-(--spacing-lg) rounded-md p-(--spacing-lg)"
        @click.stop
      >
        <div class="flex items-center gap-(--spacing-sm)">
          <AppIcon
            name="square.and.arrow.up"
            size="sm"
            class="text-(--accent)"
          />
          <h2 class="font-semibold text-(--text-primary)">
            Export Session
          </h2>
        </div>

        <div class="flex flex-col gap-(--spacing-sm)">
          <p class="font-medium text-(--text-secondary) text-(length:--text-tiny)">
            Options
          </p>
          <label
            v-for="opt in (['timestamps', 'toolResults', 'thinking', 'tokenUsage'] as const)"
            :key="opt"
            class="flex items-center gap-(--spacing-sm) text-(length:--text-caption) text-(--text-primary)"
          >
            <input
              v-model="options[opt]"
              type="checkbox"
              class="accent-(--accent)"
            >
            {{
              {
                timestamps: 'Include timestamps',
                toolResults: 'Include tool results',
                thinking: 'Include thinking blocks',
                tokenUsage: 'Include token usage',
              }[opt]
            }}
          </label>
        </div>

        <div class="border-t" />

        <div class="flex items-center gap-(--spacing-sm)">
          <button
            type="button"
            title="Copy session as Markdown"
            class="rounded-sm border px-(--spacing-md) py-(--spacing-xs) text-(length:--text-caption)"
            @click="copyMarkdown"
          >
            {{ copied ? 'Copied' : 'Copy' }}
          </button>
          <div class="flex-1" />
          <button
            type="button"
            title="Export session as Markdown"
            class="rounded-sm border border-(--accent)/30 bg-(--accent-dim) px-(--spacing-md) py-(--spacing-xs) text-(--accent) text-(length:--text-caption)"
            @click="exportToFile"
          >
            Export…
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
