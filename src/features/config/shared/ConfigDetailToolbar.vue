<script setup lang="ts">
// Ported from ConfigDetailToolbar.swift:5-97, AC3's own scope: copy name, copy content, and the
// read-only format toggle (no IPC write -- `appState.configDetailFormatted` has no Heimdall Prefs
// field anywhere, confirmed directly, the same gap M3-03/M3-04 already documented for their own
// raw/formatted toggles, so this is local state the caller owns). Open-in-editor and
// Show-in-Finder are explicitly out of scope ("come later, M3-55"); Delete is M3-44.
import { ref } from 'vue';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import AppIcon from '@/components/AppIcon.vue';

const props = defineProps<{ name: string; content: string; formatted: boolean }>();
defineEmits<{ 'update:formatted': [value: boolean] }>();

const nameCopied = ref(false);
const contentCopied = ref(false);

async function copyName() {
  await writeText(props.name);
  nameCopied.value = true;
  setTimeout(() => (nameCopied.value = false), 1500);
}

async function copyContent() {
  await writeText(props.content);
  contentCopied.value = true;
  setTimeout(() => (contentCopied.value = false), 1500);
}
</script>

<template>
  <div class="flex items-center gap-(--spacing-xs)">
    <button
      type="button"
      :title="formatted ? 'Show raw content' : 'Show rendered markdown'"
      class="flex h-7 w-7 items-center justify-center rounded-sm bg-(--bg-elevated)"
      @click="$emit('update:formatted', !formatted)"
    >
      <AppIcon
        :name="formatted ? 'doc.richtext' : 'doc.plaintext'"
        size="sm"
        :class="formatted ? 'text-(--accent)' : 'text-(--text-tertiary)'"
      />
    </button>
    <button
      type="button"
      title="Copy Name"
      class="flex h-7 w-7 items-center justify-center rounded-sm bg-(--bg-elevated)"
      @click="copyName"
    >
      <AppIcon
        :name="nameCopied ? 'checkmark' : 'doc.on.doc'"
        size="sm"
        :class="nameCopied ? 'text-(--color-green)' : 'text-(--text-tertiary)'"
      />
    </button>
    <button
      type="button"
      title="Copy Content"
      class="flex h-7 w-7 items-center justify-center rounded-sm bg-(--bg-elevated)"
      @click="copyContent"
    >
      <AppIcon
        :name="contentCopied ? 'checkmark' : 'doc.text'"
        size="sm"
        :class="contentCopied ? 'text-(--color-green)' : 'text-(--text-tertiary)'"
      />
    </button>
  </div>
</template>
