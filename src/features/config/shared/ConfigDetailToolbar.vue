<script setup lang="ts">
// Ported from ConfigDetailToolbar.swift:5-97, AC3's own scope: copy name, copy content, and the
// read-only format toggle (no IPC write -- `appState.configDetailFormatted` has no Heimdall Prefs
// field anywhere, confirmed directly, the same gap M3-03/M3-04 already documented for their own
// raw/formatted toggles, so this is local state the caller owns).
//
// M3-55 adds Open in <editor> and Show in <file manager> (ConfigDetailToolbar.swift:41-95), shown when the
// caller says which item this is (`configRef`; the commands take that id, never a path, ADR 0010). The
// editor button is hidden when the saved editor does not exist on this OS. M3-50 added the opt-in Delete
// (M3-44's command).
//
// Close (`ConfigToolbarClose`, ConfigDetailToolbar.swift:136-148) is added here as a necessary
// ripple from M3-12: every consumer of ConfigItemDetail.vue (Commands/Skills/OutputStyles) needs
// a way back to the list, and this is the one component Swift's own spec already names as owning
// it (§1.7) -- not something to duplicate three times in each page instead.
import { ref } from 'vue';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import AppIcon from '@/components/AppIcon.vue';
import { useLauncher } from '@/composables/useLauncher';
import type { ConfigRef } from '@/ipc/bindings';

// `deletable` opts in to the Delete button (ConfigToolbarDelete in Swift, shown only for items a delete
// command can address). Only ConfigItemDetail uses this toolbar (Plans have their own, PlanDetail.vue),
// so the opt-in is for a caller that has no such item to delete. `configRef` likewise opts in to the
// editor and file-manager buttons.
const props = defineProps<{ name: string; content: string; formatted: boolean; deletable?: boolean; configRef?: ConfigRef | null }>();
const launcher = useLauncher();
defineEmits<{ 'update:formatted': [value: boolean]; close: []; delete: [] }>();

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

function openInEditor() {
  if (props.configRef) void launcher.openInEditor({ kind: 'configItem', configRef: props.configRef });
}

function reveal() {
  if (props.configRef) void launcher.reveal({ kind: 'configItem', configRef: props.configRef });
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
    <button
      v-if="configRef && launcher.editorName.value"
      type="button"
      :title="`Open in ${launcher.editorName.value}`"
      :aria-label="`Open in ${launcher.editorName.value}`"
      class="flex h-7 w-7 items-center justify-center rounded-sm bg-(--bg-elevated)"
      @click="openInEditor"
    >
      <AppIcon
        name="arrow.up.forward.square"
        size="sm"
        class="text-(--text-tertiary)"
      />
    </button>
    <button
      v-if="configRef"
      type="button"
      :title="`Show in ${launcher.fileManager.value}`"
      :aria-label="`Show in ${launcher.fileManager.value}`"
      class="flex h-7 w-7 items-center justify-center rounded-sm bg-(--bg-elevated)"
      @click="reveal"
    >
      <AppIcon
        name="folder"
        size="sm"
        class="text-(--text-tertiary)"
      />
    </button>
    <button
      v-if="deletable"
      type="button"
      title="Delete"
      aria-label="Delete"
      class="flex h-7 w-7 items-center justify-center rounded-sm bg-(--bg-elevated)"
      @click="$emit('delete')"
    >
      <AppIcon
        name="trash"
        size="sm"
        class="text-(--text-tertiary)"
      />
    </button>
    <button
      type="button"
      title="Close"
      class="flex h-7 w-7 items-center justify-center rounded-sm bg-(--bg-elevated)"
      @click="$emit('close')"
    >
      <AppIcon
        name="xmark"
        size="sm"
        class="text-(--text-tertiary)"
      />
    </button>
  </div>
</template>
