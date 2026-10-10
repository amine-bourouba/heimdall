<script setup lang="ts">
// Ported from SystemContentView.swift/`SystemBlockContentView`. Rust already does the nested-tag
// parsing `SystemContentParser`/`JSONBeautifier` did in Swift -- confirmed directly against the
// real contract: `ContentBlock`'s `kind: "text"` carries `system: SystemSegment[] | null`, and
// `SystemSegment.children` is documented in `bindings.ts` itself as "one level, as
// SystemBlockContentView shows" -- the exact same one-level structure this view renders. So this
// component only renders the already-parsed segments; no parser is ported here at all.
// `isExpanded` has the same "read autoExpandBlocks once on mount, not reactively" asymmetry as
// `ThinkingBlock.vue` -- see its own header comment for why.
import { onMounted, ref } from 'vue';
import AppIcon from '@/components/AppIcon.vue';
import MarkdownView from '@/lib/markdown/MarkdownView.vue';
import type { SystemSegment } from '@/ipc/bindings';
import { usePrefsStore } from '@/stores/prefs';

defineProps<{ segments: SystemSegment[] }>();

const prefsStore = usePrefsStore();
const isExpanded = ref(false);

onMounted(() => {
  isExpanded.value = prefsStore.prefs?.autoExpandBlocks ?? false;
});
</script>

<template>
  <div class="mt-(--spacing-md) border-t border-(--border)/30 pt-(--spacing-sm)">
    <button
      type="button"
      class="flex w-fit items-center gap-(--spacing-xs) rounded-full border border-(--border-subtle) bg-(--bg-code)/50 px-(--spacing-md) py-(--spacing-xs) text-(--text-tertiary)"
      :title="isExpanded ? 'Collapse system context' : 'Expand system context'"
      @click="isExpanded = !isExpanded"
    >
      <AppIcon
        name="info.circle"
        size="sm"
      />
      <span class="text-(length:--text-tiny)">System context ({{ segments.length }})</span>
      <AppIcon
        name="chevron.down"
        size="sm"
        class="transition-transform"
        :class="{ '-rotate-90': !isExpanded }"
      />
    </button>

    <div
      v-if="isExpanded"
      class="mt-(--spacing-sm) flex flex-col gap-(--spacing-sm) rounded-sm border border-(--border-subtle) bg-(--bg-code) p-(--spacing-md)"
    >
      <div
        v-for="(segment, index) in segments"
        :key="index"
        class="flex flex-col gap-(--spacing-xs)"
      >
        <span
          v-if="segment.tag"
          class="w-fit rounded-xs bg-(--accent-dim) px-(--spacing-xs) text-(length:--text-code-micro) text-(--accent)"
        >{{ segment.tag }}</span>

        <template v-if="segment.children && segment.children.length > 0">
          <div
            v-for="(child, childIndex) in segment.children"
            :key="childIndex"
            class="flex flex-col gap-(--spacing-xxs)"
          >
            <span
              v-if="child.tag"
              class="text-(length:--text-code-nano) text-(--text-tertiary)/70"
            >{{ child.tag }}</span>
            <MarkdownView
              v-if="prefsStore.prefs?.parseMarkdownInResults ?? true"
              :content="child.text"
            />
            <p
              v-else
              class="whitespace-pre-wrap text-(length:--text-tiny) text-(--text-tertiary)"
            >
              {{ child.text }}
            </p>
          </div>
        </template>
        <template v-else>
          <MarkdownView
            v-if="prefsStore.prefs?.parseMarkdownInResults ?? true"
            :content="segment.text"
          />
          <p
            v-else
            class="whitespace-pre-wrap text-(length:--text-tiny) text-(--text-tertiary)"
          >
            {{ segment.text }}
          </p>
        </template>
      </div>
    </div>
  </div>
</template>
