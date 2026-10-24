<script setup lang="ts">
// Ported from HistoryListView.swift's own private `HistoryCard` (lines 421-508), minus delete
// (this task's own AC: "Delete actions are not present yet", M3-51) and the project-filter
// picker (not in this task's Outputs). The copy-tapped checkmark swap is kept (`:421-508`'s own
// `copyTapped` 2-second flip) since it is cheap, local state with no store/IPC of its own.
import { computed, ref } from 'vue';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import type { HistoryEntry } from '@/ipc/bindings';
import { fuzzyMatch } from '@/lib/fuzzyMatch';
import { formatRelativeTime } from '@/features/session/format';
import { historySnippet, projectName } from './store';
import AppIcon from '@/components/AppIcon.vue';
import CardChrome from '@/components/CardChrome.vue';

const props = defineProps<{ entry: HistoryEntry; query?: string }>();

const copied = ref(false);

const snippet = computed(() => historySnippet(props.entry.display));
const project = computed(() => projectName(props.entry));
const pastedCount = computed(() => Object.keys(props.entry.pastedContents).length);

// Same slice-based highlight rendering as SearchResultRow.vue, built from fuzzyMatch's own
// per-character indices rather than a contiguous range (§2.4-style fuzzy highlighting): each
// matched character gets its own `<mark>`, since a fuzzy match is a scattered subsequence, not
// one contiguous run.
function segments(text: string, query: string | undefined) {
  const trimmed = query?.trim() ?? '';
  if (trimmed === '') return [{ text, highlighted: false }];
  const indices = new Set(fuzzyMatch(text, trimmed)?.indices ?? []);
  if (indices.size === 0) return [{ text, highlighted: false }];
  const out: { text: string; highlighted: boolean }[] = [];
  let run = '';
  let runHighlighted = false;
  for (let i = 0; i < text.length; i++) {
    const highlighted = indices.has(i);
    if (run !== '' && highlighted !== runHighlighted) {
      out.push({ text: run, highlighted: runHighlighted });
      run = '';
    }
    run += text[i];
    runHighlighted = highlighted;
  }
  if (run !== '') out.push({ text: run, highlighted: runHighlighted });
  return out;
}

const snippetSegments = computed(() => segments(snippet.value, props.query));
const projectSegments = computed(() => segments(project.value, props.query));

async function copy() {
  await writeText(props.entry.display);
  copied.value = true;
  setTimeout(() => (copied.value = false), 2000);
}
</script>

<template>
  <CardChrome>
    <div class="flex flex-col gap-(--spacing-sm) p-(--spacing-lg)">
      <div class="flex items-start gap-(--spacing-sm)">
        <p class="line-clamp-3 flex-1 text-(--text-primary) text-(length:--text-body-medium)">
          <template
            v-for="(segment, index) in snippetSegments"
            :key="index"
          >
            <mark
              v-if="segment.highlighted"
              class="rounded-xs bg-(--accent-dim) text-(--accent)"
            >{{ segment.text }}</mark>
            <template v-else>
              {{ segment.text }}
            </template>
          </template>
        </p>

        <button
          type="button"
          class="shrink-0 text-(--text-tertiary)"
          title="Copy Prompt"
          aria-label="Copy prompt"
          @click="copy"
        >
          <AppIcon :name="copied ? 'checkmark.circle.fill' : 'doc.on.doc'" />
        </button>
      </div>

      <div class="flex items-center gap-(--spacing-sm)">
        <span class="flex items-center gap-(--spacing-xxs) rounded-sm bg-(--bg-elevated) px-(--spacing-sm) py-[3px] text-(--text-tertiary) text-(length:--text-tiny)">
          <AppIcon
            name="shippingbox"
            size="sm"
          />
          <template
            v-for="(segment, index) in projectSegments"
            :key="index"
          >
            <mark
              v-if="segment.highlighted"
              class="rounded-xs bg-(--accent-dim) text-(--accent)"
            >{{ segment.text }}</mark>
            <template v-else>{{ segment.text }}</template>
          </template>
        </span>

        <span
          v-if="pastedCount > 0"
          class="rounded-sm bg-(--bg-elevated) px-(--spacing-sm) py-[3px] text-(--text-tertiary) text-(length:--text-tiny)"
        >{{ pastedCount }} pasted</span>

        <div class="flex-1" />

        <span class="shrink-0 text-(--text-tertiary) text-(length:--text-tiny)">{{ formatRelativeTime(entry.timestamp) }}</span>
      </div>
    </div>
  </CardChrome>
</template>
