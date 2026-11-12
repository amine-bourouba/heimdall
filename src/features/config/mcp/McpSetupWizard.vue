<script setup lang="ts">
// Ported from MCPSetupWizardView.swift (M3-48; docs/spec/config-screens.md §6.2): pick a catalog server
// or a custom one, configure it, preview, save. Three steps (editing starts at the second, as in Swift).
//
// Written against ADR 0010 and what `add_mcp_server` / `update_mcp_server` actually do, and deliberately
// NOT porting what Swift got wrong (spec §10):
//   1. Arguments are an ordered list (ArgsEditor), never one field split on spaces.
//   2. The preview masks every env and header value completely, custom servers included.
//   3. Editing sends `keep` for a secret nobody touched, so no stored value is ever in the webview and
//      no field is pre-filled; an edit also keeps the transport, url and headers of an http server.
//   Plus two things the spec flags: the SQLite database path is written (as a variable) instead of being
//   collected and dropped, and a name that is already taken is refused before anything is sent.
// Editing always uses the custom form (see `fromServer`): command, arguments and each variable exactly
// as stored.
//
// What is written is decided by Rust, which shows its own native confirmation of the exact command,
// URL and variable names first; a declined confirmation comes back as `cancelled` and nothing is
// written, so this dialog stays open and says so.
//
// Keyboard (spec §6.2): Cancel is Esc or Mod+. on every step; Return is the default action (Preview on
// step 2 while valid, Save on step 3) except where Return has its own meaning (a focused button or
// select). Same contract as HookForm.vue / SubAgentForm.vue: a callback-prop Teleport modal that stops
// the keys the app's sidebar navigation also acts on.
import { computed, nextTick, onMounted, ref } from 'vue';
import { ipc } from '@/ipc/client';
import { useToastsStore } from '@/stores/toasts';
import { usePlatformStore } from '@/stores/platform';
import { isMod } from '@/composables/useShortcuts';
import AppIcon from '@/components/AppIcon.vue';
import ArgsEditor from './ArgsEditor.vue';
import SecretRowsEditor from './SecretRowsEditor.vue';
import { blankCustom, fromCatalog, fromServer, groupCatalog, hasProblems, previewJson, searchCatalog, toInput, TRANSPORTS, validate } from './wizardModel';
import type { WizardModel } from './wizardModel';
import type { McpCatalogEntry, McpServer } from '@/ipc/bindings';

const props = defineProps<{ editing?: McpServer | null; existingNames: string[]; onClose: () => void; onSaved: () => void }>();

const toastsStore = useToastsStore();
const platformStore = usePlatformStore();

const isEditing = computed(() => !!props.editing);
const step = ref<1 | 2 | 3>(props.editing ? 2 : 1);
const model = ref<WizardModel>(props.editing ? fromServer(props.editing) : blankCustom());

const catalog = ref<McpCatalogEntry[]>([]);
const catalogFailed = ref(false);
const query = ref('');
const groups = computed(() => groupCatalog(searchCatalog(catalog.value, query.value)));
const entry = computed(() => (model.value.catalogId ? (catalog.value.find((e) => e.id === model.value.catalogId) ?? null) : null));

const problems = computed(() => validate(model.value, props.existingNames));
const isValid = computed(() => !hasProblems(problems.value));
const preview = computed(() => (step.value === 3 ? previewJson(model.value, entry.value) : ''));
// Whether the written definition will hold any secret value: decides the "stored on disk" note.
const hasSecrets = computed(() => {
  const input = toInput(model.value, entry.value);
  return Object.values({ ...input.env, ...input.headers }).some((s) => s.kind !== 'remove');
});

const stepLabels = computed(() => (isEditing.value ? ['Configure', 'Preview'] : ['Select', 'Configure', 'Preview']));
const activeStep = computed(() => step.value - (isEditing.value ? 2 : 1));

const dialogEl = ref<HTMLElement | null>(null);

async function go(next: 1 | 2 | 3) {
  step.value = next;
  await nextTick();
  // The first field of the new step, so a keyboard user lands where they would type.
  (dialogEl.value?.querySelector<HTMLElement>('[data-autofocus]') ?? dialogEl.value)?.focus();
}

function chooseCustom() {
  model.value = blankCustom();
  void go(2);
}

function chooseEntry(selected: McpCatalogEntry) {
  model.value = fromCatalog(selected);
  void go(2);
}

const isSaving = ref(false);
const declined = ref(false);

async function save() {
  if (step.value !== 3 || !isValid.value || isSaving.value) return;
  isSaving.value = true;
  declined.value = false;
  try {
    const input = toInput(model.value, entry.value);
    const outcome = props.editing ? await ipc.updateMcpServer(props.editing.rawName, input) : await ipc.addMcpServer(input);
    if (outcome === 'cancelled') {
      declined.value = true;
      return;
    }
    if (outcome === 'unchanged') toastsStore.show(`No changes to ${input.name}`, { style: 'info' });
    else toastsStore.show(`${isEditing.value ? 'Saved' : 'Added'} ${input.name}`);
    props.onSaved();
    props.onClose();
  } catch (error) {
    toastsStore.showError(error, `Could not ${isEditing.value ? 'save' : 'add'} the server`);
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
  // A focused button or select has its own Return.
  if (event.target instanceof HTMLButtonElement || event.target instanceof HTMLSelectElement) return;
  event.preventDefault();
  if (step.value === 2 && isValid.value) void go(3);
  else if (step.value === 3) void save();
}

onMounted(async () => {
  if (!isEditing.value) {
    try {
      catalog.value = await ipc.getMcpCatalog();
    } catch (error) {
      catalogFailed.value = true;
      toastsStore.showError(error, 'Could not load the server catalog');
    }
  }
  await nextTick();
  (dialogEl.value?.querySelector<HTMLElement>('[data-autofocus]') ?? dialogEl.value)?.focus();
});

const label = 'font-medium text-(--text-secondary) text-(length:--text-caption)';
const field = 'rounded-sm border bg-(--bg-elevated) px-(--spacing-sm) py-(--spacing-xs)';
const hint = 'text-(--text-tertiary) text-(length:--text-tiny)';
const secondary = 'rounded-sm border px-(--spacing-md) py-(--spacing-xs) text-(length:--text-caption)';
const primary = 'rounded-sm border border-(--accent)/30 bg-(--accent-dim) px-(--spacing-md) py-(--spacing-xs) text-(--accent) text-(length:--text-caption) disabled:opacity-40';
const pick = 'flex w-full items-center gap-(--spacing-md) rounded-md border bg-(--bg-card) p-(--spacing-md) text-left hover:bg-(--bg-card-hover)';
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
        :aria-label="isEditing ? 'Edit MCP Server' : 'Add MCP Server'"
        class="glass flex max-h-[90vh] w-140 flex-col rounded-md outline-none"
        @click.stop
      >
        <div class="flex items-center gap-(--spacing-sm) px-(--spacing-xl) py-(--spacing-md)">
          <AppIcon
            name="powerplug.fill"
            size="sm"
            class="text-(--color-purple)"
          />
          <h2 class="font-semibold text-(--text-primary)">
            {{ isEditing ? 'Edit MCP Server' : 'Add MCP Server' }}
          </h2>
          <div class="flex-1" />
          <ol
            aria-label="Steps"
            class="flex items-center gap-(--spacing-md) text-(length:--text-tiny)"
          >
            <li
              v-for="(stepLabel, index) in stepLabels"
              :key="stepLabel"
              :aria-current="index === activeStep ? 'step' : undefined"
              :class="index === activeStep ? 'font-semibold text-(--accent)' : 'text-(--text-tertiary)'"
            >
              {{ index + 1 }}. {{ stepLabel }}
            </li>
          </ol>
        </div>
        <div class="border-t" />

        <div class="flex flex-col gap-(--spacing-lg) overflow-y-auto p-(--spacing-xl)">
          <!-- Step 1: select -->
          <template v-if="step === 1">
            <input
              v-model="query"
              data-autofocus
              type="search"
              aria-label="Search servers"
              placeholder="Search servers…"
              autocomplete="off"
              :class="field"
            >
            <button
              type="button"
              :class="pick"
              @click="chooseCustom"
            >
              <AppIcon
                name="terminal"
                size="md"
                class="shrink-0 text-(--color-purple)"
              />
              <span class="flex min-w-0 flex-1 flex-col">
                <span class="font-medium text-(--text-primary)">Custom Server</span>
                <span :class="hint">Configure any command or URL yourself</span>
              </span>
            </button>
            <p
              v-if="catalogFailed"
              role="alert"
              class="text-(--color-red) text-(length:--text-small)"
            >
              The catalog could not be loaded. You can still add a custom server.
            </p>
            <section
              v-for="group in groups"
              :key="group.category"
              :aria-label="group.label"
              class="flex flex-col gap-(--spacing-xs)"
            >
              <h3 :class="label">
                {{ group.label }}
              </h3>
              <button
                v-for="item in group.entries"
                :key="item.id"
                type="button"
                :class="pick"
                @click="chooseEntry(item)"
              >
                <AppIcon
                  :name="item.icon"
                  size="md"
                  class="shrink-0 text-(--text-secondary)"
                />
                <span class="flex min-w-0 flex-1 flex-col">
                  <span class="font-medium text-(--text-primary)">{{ item.name }}</span>
                  <span :class="hint">{{ item.description }}</span>
                </span>
                <span
                  v-if="item.envKeys.length === 0"
                  class="shrink-0 text-(--color-green) text-(length:--text-tiny)"
                >No config needed</span>
                <span
                  v-else
                  class="shrink-0 text-(--text-tertiary) text-(length:--text-tiny)"
                >{{ item.envKeys.length }} {{ item.envKeys.length === 1 ? 'var' : 'vars' }}</span>
              </button>
            </section>
            <p
              v-if="query.trim() !== '' && groups.length === 0"
              :class="hint"
            >
              No servers match "{{ query.trim() }}".
            </p>
          </template>

          <!-- Step 2: configure -->
          <template v-else-if="step === 2">
            <div class="flex flex-col gap-(--spacing-xs)">
              <label
                for="mcp-name"
                :class="label"
              >Server name</label>
              <input
                id="mcp-name"
                v-model="model.name"
                data-autofocus
                type="text"
                placeholder="e.g. github"
                autocomplete="off"
                spellcheck="false"
                aria-describedby="mcp-name-hint"
                :class="field"
              >
              <p
                id="mcp-name-hint"
                :class="problems.name?.includes('already exists') ? 'text-(--color-red) text-(length:--text-tiny)' : hint"
                :role="problems.name?.includes('already exists') ? 'alert' : undefined"
              >
                {{ problems.name ?? 'The key used under mcpServers.' }}
              </p>
            </div>

            <!-- A catalog server: its command is fixed, only its variables are filled in. -->
            <template v-if="entry">
              <div class="flex flex-col gap-(--spacing-xs)">
                <span :class="label">Command</span>
                <code class="block break-all rounded-sm bg-(--bg-elevated) p-(--spacing-sm) text-(length:--text-code-small)">{{ [entry.command, ...entry.args].join(' ') }}</code>
              </div>
              <SecretRowsEditor
                v-if="model.env.length > 0"
                v-model="model.env"
                kind="env"
                :allow-add="false"
              />
              <p
                v-else
                class="flex items-center gap-(--spacing-xs) text-(--color-green) text-(length:--text-small)"
              >
                <AppIcon
                  name="checkmark.circle.fill"
                  size="sm"
                />
                No environment variables required
              </p>
              <p
                v-if="problems.env"
                :class="hint"
              >
                {{ problems.env }}
              </p>
            </template>

            <!-- A custom server. -->
            <template v-else>
              <fieldset class="flex flex-col gap-(--spacing-xs)">
                <legend :class="label">
                  Connection
                </legend>
                <div class="flex flex-wrap items-center gap-(--spacing-lg)">
                  <label
                    v-for="t in TRANSPORTS"
                    :key="t.value"
                    class="flex items-center gap-(--spacing-xs) text-(length:--text-caption)"
                  >
                    <input
                      v-model="model.transport"
                      type="radio"
                      name="mcp-transport"
                      :value="t.value"
                    >
                    {{ t.label }}
                  </label>
                </div>
              </fieldset>

              <template v-if="model.transport === 'stdio'">
                <div class="flex flex-col gap-(--spacing-xs)">
                  <label
                    for="mcp-command"
                    :class="label"
                  >Command</label>
                  <input
                    id="mcp-command"
                    v-model="model.command"
                    type="text"
                    placeholder="e.g. npx"
                    autocomplete="off"
                    spellcheck="false"
                    aria-describedby="mcp-command-hint"
                    :class="[field, 'font-mono text-(length:--text-code-small)']"
                  >
                  <p
                    id="mcp-command-hint"
                    :class="hint"
                  >
                    {{ problems.command ?? 'The program to run.' }}
                  </p>
                </div>
                <div class="flex flex-col gap-(--spacing-xs)">
                  <span :class="label">Arguments</span>
                  <ArgsEditor v-model="model.args" />
                  <p :class="hint">
                    One argument per line, passed exactly as typed. An argument with spaces stays one argument.
                  </p>
                </div>
                <div class="flex flex-col gap-(--spacing-xs)">
                  <span :class="label">Environment variables</span>
                  <SecretRowsEditor
                    v-model="model.env"
                    kind="env"
                  />
                  <p
                    v-if="problems.env"
                    :class="hint"
                  >
                    {{ problems.env }}
                  </p>
                </div>
              </template>
              <template v-else>
                <div class="flex flex-col gap-(--spacing-xs)">
                  <label
                    for="mcp-url"
                    :class="label"
                  >URL</label>
                  <input
                    id="mcp-url"
                    v-model="model.url"
                    type="url"
                    placeholder="https://example.com/mcp"
                    autocomplete="off"
                    spellcheck="false"
                    aria-describedby="mcp-url-hint"
                    :class="[field, 'font-mono text-(length:--text-code-small)']"
                  >
                  <p
                    id="mcp-url-hint"
                    :class="hint"
                  >
                    {{ problems.url ?? 'The server\'s address.' }}
                  </p>
                </div>
                <div class="flex flex-col gap-(--spacing-xs)">
                  <span :class="label">Headers</span>
                  <SecretRowsEditor
                    v-model="model.headers"
                    kind="header"
                  />
                  <p
                    v-if="problems.headers"
                    :class="hint"
                  >
                    {{ problems.headers }}
                  </p>
                </div>
              </template>
            </template>
          </template>

          <!-- Step 3: preview and save -->
          <template v-else>
            <div class="flex flex-col gap-(--spacing-xs)">
              <span
                id="mcp-preview-label"
                :class="label"
              >Preview</span>
              <pre
                aria-labelledby="mcp-preview-label"
                class="overflow-x-auto rounded-sm bg-(--bg-elevated) p-(--spacing-md) font-mono text-(length:--text-code-small) select-text"
              >{{ preview }}</pre>
              <p :class="hint">
                Every environment variable and header value is hidden here. The preview shows what will be written, not the values.
              </p>
            </div>
            <ul class="flex flex-col gap-(--spacing-xs) text-(--text-secondary) text-(length:--text-small)">
              <li>This will be written to <code>~/.claude.json</code> under the <code>mcpServers</code> key.</li>
              <li v-if="hasSecrets">
                Secret values are stored in <code>~/.claude.json</code> on disk.
              </li>
              <li>You will be asked to confirm the exact command, URL and variable names before anything is written.</li>
            </ul>
            <p
              v-if="declined"
              role="status"
              class="text-(--text-secondary) text-(length:--text-small)"
            >
              Nothing was written: the confirmation was declined. You can change something and try again.
            </p>
          </template>
        </div>

        <div class="border-t" />
        <div class="flex items-center gap-(--spacing-sm) px-(--spacing-xl) py-(--spacing-md)">
          <button
            type="button"
            title="Cancel"
            :class="secondary"
            @click="close"
          >
            Cancel
          </button>
          <div class="flex-1" />
          <button
            v-if="step === 3 || (step === 2 && !isEditing)"
            type="button"
            title="Back"
            :class="secondary"
            @click="go((step - 1) as 1 | 2)"
          >
            Back
          </button>
          <button
            v-if="step === 2"
            type="button"
            title="Preview"
            :class="primary"
            :disabled="!isValid"
            @click="go(3)"
          >
            Preview
          </button>
          <button
            v-else-if="step === 3"
            data-autofocus
            type="button"
            :title="isEditing ? 'Save server' : 'Add server'"
            :class="primary"
            :disabled="isSaving"
            @click="save"
          >
            {{ isEditing ? 'Save' : 'Add Server' }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
