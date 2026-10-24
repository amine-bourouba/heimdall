<script setup lang="ts">
// Ported from MemoryDetailView.swift. Same scope note as PlanDetail.vue: no raw/formatted
// toggle (no Heimdall Prefs field for it), MarkdownView renders the content directly.
import { ref } from 'vue';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import type { MemoryFileDetail } from '@/ipc/bindings';
import AppIcon from '@/components/AppIcon.vue';
import MarkdownView from '@/lib/markdown/MarkdownView.vue';

defineProps<{ memory: MemoryFileDetail }>();
const emit = defineEmits<{ back: [] }>();

const copied = ref(false);

async function copy(content: string) {
  await writeText(content);
  copied.value = true;
  setTimeout(() => (copied.value = false), 2000);
}
</script>

<template>
  <div class="flex h-full flex-col">
    <div class="flex items-center gap-(--spacing-md) border-b px-(--spacing-xxxl) py-(--spacing-xl)">
      <button
        type="button"
        class="text-(--text-tertiary)"
        aria-label="Back to memory"
        title="Back to memory"
        @click="emit('back')"
      >
        <AppIcon name="chevron.left" />
      </button>
      <div class="flex h-9 w-9 items-center justify-center rounded-md bg-(--color-purple)/15">
        <AppIcon
          :name="memory.isMain ? 'brain.head.profile.fill' : 'doc.text.fill'"
          class="text-(--color-purple)"
        />
      </div>
      <div class="flex flex-col gap-(--spacing-xxs)">
        <h1 class="text-(--text-primary) text-(length:--text-heading)">
          {{ memory.name }}
        </h1>
        <div class="flex items-center gap-(--spacing-sm)">
          <span
            v-if="memory.isMain"
            class="rounded-sm bg-(--accent-dim) px-(--spacing-sm) py-[2px] text-(--accent) text-(length:--text-tiny)"
          >Entrypoint</span>
          <span class="rounded-sm bg-(--bg-elevated) px-(--spacing-sm) py-[3px] text-(--text-tertiary) text-(length:--text-tiny)">{{ memory.filename }}</span>
        </div>
      </div>
      <div class="flex-1" />
      <button
        type="button"
        class="shrink-0 text-(--text-tertiary)"
        title="Copy Content"
        aria-label="Copy content"
        @click="copy(memory.content)"
      >
        <AppIcon :name="copied ? 'checkmark.circle.fill' : 'doc.on.doc'" />
      </button>
    </div>

    <div
      v-if="memory.content === ''"
      class="flex flex-1 flex-col items-center justify-center gap-(--spacing-md) text-(--text-tertiary)"
    >
      <AppIcon
        name="doc.text"
        size="lg"
      />
      <p>No content</p>
    </div>
    <div
      v-else
      class="flex-1 overflow-y-auto px-(--spacing-xxxl) py-(--spacing-xl)"
    >
      <MarkdownView :content="memory.content" />
    </div>
  </div>
</template>
