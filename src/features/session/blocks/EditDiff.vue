<script setup lang="ts">
// Ported from EditDiffView.swift. Unlike Poirot -- which recomputes `LineDiff.diff` itself on a
// background task, capping each side at `maxLines = 5000` first -- the diff, cap included, is
// already computed in Rust (`diff_lines`, M1-42's `DiffResult`); this component only fetches and
// renders it. Save-as-patch and copy-as-image are explicitly out of scope (this task's AC3,
// M3-10/M3-39's own job) -- only the plain-text copy button (`copyButton` in the real source,
// `.help("Copy diff text")`) is ported.
import { ref, watch } from 'vue';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import AppIcon from '@/components/AppIcon.vue';
import DiffTable from './DiffTable.vue';
import { ipc } from '@/ipc/client';
import type { DiffResult } from '@/ipc/bindings';

const props = defineProps<{ oldString: string; newString: string }>();

const result = ref<DiffResult | null>(null);
const copied = ref(false);

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
      <button
        type="button"
        title="Copy diff text"
        class="absolute right-(--spacing-sm) top-(--spacing-sm) flex h-6.5 w-6.5 items-center justify-center rounded-sm bg-(--bg-elevated)"
        @click="copy"
      >
        <AppIcon
          :name="copied ? 'checkmark' : 'doc.on.doc'"
          size="sm"
          :class="copied ? 'text-(--color-green)' : 'text-(--text-tertiary)'"
        />
      </button>
    </template>
  </div>
</template>
