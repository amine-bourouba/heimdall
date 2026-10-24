<script setup lang="ts">
// Ported from PlanDetailView.swift. The raw/formatted toggle (`appState.configDetailFormatted`)
// is not ported: not in this task's own AC ("detail renders through MarkdownView", singular),
// and no Heimdall Prefs field for it exists yet.
import { ref } from 'vue';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import type { ConfigItemDetail } from '@/ipc/bindings';
import AppIcon from '@/components/AppIcon.vue';
import MarkdownView from '@/lib/markdown/MarkdownView.vue';

defineProps<{ plan: ConfigItemDetail }>();
const emit = defineEmits<{ back: [] }>();

const copied = ref(false);

async function copy(body: string) {
  await writeText(body);
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
        aria-label="Back to plans"
        title="Back to plans"
        @click="emit('back')"
      >
        <AppIcon name="chevron.left" />
      </button>
      <div class="flex h-9 w-9 items-center justify-center rounded-md bg-(--accent-dim)">
        <AppIcon
          name="list.bullet.clipboard.fill"
          class="text-(--accent)"
        />
      </div>
      <div class="flex flex-col gap-(--spacing-xxs)">
        <h1 class="text-(--text-primary) text-(length:--text-heading)">
          {{ plan.name }}
        </h1>
        <span class="self-start rounded-sm bg-(--bg-elevated) px-(--spacing-sm) py-[3px] text-(--text-tertiary) text-(length:--text-tiny)">{{ plan.displayPath }}</span>
      </div>
      <div class="flex-1" />
      <button
        type="button"
        class="shrink-0 text-(--text-tertiary)"
        title="Copy Content"
        aria-label="Copy content"
        @click="copy(plan.body)"
      >
        <AppIcon :name="copied ? 'checkmark.circle.fill' : 'doc.on.doc'" />
      </button>
    </div>

    <div
      v-if="plan.body === ''"
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
      <MarkdownView :content="plan.body" />
    </div>
  </div>
</template>
