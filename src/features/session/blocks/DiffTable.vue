<script setup lang="ts">
// Ported from EditDiffView.swift's `diffLineRow`/`gutterWidth`/`indicator(for:)`/`textColor(for:)`/
// `backgroundColor(for:)`. Pure presentation, no fetching and no copy button, so the file-history
// view (a later task, reading `diffFileVersions` instead of `diffLines`) can reuse it directly
// with its own already-computed `DiffResult` -- this task's own AC1.
import { computed } from 'vue';
import type { DiffLine } from '@/ipc/bindings';

const props = defineProps<{ lines: DiffLine[]; maxLineNumber: number }>();

// EditDiffView.swift:38-45 -- `max(digits(maxLineNumber), 2) * 8 + 6`.
const gutterWidth = computed(() => `${Math.max(String(props.maxLineNumber).length, 2) * 8 + 6}px`);

function indicator(kind: DiffLine['kind']): string {
  return kind === 'add' ? '+' : kind === 'remove' ? '-' : ' ';
}
</script>

<template>
  <div class="overflow-x-auto">
    <div
      v-for="(line, i) in lines"
      :key="i"
      class="flex px-(--spacing-xs) py-px"
      :class="{ 'bg-(--diff-add-bg)': line.kind === 'add', 'bg-(--diff-remove-bg)': line.kind === 'remove' }"
    >
      <span
        class="shrink-0 text-right text-(length:--text-code-small) text-(--text-tertiary)/60"
        :style="{ width: gutterWidth }"
      >{{ line.oldLine ?? '' }}</span>
      <span
        class="shrink-0 text-right text-(length:--text-code-small) text-(--text-tertiary)/60"
        :style="{ width: gutterWidth }"
      >{{ line.newLine ?? '' }}</span>
      <span
        class="w-5 shrink-0 text-center text-(length:--text-code)"
        :class="line.kind === 'add' ? 'text-(--diff-add-text)' : line.kind === 'remove' ? 'text-(--diff-remove-text)' : 'text-(--text-tertiary)'"
      >{{ indicator(line.kind) }}</span>
      <span
        class="whitespace-pre text-(length:--text-code)"
        :class="line.kind === 'add' ? 'text-(--diff-add-text)' : line.kind === 'remove' ? 'text-(--diff-remove-text)' : 'text-(--text-secondary)'"
      >{{ line.text }}</span>
    </div>
  </div>
</template>
