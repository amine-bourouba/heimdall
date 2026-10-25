<script setup lang="ts">
// Ported from EditDiffView.swift. Unlike the Swift reference app -- which recomputes `LineDiff.diff` itself on a
// background task, capping each side at `maxLines = 5000` first -- the diff, cap included, is
// already computed in Rust (`diff_lines`, M1-42's `DiffResult`); this component only fetches and
// renders it. Copy-as-image is explicitly out of scope (M3-39's own job). Save-as-patch (M3-10)
// calls `export_edit_patch`, which resolves this exact block server-side from `blockId` alone
// (ADR 0010: commands take ids, never the file content or path the webview already has) -- it
// never needs `oldString`/`newString` itself, those stay this component's own diff-fetch inputs.
import { ref, watch } from 'vue';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import AppIcon from '@/components/AppIcon.vue';
import DiffTable from './DiffTable.vue';
import { ipc } from '@/ipc/client';
import type { DiffResult } from '@/ipc/bindings';

const props = defineProps<{ oldString: string; newString: string; projectId: string; sessionId: string; blockId: string }>();

const result = ref<DiffResult | null>(null);
const copied = ref(false);
const savedAsPatch = ref(false);

async function load() {
  result.value = null;
  result.value = await ipc.diffLines(props.oldString, props.newString);
}

watch(() => [props.oldString, props.newString], load, { immediate: true });

// LineDiff.unifiedText, ported verbatim: a " "/"+"/"-" prefix per line, joined by "\n".
function unifiedText(): string {
  if (!result.value) return '';
  return result.value.lines.map((line) => `${indicator(line.kind)}${line.text}`).join('\n');
}

function indicator(kind: DiffResult['lines'][number]['kind']): string {
  return kind === 'add' ? '+' : kind === 'remove' ? '-' : ' ';
}

async function copy() {
  await writeText(unifiedText());
  copied.value = true;
  setTimeout(() => (copied.value = false), 1500);
}

// WriteOutcome::Cancelled (the user closed the save dialog) gets no checkmark flash -- nothing
// was actually saved, same reasoning ExportOptionsDialog.vue's own exportToFile() uses.
async function saveAsPatch() {
  const outcome = await ipc.exportEditPatch(props.projectId, props.sessionId, props.blockId);
  if (outcome !== 'written') return;
  savedAsPatch.value = true;
  setTimeout(() => (savedAsPatch.value = false), 1500);
}
</script>

<template>
  <div
    data-testid="edit-diff"
    class="relative bg-(--bg-code)"
  >
    <p
      v-if="!result"
      class="p-(--spacing-lg) text-center text-(length:--text-code) text-(--text-tertiary)"
    >
      Loading diff…
    </p>
    <template v-else>
      <p
        v-if="result.truncated"
        class="border-b border-(--border)/30 p-(--spacing-xs) text-center text-(length:--text-tiny) text-(--text-tertiary)"
      >
        Diff truncated to the first 5000 lines of each side
      </p>
      <DiffTable
        :lines="result.lines"
        :max-line-number="result.maxLineNumber"
      />
      <div class="absolute right-(--spacing-sm) top-(--spacing-sm) flex gap-(--spacing-xs)">
        <button
          type="button"
          title="Save as patch"
          class="flex h-6.5 w-6.5 items-center justify-center rounded-sm bg-(--bg-elevated)"
          @click="saveAsPatch"
        >
          <AppIcon
            :name="savedAsPatch ? 'checkmark' : 'arrow.down.doc'"
            size="sm"
            :class="savedAsPatch ? 'text-(--color-green)' : 'text-(--text-tertiary)'"
          />
        </button>
        <button
          type="button"
          title="Copy diff text"
          class="flex h-6.5 w-6.5 items-center justify-center rounded-sm bg-(--bg-elevated)"
          @click="copy"
        >
          <AppIcon
            :name="copied ? 'checkmark' : 'doc.on.doc'"
            size="sm"
            :class="copied ? 'text-(--color-green)' : 'text-(--text-tertiary)'"
          />
        </button>
      </div>
    </template>
  </div>
</template>
