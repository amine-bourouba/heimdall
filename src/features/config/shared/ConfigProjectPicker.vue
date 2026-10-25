<script setup lang="ts">
// Ported from ConfigProjectPicker.swift:3-76, plus this task's own AC1 addition ("offers known
// projects") that has no Swift equivalent at all -- Swift's own picker is only an icon, a
// name/placeholder, an "x", and "Browse...". A separate one-shot "jump to a known project" select
// is added alongside those four, rather than folding "known project" and "Browse..." into one
// native <select> (a single element can't represent "picking this option opens a dialog instead
// of persisting a selection" without real cleverness for no real benefit here).
//
// store.ts's own header comment covers why "x" only clears this store's in-memory
// `selectedProject`, not the underlying `Prefs.configProjectPath` -- there is no command that
// could clear that even if this component wanted to.
import { computed, onMounted, ref } from 'vue';
import AppIcon from '@/components/AppIcon.vue';
import { useConfigStore } from '../store';

const configStore = useConfigStore();

onMounted(() => void configStore.load());

const jumpValue = ref('');

const currentLabel = computed(() => {
  const project = configStore.selectedProject;
  if (!project) return 'No project selected';
  if (project.kind === 'pickedFolder') return configStore.pickedFolderName() ?? 'Picked folder';
  return configStore.projects.find((p) => p.id === project.projectId)?.name ?? project.projectId;
});

function selectKnownProject() {
  if (jumpValue.value) configStore.selectProject({ kind: 'project', projectId: jumpValue.value });
  jumpValue.value = '';
}
</script>

<template>
  <div class="flex items-center gap-(--spacing-sm)">
    <div class="flex items-center gap-(--spacing-xs)">
      <AppIcon
        name="folder.fill"
        size="sm"
        :class="configStore.selectedProject ? 'text-(--color-green)' : 'text-(--text-tertiary)'"
      />
      <span class="max-w-32 truncate text-(length:--text-tiny) text-(--text-secondary)">{{ currentLabel }}</span>
      <button
        v-if="configStore.selectedProject"
        type="button"
        title="Clear project"
        @click="configStore.selectProject(null)"
      >
        <AppIcon
          name="xmark.circle.fill"
          size="sm"
          class="text-(--text-tertiary)"
        />
      </button>
    </div>

    <select
      v-model="jumpValue"
      aria-label="Jump to a known project"
      class="rounded-sm border border-(--border) bg-(--bg-elevated) px-(--spacing-sm) py-(--spacing-xs) text-(length:--text-tiny)"
      @change="selectKnownProject"
    >
      <option value="">
        Known projects…
      </option>
      <option
        v-for="project in configStore.projects"
        :key="project.id"
        :value="project.id"
      >
        {{ project.name }}
      </option>
    </select>

    <button
      type="button"
      title="Browse for a folder"
      class="rounded-sm border border-(--border) px-(--spacing-sm) py-(--spacing-xs) text-(length:--text-tiny) text-(--text-secondary)"
      @click="configStore.browseForProject()"
    >
      Browse…
    </button>
  </div>
</template>
