<script setup lang="ts">
// Ported from ThinkingBlockView.swift. `isExpanded` reads `autoExpandBlocks` once on mount, not
// reactively -- `ToolBlockView`/`ThinkingBlockView`/`SystemContentView` are the three real views
// documented (docs/spec/session-views.md §2) as *not* observing `allBlocksExpanded`'s live
// toggle, unlike `ToolBlockView` itself; reproducing that exact asymmetry rather than "fixing"
// it, since this task's own Inputs don't ask for the fix and consciously changing it would be a
// different task's own design decision.
//
// The hover-opacity choreography for the wrap/copy buttons (`isContentHovered`/
// `isButtonsHovered`, three opacity tiers) is not ported -- the AC asks for truncation/expand/
// copy/reduced-motion, not that exact cosmetic reveal; the buttons are simply always visible here.
import { onMounted, ref } from 'vue';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import AppIcon from '@/components/AppIcon.vue';
import { usePrefsStore } from '@/stores/prefs';

const props = defineProps<{ text: string }>();

const TRUNCATED_LINE_COUNT = 50;

const prefsStore = usePrefsStore();
const isExpanded = ref(false);
const showAll = ref(false);
const copied = ref(false);

onMounted(() => {
  isExpanded.value = prefsStore.prefs?.autoExpandBlocks ?? false;
});

const lines = () => props.text.split('\n');
const isTruncatable = () => lines().length > TRUNCATED_LINE_COUNT;
const displayedText = () => (!showAll.value && isTruncatable() ? lines().slice(0, TRUNCATED_LINE_COUNT).join('\n') : props.text);

async function toggleWrap() {
  await prefsStore.update({ wrapCodeLines: !(prefsStore.prefs?.wrapCodeLines ?? true) });
}

let copiedTimeout: ReturnType<typeof setTimeout> | undefined;
async function copy() {
  await writeText(props.text);
  copied.value = true;
  clearTimeout(copiedTimeout);
  copiedTimeout = setTimeout(() => (copied.value = false), 1500);
}
</script>

<template>
  <div class="flex flex-col">
    <button
      type="button"
      class="flex w-fit items-center gap-(--spacing-xs) rounded-full bg-(--color-purple)/8 px-(--spacing-md) py-(--spacing-xs) text-(--color-purple)"
      :title="isExpanded ? 'Collapse thinking' : 'Expand thinking'"
      @click="isExpanded = !isExpanded"
    >
      <AppIcon
        name="brain.head.profile"
        size="sm"
        :effect="isExpanded ? 'breathe' : undefined"
      />
      <span class="text-(length:--text-tiny)">Thinking</span>
      <AppIcon
        name="chevron.down"
        size="sm"
        class="transition-transform"
        :class="{ '-rotate-90': !isExpanded }"
      />
    </button>

    <div
      v-if="isExpanded"
      class="mt-(--spacing-sm) rounded-sm border border-(--color-purple)/15 bg-(--bg-code)"
    >
      <div class="relative p-(--spacing-md)">
        <pre
          class="text-(length:--text-code) text-(--text-tertiary)"
          :class="prefsStore.prefs?.wrapCodeLines ?? true ? 'whitespace-pre-wrap' : 'overflow-x-auto whitespace-pre'"
        >{{ displayedText() }}</pre>

        <div class="absolute right-(--spacing-sm) top-(--spacing-sm) flex gap-(--spacing-xs)">
          <button
            type="button"
            :title="(prefsStore.prefs?.wrapCodeLines ?? true) ? 'Disable line wrapping' : 'Enable line wrapping'"
            class="flex h-[26px] w-[26px] items-center justify-center rounded-sm bg-(--bg-elevated)"
            @click="toggleWrap"
          >
            <AppIcon
              :name="(prefsStore.prefs?.wrapCodeLines ?? true) ? 'text.word.spacing' : 'arrow.right.to.line'"
              size="sm"
              :class="(prefsStore.prefs?.wrapCodeLines ?? true) ? 'text-(--accent)' : 'text-(--text-tertiary)'"
            />
          </button>
          <button
            type="button"
            title="Copy thinking"
            class="flex h-[26px] w-[26px] items-center justify-center rounded-sm bg-(--bg-elevated)"
            @click="copy"
          >
            <AppIcon
              :name="copied ? 'checkmark' : 'doc.on.doc'"
              size="sm"
              :class="copied ? 'text-(--color-green)' : 'text-(--text-tertiary)'"
            />
          </button>
        </div>
      </div>

      <template v-if="isTruncatable()">
        <div class="border-t border-(--border)/30" />
        <button
          type="button"
          class="w-full py-(--spacing-xs) text-center text-(length:--text-tiny) text-(--color-purple)"
          :title="showAll ? 'Show less' : `Show all ${lines().length} lines`"
          @click="showAll = !showAll"
        >
          {{ showAll ? 'Show less' : `Show all ${lines().length} lines` }}
        </button>
      </template>
    </div>
  </div>
</template>
