<script setup lang="ts">
// Ported from FileHistoryView.swift (M3-07). Self-contained like SessionFacetsCard.vue/
// SessionTodos.vue: fetches its own data from `projectId`/`sessionId` rather than through
// session/store.ts.
//
// Diffs are computed server-side, not re-implemented client-side the way Swift's own
// `EditDiffView(oldString:, newString:)` does: any version with a predecessor goes through
// `diffFileVersions`; a file's first version has no predecessor for `diff_file_versions` to
// diff against (it always takes two real version ids -- there is no "empty" sentinel), so that
// one case goes through `diffLines('', content)` instead -- the same server-side diff engine,
// just fed an empty "before" string, matching Swift's own "first version -- show full content as
// added" special case without reinventing the diff algorithm here.
import { computed, ref, watch } from 'vue';
import { ipc } from '@/ipc/client';
import type { DiffResult, FileHistoryFile } from '@/ipc/bindings';
import AppIcon from '@/components/AppIcon.vue';
import DiffTable from './blocks/DiffTable.vue';
import { useSessionStore } from './store';
import { formatRelativeTime } from './format';

const props = defineProps<{ projectId: string; sessionId: string }>();

const sessionStore = useSessionStore();

const files = ref<FileHistoryFile[]>([]);
const selectedFile = ref<FileHistoryFile | null>(null);
const selectedVersionIndex = ref(0);
const diff = ref<DiffResult | null>(null);
const isLoading = ref(true);

// Same staleness-guard shape as SessionFacetsCard.vue/SessionTodos.vue's own `generation`: a
// fetch for a since-abandoned session must not overwrite state for whichever session is open
// now. A second counter guards the (independently racy) per-version diff fetch the same way.
let generation = 0;
let diffGeneration = 0;

async function load(projectId: string, sessionId: string) {
  const thisGeneration = ++generation;
  isLoading.value = true;
  const result = await ipc.getFileHistory(projectId, sessionId);
  if (thisGeneration !== generation) return;
  files.value = result.files;
  isLoading.value = false;
  selectFile(files.value[0] ?? null);
}

watch(() => [props.projectId, props.sessionId] as const, ([p, s]) => void load(p, s), { immediate: true });

function selectFile(file: FileHistoryFile | null) {
  selectedFile.value = file;
  // FileHistoryView.swift:347-350/191: auto-selects the latest version, not the first.
  void selectVersion(file ? file.versions.length - 1 : 0);
}

async function selectVersion(index: number) {
  selectedVersionIndex.value = index;
  const file = selectedFile.value;
  if (!file || !file.versions[index]) {
    diff.value = null;
    return;
  }
  const thisDiffGeneration = ++diffGeneration;
  const version = file.versions[index];
  let result: DiffResult;
  if (index === 0) {
    const { content } = await ipc.getFileVersion(props.sessionId, version.versionId);
    result = await ipc.diffLines('', content);
  } else {
    result = await ipc.diffFileVersions(props.sessionId, file.versions[index - 1].versionId, version.versionId);
  }
  if (thisDiffGeneration !== diffGeneration) return;
  diff.value = result;
}

function fileNameOnly(fileName: string): string {
  return fileName.split('/').pop() ?? fileName;
}

function directoryPath(fileName: string): string {
  return fileName.split('/').slice(0, -1).join('/');
}

// FileHistoryView.swift:384-398's own extension switch, trimmed to the icons icon-map.ts
// actually has a Lucide mapping for -- anything else (py/rs/go/swift/...) falls back to the
// same generic `doc.text` every unmapped case already gets, rather than adding new icon-map.ts
// entries for a single file-list row.
const FILE_ICONS: Record<string, string> = {
  js: 'curlybraces',
  ts: 'curlybraces',
  jsx: 'curlybraces',
  tsx: 'curlybraces',
  json: 'curlybraces',
  yml: 'list.bullet.rectangle',
  yaml: 'list.bullet.rectangle',
  toml: 'list.bullet.rectangle',
  html: 'globe',
  css: 'globe',
  sh: 'terminal',
  zsh: 'terminal',
  bash: 'terminal',
};
function fileIcon(fileName: string): string {
  const ext = fileName.split('.').pop()?.toLowerCase() ?? '';
  return FILE_ICONS[ext] ?? 'doc.text';
}

const fileCountLabel = computed(() => `${files.value.length} ${files.value.length === 1 ? 'file' : 'files'} changed`);
</script>

<template>
  <div class="flex h-full flex-col">
    <div class="flex shrink-0 items-center gap-(--spacing-md) border-b border-(--border)/30 px-(--spacing-xxxl) py-(--spacing-lg)">
      <div class="flex h-9 w-9 shrink-0 items-center justify-center rounded-md bg-(--accent)/15">
        <AppIcon
          name="clock.arrow.2.circlepath"
          size="sm"
          class="text-(--accent)"
        />
      </div>
      <div class="flex flex-col">
        <h2 class="text-(length:--text-large) font-semibold">
          File History
        </h2>
        <p class="text-(length:--text-small) text-(--text-secondary)">
          {{ fileCountLabel }}
        </p>
      </div>
      <div class="flex-1" />
      <button
        type="button"
        title="Close"
        @click="sessionStore.isShowingFileHistory = false"
      >
        <AppIcon
          name="xmark.circle.fill"
          size="lg"
          class="text-(--text-tertiary)"
        />
      </button>
    </div>

    <p
      v-if="isLoading"
      class="flex flex-1 items-center justify-center text-(--text-tertiary)"
    >
      Loading file history…
    </p>

    <div
      v-else-if="files.length === 0"
      class="flex flex-1 flex-col items-center justify-center gap-(--spacing-sm) text-center"
    >
      <AppIcon
        name="doc.text.magnifyingglass"
        size="xl"
        class="text-(--text-tertiary)/50"
      />
      <p class="font-semibold text-(--text-secondary)">
        No File History
      </p>
      <p class="text-(length:--text-small) text-(--text-tertiary)">
        No versioned file snapshots were found for this session.
      </p>
    </div>

    <div
      v-else
      class="flex flex-1 overflow-hidden"
    >
      <div class="w-64 shrink-0 overflow-y-auto border-r border-(--border)/30 p-(--spacing-sm)">
        <button
          v-for="file in files"
          :key="file.fileName"
          type="button"
          class="flex w-full items-center gap-(--spacing-sm) rounded-sm px-(--spacing-md) py-(--spacing-sm) text-left"
          :class="selectedFile === file ? 'bg-(--accent-dim)' : ''"
          :title="file.fileName"
          @click="selectFile(file)"
        >
          <AppIcon
            :name="fileIcon(file.fileName)"
            size="sm"
            :class="selectedFile === file ? 'text-(--accent)' : 'text-(--text-secondary)'"
          />
          <div class="min-w-0 flex-1">
            <p
              class="truncate text-(length:--text-caption)"
              :class="selectedFile === file ? 'text-(--text-primary)' : 'text-(--text-secondary)'"
            >
              {{ fileNameOnly(file.fileName) }}
            </p>
            <p
              v-if="file.fileName.includes('/')"
              class="truncate text-(length:--text-tiny) text-(--text-tertiary)"
            >
              {{ directoryPath(file.fileName) }}
            </p>
          </div>
          <span
            class="shrink-0 rounded-full px-(--spacing-sm) py-px text-(length:--text-tiny)"
            :class="selectedFile === file ? 'bg-(--accent)/15 text-(--accent)' : 'bg-(--bg-elevated) text-(--text-tertiary)'"
          >{{ file.versions.length }}</span>
        </button>
      </div>

      <div
        v-if="selectedFile"
        class="flex flex-1 flex-col overflow-hidden"
      >
        <div class="shrink-0 overflow-x-auto border-b border-(--border)/30 bg-(--bg-card) px-(--spacing-lg) py-(--spacing-md)">
          <div class="flex items-center gap-(--spacing-sm)">
            <button
              v-for="(version, index) in selectedFile.versions"
              :key="version.versionId"
              type="button"
              class="flex shrink-0 items-center gap-(--spacing-xs) rounded-sm border px-(--spacing-md) py-(--spacing-xs) text-(length:--text-tiny)"
              :class="selectedVersionIndex === index ? 'border-(--accent)/30 bg-(--accent-dim) text-(--accent)' : 'border-(--border) text-(--text-secondary)'"
              @click="selectVersion(index)"
            >
              <span class="font-semibold">v{{ version.version }}</span>
              <span class="text-(--text-tertiary)">{{ formatRelativeTime(version.backupTime) }}</span>
            </button>
          </div>
        </div>
        <div class="flex-1 overflow-y-auto">
          <DiffTable
            v-if="diff"
            :lines="diff.lines"
            :max-line-number="diff.maxLineNumber"
          />
        </div>
      </div>
      <div
        v-else
        class="flex flex-1 items-center justify-center text-(--text-tertiary)"
      >
        Select a file to view changes
      </div>
    </div>
  </div>
</template>
