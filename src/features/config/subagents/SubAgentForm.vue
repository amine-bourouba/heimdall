<script setup lang="ts">
// Ported from SubAgentFormView.swift (M3-49; docs/spec/config-screens.md §9.2): name, the derived
// file name, description, model, colour, tools (all / by category / individually), memory and the
// system prompt. Built-in agents never reach this form (they have no file; the page offers them
// Duplicate only).
//
// Written against what `save_agent` (files.rs) actually does with the payload, not just against
// the Swift form, and deliberately NOT porting five Swift behaviours that silently change or
// destroy an existing agent:
//
// 1. Defaults applied on *edit*: Swift initialises an existing agent's unset model/colour/memory to
//    "sonnet" / orange / "user", so saving an agent that had none of them pins a model, adds a
//    colour and switches persistent memory ON. Here an unset value stays `null` ("not set -- left
//    alone", with a hint) and only a NEW agent gets those defaults. (The writer then keeps the
//    original text of any key whose submitted value equals what the reader showed -- so a colour
//    or memory this form's own enums cannot represent also survives; see `agent_frontmatter`.)
// 2. Tools outside the 12 known: Swift saves `knownTools.filter { selected.contains }`, dropping
//    any other tool the agent had (an MCP tool, say). Those are kept, shown as removable chips.
// 3. A name that slugifies to nothing ("!!!") passes Swift's non-blank check, then fails in the
//    writer: it is invalid here, with a reason.
// 4. The file name shown is the one the save will really write: `unique_slug` never overwrites
//    another agent (a clash becomes `-2`, `-3`), and an edit whose name changes the slug renames
//    the file -- both are said, instead of a preview that can be wrong.
// 5. Errors: a failed save is shown and the form stays open (the app has no global handler).
//
// Keyboard (spec §9.2): Cancel is Esc or Mod+. ; Save/Create is Return while valid, except where
// Return has its own meaning (a focused button, the prompt's new line) -- same contract as
// HookForm.vue, which has the longer account of why this is a callback-prop Teleport modal that
// stops the keys the app's sidebar navigation would otherwise also act on.
import { computed, onMounted, ref } from 'vue';
import { ipc } from '@/ipc/client';
import { useToastsStore } from '@/stores/toasts';
import { usePlatformStore } from '@/stores/platform';
import { isMod } from '@/composables/useShortcuts';
import AppIcon from '@/components/AppIcon.vue';
import { agentSlug, COLORS, KNOWN_TOOLS, MEMORIES, MODELS, newAgentModel, predictSlug, TOOL_CATEGORIES, toInput } from './agentModel';
import type { AgentFormModel } from './agentModel';

const props = defineProps<{ editing?: AgentFormModel | null; existingSlugs: string[]; onClose: () => void; onSaved: () => void }>();

const toastsStore = useToastsStore();
const platformStore = usePlatformStore();

const initial = props.editing ?? newAgentModel();
const isEditing = computed(() => !!props.editing);

const name = ref(initial.name);
const description = ref(initial.description);
const model = ref(initial.model);
const color = ref(initial.color);
const memory = ref(initial.memory);
const prompt = ref(initial.prompt);

const knownTools = new Set<string>(KNOWN_TOOLS);
const selected = ref(new Set(initial.tools.filter((t) => knownTools.has(t))));
const extraTools = ref(initial.tools.filter((t) => !knownTools.has(t)));

const allSelected = computed(() => KNOWN_TOOLS.every((t) => selected.value.has(t)));
function toggleAll(on: boolean) {
  selected.value = new Set(on ? KNOWN_TOOLS : []);
}
function categoryState(tools: readonly string[]) {
  const count = tools.filter((t) => selected.value.has(t)).length;
  return { all: count === tools.length, some: count > 0 && count < tools.length };
}
function toggleTools(tools: readonly string[], on: boolean) {
  const next = new Set(selected.value);
  for (const tool of tools) {
    if (on) next.add(tool);
    else next.delete(tool);
  }
  selected.value = next;
}
const finalTools = computed(() => [...KNOWN_TOOLS.filter((t) => selected.value.has(t)), ...extraTools.value]);

const baseSlug = computed(() => agentSlug(name.value));
const slug = computed(() => predictSlug(name.value, props.editing?.slug ?? null, props.existingSlugs));
const isValid = computed(() => name.value.trim() !== '' && description.value.trim() !== '' && baseSlug.value !== '');

const isSaving = ref(false);

async function save() {
  if (!isValid.value || isSaving.value) return;
  isSaving.value = true;
  try {
    const outcome = await ipc.saveAgent(
      toInput({ slug: null, name: name.value, description: description.value, tools: finalTools.value, model: model.value, color: color.value, memory: memory.value, prompt: prompt.value }),
      props.editing?.slug ?? null,
    );
    if (outcome !== 'written') return;
    toastsStore.show(`${isEditing.value ? 'Updated' : 'Created'} ${name.value.trim()}`);
    props.onSaved();
    props.onClose();
  } catch (error) {
    toastsStore.showError(error, 'Could not save the agent');
  } finally {
    isSaving.value = false;
  }
}

function close() {
  props.onClose();
}

// See HookForm.vue: keys the app's document-level sidebar navigation also acts on.
const NAV_KEYS = new Set(['Escape', 'Enter', 'ArrowUp', 'ArrowDown']);

function onKeydown(event: KeyboardEvent) {
  if (NAV_KEYS.has(event.key)) event.stopPropagation();
  if (event.key === 'Escape' || (event.key === '.' && isMod(event, platformStore.os === 'macos'))) {
    event.preventDefault();
    close();
    return;
  }
  if (event.key !== 'Enter' || event.isComposing) return;
  // A focused button has its own Return, and in the prompt Return is a new line.
  if (event.target instanceof HTMLButtonElement || event.target instanceof HTMLTextAreaElement) return;
  event.preventDefault();
  void save();
}

const dialogEl = ref<HTMLElement | null>(null);
onMounted(() => dialogEl.value?.focus());
</script>

<template>
  <Teleport to="body">
    <div
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/40"
      @click="close"
      @keydown="onKeydown"
    >
      <div
        ref="dialogEl"
        role="dialog"
        aria-modal="true"
        tabindex="-1"
        :aria-label="isEditing ? 'Edit Agent' : 'New Agent'"
        class="glass flex max-h-[90vh] w-130 flex-col rounded-md outline-none"
        @click.stop
      >
        <div class="flex items-center gap-(--spacing-sm) px-(--spacing-xl) py-(--spacing-md)">
          <AppIcon
            name="person.2.fill"
            size="sm"
            class="text-(--color-orange)"
          />
          <h2 class="font-semibold text-(--text-primary)">
            {{ isEditing ? 'Edit Agent' : 'New Agent' }}
          </h2>
        </div>
        <div class="border-t" />

        <div class="flex flex-col gap-(--spacing-lg) overflow-y-auto p-(--spacing-xl)">
          <div class="flex flex-col gap-(--spacing-xs)">
            <label
              for="agent-name"
              class="font-medium text-(--text-secondary) text-(length:--text-caption)"
            >Name</label>
            <input
              id="agent-name"
              v-model="name"
              type="text"
              placeholder="Agent name"
              class="rounded-sm border bg-(--bg-elevated) px-(--spacing-sm) py-(--spacing-xs)"
            >
          </div>

          <div class="flex flex-col gap-(--spacing-xs)">
            <span class="font-medium text-(--text-secondary) text-(length:--text-caption)">File</span>
            <p class="break-all text-(--text-tertiary) text-(length:--text-code-small)">
              {{ slug === '' ? '—' : `~/.claude/agents/${slug}.md` }}
            </p>
            <p
              v-if="name.trim() !== '' && baseSlug === ''"
              role="alert"
              class="text-(--color-red) text-(length:--text-tiny)"
            >
              The name needs at least one letter or number: it becomes the file name.
            </p>
            <p
              v-else-if="editing && slug !== '' && slug !== editing.slug"
              class="text-(--text-secondary) text-(length:--text-tiny)"
            >
              Saving renames the file from {{ editing.slug }}.md.
            </p>
            <p
              v-if="slug !== '' && slug !== baseSlug"
              class="text-(--text-secondary) text-(length:--text-tiny)"
            >
              {{ baseSlug }}.md already exists, so this agent gets its own file name rather than replacing it.
            </p>
          </div>

          <div class="flex flex-col gap-(--spacing-xs)">
            <label
              for="agent-description"
              class="font-medium text-(--text-secondary) text-(length:--text-caption)"
            >Description</label>
            <input
              id="agent-description"
              v-model="description"
              type="text"
              placeholder="What does this agent do?"
              class="rounded-sm border bg-(--bg-elevated) px-(--spacing-sm) py-(--spacing-xs)"
            >
          </div>

          <div
            role="group"
            aria-labelledby="agent-model-label"
            class="flex flex-col gap-(--spacing-xs)"
          >
            <span
              id="agent-model-label"
              class="font-medium text-(--text-secondary) text-(length:--text-caption)"
            >Model</span>
            <div class="flex overflow-hidden rounded-sm border">
              <button
                v-for="m in MODELS"
                :key="m.value"
                type="button"
                :aria-pressed="model === m.value"
                class="flex-1 px-(--spacing-sm) py-(--spacing-xs) text-(length:--text-caption)"
                :class="model === m.value ? 'bg-(--accent-dim) text-(--accent)' : 'text-(--text-secondary)'"
                @click="model = m.value"
              >
                {{ m.label }}
              </button>
            </div>
            <p
              v-if="!MODELS.some((m) => m.value === model)"
              class="text-(--text-tertiary) text-(length:--text-tiny)"
            >
              {{ model === null ? 'Not set on this agent: left as it is unless you pick one.' : `Currently "${model}": kept unless you pick one.` }}
            </p>
          </div>

          <div
            role="group"
            aria-labelledby="agent-color-label"
            class="flex flex-col gap-(--spacing-xs)"
          >
            <span
              id="agent-color-label"
              class="font-medium text-(--text-secondary) text-(length:--text-caption)"
            >Color</span>
            <div class="flex items-center gap-(--spacing-sm)">
              <button
                v-for="c in COLORS"
                :key="c.value"
                type="button"
                :title="c.label"
                :aria-label="c.label"
                :aria-pressed="color === c.value"
                class="h-6 w-6 rounded-full"
                :class="[c.fill, color === c.value ? 'ring-2 ring-(--text-primary) ring-offset-1' : '']"
                @click="color = c.value"
              />
            </div>
            <p
              v-if="color === null"
              class="text-(--text-tertiary) text-(length:--text-tiny)"
            >
              Not set on this agent: left as it is unless you pick one.
            </p>
          </div>

          <div
            role="group"
            aria-labelledby="agent-tools-label"
            class="flex flex-col gap-(--spacing-sm)"
          >
            <div class="flex items-center justify-between">
              <span
                id="agent-tools-label"
                class="font-medium text-(--text-secondary) text-(length:--text-caption)"
              >Tools</span>
              <span class="text-(--text-tertiary) text-(length:--text-tiny)">{{ selected.size }} of {{ KNOWN_TOOLS.length }}</span>
            </div>
            <label class="flex items-center gap-(--spacing-sm) font-medium text-(length:--text-caption)">
              <input
                type="checkbox"
                :checked="allSelected"
                :indeterminate="selected.size > 0 && !allSelected"
                @change="toggleAll(($event.target as HTMLInputElement).checked)"
              >
              All tools
            </label>
            <label
              v-for="category in TOOL_CATEGORIES"
              :key="category.id"
              class="flex items-center gap-(--spacing-sm) pl-(--spacing-md) text-(length:--text-caption)"
            >
              <input
                type="checkbox"
                :checked="categoryState(category.tools).all"
                :indeterminate="categoryState(category.tools).some"
                @change="toggleTools(category.tools, ($event.target as HTMLInputElement).checked)"
              >
              {{ category.name }}
            </label>
            <details>
              <summary class="cursor-pointer text-(--text-tertiary) text-(length:--text-caption)">
                Show individual tools
              </summary>
              <div class="mt-(--spacing-xs) grid grid-cols-3 gap-(--spacing-xs)">
                <label
                  v-for="tool in KNOWN_TOOLS"
                  :key="tool"
                  class="flex items-center gap-(--spacing-xs) text-(length:--text-code-small)"
                >
                  <input
                    type="checkbox"
                    :checked="selected.has(tool)"
                    @change="toggleTools([tool], ($event.target as HTMLInputElement).checked)"
                  >
                  {{ tool }}
                </label>
              </div>
            </details>
            <div
              v-if="extraTools.length > 0"
              class="flex flex-col gap-(--spacing-xs)"
            >
              <p class="text-(--text-tertiary) text-(length:--text-tiny)">
                Other tools on this agent, kept as they are:
              </p>
              <div class="flex flex-wrap gap-(--spacing-xs)">
                <span
                  v-for="tool in extraTools"
                  :key="tool"
                  class="flex items-center gap-(--spacing-xxs) rounded-full bg-(--bg-elevated) px-(--spacing-sm) py-px text-(length:--text-code-small)"
                >
                  {{ tool }}
                  <button
                    type="button"
                    :title="`Remove ${tool}`"
                    :aria-label="`Remove ${tool}`"
                    class="text-(--text-tertiary)"
                    @click="extraTools = extraTools.filter((t) => t !== tool)"
                  >
                    <AppIcon
                      name="xmark.circle"
                      size="sm"
                    />
                  </button>
                </span>
              </div>
            </div>
            <p
              v-if="finalTools.length === 0"
              class="text-(--text-tertiary) text-(length:--text-tiny)"
            >
              No tools selected: the file will have no tools line.
            </p>
          </div>

          <div
            role="radiogroup"
            aria-labelledby="agent-memory-label"
            class="flex flex-col gap-(--spacing-xs)"
          >
            <span
              id="agent-memory-label"
              class="font-medium text-(--text-secondary) text-(length:--text-caption)"
            >Memory</span>
            <label
              v-for="m in MEMORIES"
              :key="m.value"
              class="flex items-center gap-(--spacing-sm) text-(length:--text-caption)"
            >
              <input
                v-model="memory"
                type="radio"
                name="agent-memory"
                :value="m.value"
              >
              {{ m.label }}
            </label>
            <p
              v-if="memory === null"
              class="text-(--text-tertiary) text-(length:--text-tiny)"
            >
              Not set to either on this agent: left as it is unless you pick one.
            </p>
          </div>

          <div class="flex flex-col gap-(--spacing-xs)">
            <label
              for="agent-prompt"
              class="font-medium text-(--text-secondary) text-(length:--text-caption)"
            >System Prompt</label>
            <textarea
              id="agent-prompt"
              v-model="prompt"
              rows="6"
              class="min-h-30 rounded-sm border bg-(--bg-code) p-(--spacing-sm) text-(length:--text-code-small)"
            />
          </div>
        </div>

        <div class="border-t" />
        <div class="flex items-center gap-(--spacing-sm) px-(--spacing-xl) py-(--spacing-md)">
          <button
            type="button"
            title="Cancel"
            class="rounded-sm border px-(--spacing-md) py-(--spacing-xs) text-(length:--text-caption)"
            @click="close"
          >
            Cancel
          </button>
          <div class="flex-1" />
          <button
            type="button"
            :title="isEditing ? 'Save agent' : 'Create agent'"
            class="rounded-sm border border-(--accent)/30 bg-(--accent-dim) px-(--spacing-md) py-(--spacing-xs) text-(--accent) text-(length:--text-caption) disabled:opacity-40"
            :disabled="!isValid || isSaving"
            @click="save"
          >
            {{ isEditing ? 'Save' : 'Create' }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
