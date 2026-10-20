<script setup lang="ts">
// A text `ContentBlock` renders through `MarkdownView` (M2-10) -- the Goal's own "Text ... blocks
// render through MarkdownView". When the block's own `system` field is populated (Rust found
// embedded system-reminder/command-message tags in the text), the parsed segments render through
// `SystemContent` right below it, matching how the Swift reference app pairs a text `Markdown` view with an
// optional `SystemContentView` for the same content.
import MarkdownView from '@/lib/markdown/MarkdownView.vue';
import type { SystemSegment } from '@/ipc/bindings';
import SystemContent from './SystemContent.vue';

defineProps<{ text: string; system?: SystemSegment[] | null }>();
</script>

<template>
  <div>
    <MarkdownView :content="text" />
    <SystemContent
      v-if="system && system.length > 0"
      :segments="system"
    />
  </div>
</template>
