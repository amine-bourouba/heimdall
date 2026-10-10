<script setup lang="ts">
// Ported from ToolBlockView.swift. Heimdall's real `ToolUse` has no `isBash`/`isEdit` computed
// flags or a provider abstraction (ADR 0008 removes the multi-provider layer `toolIcon`/
// `toolDisplayName` came from) -- `TOOL_DEFINITIONS` below is `ClaudeCodeProvider.swift`'s own
// 7-entry table, ported directly (same fallback: the raw tool name + a generic "wrench" icon for
// anything else, e.g. MCP tool names, WebFetch/WebSearch, TodoWrite).
//
// `tool.edit` (Heimdall's real contract) replaces `tool.hasDiffData`/`oldString`/`newString` --
// when present, this renders a placeholder instead of a real diff: `EditDiffView` is a separate,
// substantial component M2-19 ("Edit diff view") owns as its own dedicated task, not in this
// task's own Inputs.
//
// `isExpanded` reads `autoExpandBlocks` once on mount, same as `ThinkingBlock`/`SystemContent` --
// but, confirmed directly in the real source, `ToolBlockView` is also the *one* block that
// really does observe `appState.allBlocksExpanded`'s live toggle (`.onChange`); that preference
// has no Heimdall store yet (M2-14 deliberately left session-detail-only UI state like it
// unported), so that live-reactivity half is not wired in here -- flagged in Follow-ups.
import { onMounted, ref } from 'vue';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import AppIcon from '@/components/AppIcon.vue';
import MarkdownView from '@/lib/markdown/MarkdownView.vue';
import type { ToolResult, ToolUse } from '@/ipc/bindings';
import { usePrefsStore } from '@/stores/prefs';

const props = defineProps<{ tool: ToolUse; result?: ToolResult | null }>();

const TRUNCATED_LINE_COUNT = 80;

const TOOL_DEFINITIONS: Record<string, { displayName: string; icon: string }> = {
  Read: { displayName: 'Read', icon: 'doc.text' },
  Write: { displayName: 'Write', icon: 'doc.text.fill' },
  Edit: { displayName: 'Edit', icon: 'pencil' },
  Bash: { displayName: 'Bash', icon: 'terminal' },
  Glob: { displayName: 'Glob', icon: 'magnifyingglass' },
  Grep: { displayName: 'Grep', icon: 'text.magnifyingglass' },
  Task: { displayName: 'Task', icon: 'checklist' },
};

const prefsStore = usePrefsStore();
const isExpanded = ref(false);
const showAllLines = ref(false);
// Poirot's own default really is raw/plain text, even for non-bash tools -- `isMarkdownContent`
// alone does not turn markdown rendering on, `!showRawContent` has to be true too.
const showRawContent = ref(true);
const copied = ref(false);
const commandCopied = ref(false);
const markdownCopied = ref(false);

onMounted(() => {
  isExpanded.value = prefsStore.prefs?.autoExpandBlocks ?? false;
});

const isBash = () => props.tool.name === 'Bash';
const isError = () => props.result?.isError ?? false;
const toolIcon = () => TOOL_DEFINITIONS[props.tool.name]?.icon ?? 'wrench';
const toolDisplayName = () => TOOL_DEFINITIONS[props.tool.name]?.displayName ?? props.tool.name;
const isMarkdownContent = () => !isBash();
const content = () => props.result?.content ?? '';
const contentLines = () => content().split('\n');
const isTruncatable = () => contentLines().length > TRUNCATED_LINE_COUNT;
const displayedContent = () => (!showAllLines.value && isTruncatable() ? contentLines().slice(0, TRUNCATED_LINE_COUNT).join('\n') : content());

// Ported from SessionExporter.toolBlockToMarkdown. `tool.input` (a Swift `[String: String]`) has
// no Heimdall equivalent -- `inputPretty` (already pretty-printed JSON, Rust's own job) is used
// for the generic case instead of iterating individual input fields.
function toolBlockToMarkdown(): string {
  const lines: string[] = [];
  if (isBash() && props.tool.command) {
    lines.push('### Tool: Bash', '```bash', props.tool.command, '```');
  } else if (props.tool.name === 'Edit' && props.tool.edit) {
    lines.push(`### Tool: Edit — \`${props.tool.filePath ?? ''}\``, '```diff');
    for (const line of props.tool.edit.oldString.split('\n')) lines.push(`- ${line}`);
    for (const line of props.tool.edit.newString.split('\n')) lines.push(`+ ${line}`);
    lines.push('```');
  } else {
    lines.push(`### Tool: ${props.tool.name}`);
    if (props.tool.filePath) lines.push(`\`${props.tool.filePath}\``);
    if (props.tool.inputPretty) lines.push('```json', props.tool.inputPretty, '```');
  }

  if (props.result) {
    const label = props.result.isError ? 'Error' : 'Output';
    const resultContent = props.result.content.length > 2000 ? `${props.result.content.slice(0, 2000)}\n... (truncated)` : props.result.content;
    lines.push('', `**${label}:**`, '```', resultContent, '```');
  }

  return lines.join('\n');
}

async function toggleWrap() {
  await prefsStore.update({ wrapCodeLines: !(prefsStore.prefs?.wrapCodeLines ?? true) });
}

async function copyContent() {
  await writeText(content());
  copied.value = true;
  setTimeout(() => (copied.value = false), 1500);
}

async function copyCommand() {
  if (!props.tool.command) return;
  await writeText(props.tool.command);
  commandCopied.value = true;
  setTimeout(() => (commandCopied.value = false), 1500);
}

async function copyAsMarkdown() {
  await writeText(toolBlockToMarkdown());
  markdownCopied.value = true;
  setTimeout(() => (markdownCopied.value = false), 1500);
}
</script>

<template>
  <div
    class="overflow-hidden rounded-md border"
    :class="isError() ? 'border-(--color-red)/20' : 'border-(--border)'"
  >
    <button
      type="button"
      class="flex w-full items-center gap-(--spacing-sm) bg-(--bg-elevated) px-(--spacing-lg) py-(--spacing-sm) text-left"
      :title="isExpanded ? 'Collapse' : 'Expand'"
      @click="isExpanded = !isExpanded"
    >
      <AppIcon
        :name="toolIcon()"
        size="sm"
        class="text-(--text-secondary)"
      />
      <span class="shrink-0 text-(length:--text-small) font-bold text-(--text-secondary)">{{ toolDisplayName() }}</span>
      <span
        v-if="isBash() && tool.command"
        class="truncate text-(length:--text-code-small) text-(--text-secondary)"
      >$ {{ tool.command }}</span>
      <span
        v-else-if="tool.filePath"
        class="truncate text-(length:--text-code-small) text-(--text-tertiary)"
      >{{ tool.filePath }}</span>

      <div class="flex-1" />

      <span
        class="rounded-xs px-(--spacing-xs) py-[1px] text-(length:--text-micro) font-semibold uppercase"
        :class="isError() ? 'bg-(--color-red)/10 text-(--color-red)' : 'bg-(--color-green)/10 text-(--color-green)'"
      >{{ isBash() ? (isError() ? 'Exit 1' : 'Exit 0') : isError() ? 'Error' : 'Done' }}</span>

      <AppIcon
        name="chevron.right"
        size="sm"
        class="text-(--text-tertiary) transition-transform"
        :class="{ 'rotate-90': isExpanded }"
      />
    </button>

    <template v-if="isExpanded">
      <div class="border-t border-(--border)/30" />

      <div v-if="tool.edit">
        <p class="p-(--spacing-lg) text-(length:--text-code) text-(--text-tertiary)">
          Diff view not built yet (M2-19).
        </p>
      </div>
      <div
        v-else-if="content()"
        class="bg-(--bg-code)"
      >
        <div class="relative p-(--spacing-lg)">
          <MarkdownView
            v-if="isMarkdownContent() && !showRawContent"
            :content="displayedContent()"
          />
          <pre
            v-else
            class="text-(length:--text-code)"
            :class="[isError() ? 'text-(--color-red)/80' : 'text-(--text-secondary)', (prefsStore.prefs?.wrapCodeLines ?? true) ? 'whitespace-pre-wrap' : 'overflow-x-auto whitespace-pre']"
          >{{ displayedContent() }}</pre>

          <div class="absolute right-(--spacing-sm) top-(--spacing-sm) flex gap-(--spacing-xs)">
            <button
              v-if="isBash() && tool.command"
              type="button"
              title="Copy command"
              class="flex h-6.5 w-6.5 items-center justify-center rounded-sm bg-(--bg-elevated)"
              @click="copyCommand"
            >
              <AppIcon
                :name="commandCopied ? 'checkmark' : 'terminal'"
                size="sm"
                :class="commandCopied ? 'text-(--color-green)' : 'text-(--text-tertiary)'"
              />
            </button>
            <button
              v-if="!isBash()"
              type="button"
              :title="showRawContent ? 'Show rendered markdown' : 'Show raw content'"
              class="flex h-6.5 w-6.5 items-center justify-center rounded-sm bg-(--bg-elevated)"
              @click="showRawContent = !showRawContent"
            >
              <AppIcon
                :name="showRawContent ? 'doc.plaintext' : 'doc.richtext'"
                size="sm"
                :class="showRawContent ? 'text-(--text-tertiary)' : 'text-(--accent)'"
              />
            </button>
            <button
              type="button"
              title="Copy as Markdown"
              class="flex h-6.5 w-6.5 items-center justify-center rounded-sm bg-(--bg-elevated)"
              @click="copyAsMarkdown"
            >
              <AppIcon
                :name="markdownCopied ? 'checkmark' : 'doc.text'"
                size="sm"
                :class="markdownCopied ? 'text-(--color-green)' : 'text-(--text-tertiary)'"
              />
            </button>
            <button
              type="button"
              :title="(prefsStore.prefs?.wrapCodeLines ?? true) ? 'Disable line wrapping' : 'Enable line wrapping'"
              class="flex h-6.5 w-6.5 items-center justify-center rounded-sm bg-(--bg-elevated)"
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
              title="Copy content"
              class="flex h-6.5 w-6.5 items-center justify-center rounded-sm bg-(--bg-elevated)"
              @click="copyContent"
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
            class="w-full py-(--spacing-xs) text-center text-(length:--text-tiny) text-(--text-secondary)"
            :title="showAllLines ? 'Show less' : `Show all ${contentLines().length} lines`"
            @click="showAllLines = !showAllLines"
          >
            {{ showAllLines ? 'Show less' : `Show all ${contentLines().length} lines` }}
          </button>
        </template>
      </div>
      <p
        v-else
        class="bg-(--bg-code) p-(--spacing-lg) text-(length:--text-code) text-(--text-tertiary)"
      >
        No output
      </p>
    </template>
  </div>
</template>
