<script setup lang="ts">
// Ported from HookFormView.swift (M3-47; docs/spec/config-screens.md §5.2), deliberately NOT
// porting four hazards that same spec flags or that follow from how Heimdall's write commands work:
//
// 1. Multi-handler truncation (spec §10 item 5, "the single most destructive bug found in the
//    config screens"): Swift's form only reads/writes `firstHandler`, so saving a group with N
//    handlers silently deletes the other N-1. `save_hook` replaces the whole group with exactly
//    the handlers it is given (`serialize_group` in settings.rs), so this form edits the first
//    handler and re-sends the rest unchanged (Rust merges each handler's unknown keys back by
//    index) -- nothing is dropped, and the native confirmation Rust shows lists every handler.
// 2. Wrong-file writes: `matcherGroupIndex` is an index into the entry's OWN settings.json, and
//    the list merges global + project entries, so an existing entry must be written back with its
//    own scope's context, not whatever project happens to be selected (`writeContext` below).
// 3. Changing the event while editing: `save_hook` replaces `groups[existingIndex]` of the event
//    it is given, so a changed event would overwrite an unrelated group in the new event (or
//    duplicate the hook). The event picker is disabled when editing; move a hook by creating a
//    new one and deleting the old.
// 4. Silent timeout loss: Swift's `Int(timeout)` turns a typo like "30s" into "no timeout" with a
//    success toast anyway (spec §5.2 field 5, §10 item 10). A non-empty, non-numeric timeout
//    disables Save here instead (the field is `Option<u64>` in Rust, so a fraction or a negative
//    would fail IPC deserialisation too).
//
// Event metadata (icon/description/supportsMatcher) is Hook.swift's own HookEvent enum, restated
// here directly rather than derived: Heimdall's own HookEntry.event is a free string, not a
// closed enum (M3-13's own HooksPage.vue header comment already explains why -- "kept as text so
// events newer than Heimdall still show up"), so there is nothing server-side to read this from.
//
// Keyboard (spec §5.2): Cancel is Esc or Mod+. (Cmd on macOS, Ctrl elsewhere); Save/Create is
// Return, only while valid. Return is ignored on a focused button or the event select, which have
// their own Return behaviour (a focused Cancel must cancel, not save).
import { computed, onMounted, ref } from 'vue';
import { ipc } from '@/ipc/client';
import { useToastsStore } from '@/stores/toasts';
import { usePlatformStore } from '@/stores/platform';
import { isMod } from '@/composables/useShortcuts';
import type { HookEntry, HookHandlerType } from '@/ipc/bindings';
import AppIcon from '@/components/AppIcon.vue';
import ConfigBadge from '../shared/ConfigBadge.vue';
import { useConfigStore } from '../store';

// `onClose`/`onSaved` are callback props, not `defineEmits` -- confirmed directly, not assumed:
// `@vue/test-utils` does not track emits at all for a component whose entire template root is a
// `<Teleport>` (even a manual `wrapper.vm.$emit('close')` goes untracked by `wrapper.emitted()`
// here), which is exactly why every other Teleport-rooted dialog in this codebase
// (ExportOptionsDialog.vue, DebugLogSheet.vue, ShortcutHelp.vue, Onboarding.vue) already avoids
// emits, closing via a shared store flag instead. A store flag would mean adding hook-specific
// state to the cross-screen `configStore` (used by every config screen, not just this one) --
// plain callback props avoid that too, while still testing correctly: calling a prop is not an
// emit, so `wrapper.emitted()`'s gap never applies to it.
const props = defineProps<{ editingEntry?: HookEntry | null; onClose: () => void; onSaved: () => void }>();

const configStore = useConfigStore();
const toastsStore = useToastsStore();
const platformStore = usePlatformStore();

// Hook.swift:5-23's own enum-declaration order (not alphabetical) -- matches the Picker's own
// row order in HookFormView.swift.
const HOOK_EVENTS: { value: string; label: string; icon: string; description: string; supportsMatcher: boolean }[] = [
  { value: 'PreToolUse', label: 'PreToolUse', icon: 'hand.raised', description: 'Before a tool call executes (can block)', supportsMatcher: true },
  { value: 'PostToolUse', label: 'PostToolUse', icon: 'checkmark.circle', description: 'After a tool call succeeds', supportsMatcher: true },
  { value: 'PostToolUseFailure', label: 'PostToolUseFailure', icon: 'xmark.circle', description: 'After a tool call fails', supportsMatcher: true },
  { value: 'PermissionRequest', label: 'PermissionRequest', icon: 'lock.shield', description: 'When a permission dialog appears', supportsMatcher: true },
  { value: 'SessionStart', label: 'SessionStart', icon: 'play.circle', description: 'When a session begins or resumes', supportsMatcher: true },
  { value: 'SessionEnd', label: 'SessionEnd', icon: 'stop.circle', description: 'When a session terminates', supportsMatcher: true },
  { value: 'UserPromptSubmit', label: 'UserPromptSubmit', icon: 'text.bubble', description: 'When a prompt is submitted', supportsMatcher: false },
  { value: 'Notification', label: 'Notification', icon: 'bell', description: 'When a notification is sent', supportsMatcher: true },
  { value: 'SubagentStart', label: 'SubagentStart', icon: 'person.badge.plus', description: 'When a subagent is spawned', supportsMatcher: true },
  { value: 'SubagentStop', label: 'SubagentStop', icon: 'person.badge.minus', description: 'When a subagent finishes', supportsMatcher: true },
  { value: 'Stop', label: 'Stop', icon: 'stop.fill', description: 'When Claude finishes responding', supportsMatcher: false },
  { value: 'TeammateIdle', label: 'TeammateIdle', icon: 'person.crop.circle.badge.clock', description: 'When a teammate is about to go idle', supportsMatcher: false },
  { value: 'TaskCompleted', label: 'TaskCompleted', icon: 'checkmark.seal', description: 'When a task is marked completed', supportsMatcher: false },
  { value: 'InstructionsLoaded', label: 'InstructionsLoaded', icon: 'doc.text', description: 'When CLAUDE.md or rules are loaded', supportsMatcher: false },
  { value: 'ConfigChange', label: 'ConfigChange', icon: 'gearshape', description: 'When a config file changes', supportsMatcher: true },
  { value: 'WorktreeCreate', label: 'WorktreeCreate', icon: 'plus.rectangle.on.folder', description: 'When a worktree is being created', supportsMatcher: false },
  { value: 'WorktreeRemove', label: 'WorktreeRemove', icon: 'minus.rectangle', description: 'When a worktree is being removed', supportsMatcher: false },
  { value: 'PreCompact', label: 'PreCompact', icon: 'arrow.down.right.and.arrow.up.left', description: 'Before context compaction', supportsMatcher: true },
];

const HANDLER_TYPES: HookHandlerType[] = ['command', 'http'];

const isEditing = computed(() => !!props.editingEntry);

const selectedEvent = ref(props.editingEntry?.event ?? HOOK_EVENTS[0].value);
const matcher = ref(props.editingEntry?.matcher ?? '');
const firstHandler = props.editingEntry?.handlers[0];
const handlerType = ref<HookHandlerType>(firstHandler?.type ?? 'command');
const command = ref(firstHandler?.command ?? '');
const url = ref(firstHandler?.url ?? '');
const timeout = ref(firstHandler?.timeout != null ? String(firstHandler.timeout) : '');
const statusMessage = ref(firstHandler?.statusMessage ?? '');

// An event this app doesn't recognise (a free string from a newer settings.json) gets a plain
// fallback rather than crashing the form -- the same "kept as text" tolerance HooksPage.vue
// already extends to display.
const selectedEventMeta = computed(() => HOOK_EVENTS.find((e) => e.value === selectedEvent.value) ?? { value: selectedEvent.value, label: selectedEvent.value, icon: 'arrow.triangle.branch', description: '', supportsMatcher: true });

// An event newer than this catalog still needs an option to show while editing it.
const eventOptions = computed(() => (HOOK_EVENTS.some((e) => e.value === selectedEvent.value) ? HOOK_EVENTS : [selectedEventMeta.value, ...HOOK_EVENTS]));

// The handlers after the first, kept exactly as read (hazard 1). Only the fields `HookHandlerInput`
// has -- never `extraKeys`, which Rust merges back from the file by index.
const untouchedHandlers = (props.editingEntry?.handlers.slice(1) ?? []).map((h) => ({
  type: h.type,
  command: h.command,
  url: h.url,
  timeout: h.timeout,
  statusMessage: h.statusMessage,
}));

// Hazard 2: an existing entry is written back with its own scope's context. A project-scope entry
// only ever appears while a project is selected (the reader adds the project file only then), so
// `selectedProject` is the right context for it; a global-scope entry is always `null`, even with
// a project selected. A new hook goes wherever the screen currently points.
const writeContext = computed(() => {
  if (props.editingEntry) return props.editingEntry.scope === 'project' ? configStore.selectedProject : null;
  return configStore.selectedProject;
});
const scope = computed(() => (props.editingEntry ? props.editingEntry.scope : configStore.selectedProject ? 'project' : 'global'));

const isTimeoutValid = computed(() => timeout.value.trim() === '' || /^\d+$/.test(timeout.value.trim()));
const isValid = computed(() => isTimeoutValid.value && (handlerType.value === 'command' ? command.value.trim() !== '' : url.value.trim() !== ''));

const isSaving = ref(false);

async function save() {
  if (!isValid.value || isSaving.value) return;
  isSaving.value = true;
  try {
    const trimmedMatcher = matcher.value.trim();
    const trimmedTimeout = timeout.value.trim();
    const outcome = await ipc.saveHook(
      writeContext.value,
      selectedEvent.value,
      {
        matcher: selectedEventMeta.value.supportsMatcher && trimmedMatcher !== '' ? trimmedMatcher : null,
        handlers: [
          {
            type: handlerType.value,
            command: handlerType.value === 'command' ? command.value.trim() : null,
            url: handlerType.value === 'http' ? url.value.trim() : null,
            timeout: trimmedTimeout === '' ? null : Number(trimmedTimeout),
            statusMessage: statusMessage.value.trim() === '' ? null : statusMessage.value.trim(),
          },
          ...untouchedHandlers,
        ],
      },
      props.editingEntry?.matcherGroupIndex ?? null,
    );
    // save_hook shows a native, Rust-side confirmation naming every handler's exact command/URL
    // before writing (M3-44) -- declining it returns 'cancelled', and the form stays open rather
    // than closing on a write that didn't happen (ExportOptionsDialog.vue's own convention).
    if (outcome !== 'written') return;
    toastsStore.show(isEditing.value ? 'Hook updated' : 'Hook created');
    props.onSaved();
    props.onClose();
  } catch (error) {
    // A busy file, a settings.json that does not parse (ADR 0009 refuses to overwrite it), a bad
    // project context ... -- shown, and the form stays open so nothing typed is lost.
    toastsStore.showError(error, 'Could not save the hook');
  } finally {
    isSaving.value = false;
  }
}

function close() {
  props.onClose();
}

// The keys the app's document-level sidebar navigation (`useKeyboardNav`) owns. Left to bubble out
// of this modal it would ALSO act on them: Return would navigate to the highlighted sidebar item
// (Sessions, by default) right after saving -- found by driving the real build, not by any unit
// test -- and it `preventDefault`s Return and both arrows unconditionally, so a focused Cancel
// button could not be activated with Return and the event select could not be changed with the
// arrow keys. The same fix SearchOverlay.vue already needed (M3-19). Modifier shortcuts (font
// size, search) are not in this set and keep working.
const NAV_KEYS = new Set(['Escape', 'Enter', 'ArrowUp', 'ArrowDown']);

function onKeydown(event: KeyboardEvent) {
  if (NAV_KEYS.has(event.key)) event.stopPropagation();
  if (event.key === 'Escape' || (event.key === '.' && isMod(event, platformStore.os === 'macos'))) {
    event.preventDefault();
    close();
    return;
  }
  if (event.key !== 'Enter' || event.isComposing) return;
  // A focused button or the event select already has its own Return behaviour: Cancel must cancel
  // (not save), and Return on the select opens it.
  if (event.target instanceof HTMLButtonElement || event.target instanceof HTMLSelectElement) return;
  event.preventDefault();
  void save();
}

// Key events only reach the backdrop's handler from inside the dialog, so focus it on open --
// otherwise Esc/Return would do nothing until the user clicked a field.
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
        :aria-label="isEditing ? 'Edit Hook' : 'New Hook'"
        class="glass flex max-h-[90vh] w-120 flex-col rounded-md outline-none"
        @click.stop
      >
        <div class="flex items-center gap-(--spacing-sm) px-(--spacing-xl) py-(--spacing-md)">
          <AppIcon
            name="arrow.triangle.branch"
            size="sm"
            class="text-(--color-orange)"
          />
          <h2 class="font-semibold text-(--text-primary)">
            {{ isEditing ? 'Edit Hook' : 'New Hook' }}
          </h2>
          <div class="flex-1" />
          <ConfigBadge :scope="scope" />
        </div>
        <div class="border-t" />

        <div class="flex flex-col gap-(--spacing-lg) overflow-y-auto p-(--spacing-xl)">
          <p
            v-if="untouchedHandlers.length > 0"
            class="rounded-sm bg-(--bg-elevated) px-(--spacing-sm) py-(--spacing-xs) text-(--text-secondary) text-(length:--text-tiny)"
          >
            This hook has {{ untouchedHandlers.length + 1 }} handlers. Only the first is editable here; the other {{ untouchedHandlers.length }} are kept unchanged.
          </p>

          <div class="flex flex-col gap-(--spacing-xs)">
            <label
              for="hook-event"
              class="font-medium text-(--text-secondary) text-(length:--text-caption)"
            >Event</label>
            <select
              id="hook-event"
              v-model="selectedEvent"
              :disabled="isEditing"
              :title="isEditing ? 'The event of an existing hook cannot be changed. Create a new hook and delete this one to move it.' : undefined"
              class="rounded-sm border bg-(--bg-elevated) px-(--spacing-sm) py-(--spacing-xs) disabled:opacity-60"
            >
              <option
                v-for="opt in eventOptions"
                :key="opt.value"
                :value="opt.value"
              >
                {{ opt.label }}
              </option>
            </select>
            <p class="text-(--text-tertiary) text-(length:--text-tiny)">
              {{ selectedEventMeta.description }}
            </p>
          </div>

          <div
            v-if="selectedEventMeta.supportsMatcher"
            class="flex flex-col gap-(--spacing-xs)"
          >
            <label
              for="hook-matcher"
              class="font-medium text-(--text-secondary) text-(length:--text-caption)"
            >Matcher (regex)</label>
            <input
              id="hook-matcher"
              v-model="matcher"
              type="text"
              placeholder="e.g. Bash|Write"
              class="rounded-sm border bg-(--bg-elevated) px-(--spacing-sm) py-(--spacing-xs) text-(length:--text-code-small)"
            >
            <p class="text-(--text-tertiary) text-(length:--text-tiny)">
              Optional regex to filter when the hook fires
            </p>
          </div>

          <div
            role="group"
            aria-labelledby="hook-handler-type"
            class="flex flex-col gap-(--spacing-xs)"
          >
            <span
              id="hook-handler-type"
              class="font-medium text-(--text-secondary) text-(length:--text-caption)"
            >Handler Type</span>
            <div class="flex overflow-hidden rounded-sm border">
              <button
                v-for="type in HANDLER_TYPES"
                :key="type"
                type="button"
                :aria-pressed="handlerType === type"
                class="flex-1 px-(--spacing-sm) py-(--spacing-xs) text-(length:--text-caption)"
                :class="handlerType === type ? 'bg-(--accent-dim) text-(--accent)' : 'text-(--text-secondary)'"
                @click="handlerType = type"
              >
                {{ type === 'command' ? 'Command' : 'HTTP' }}
              </button>
            </div>
          </div>

          <div class="flex flex-col gap-(--spacing-xs)">
            <label
              for="hook-target"
              class="font-medium text-(--text-secondary) text-(length:--text-caption)"
            >{{ handlerType === 'command' ? 'Command' : 'URL' }}</label>
            <input
              v-if="handlerType === 'command'"
              id="hook-target"
              v-model="command"
              type="text"
              placeholder="e.g. .claude/hooks/check.sh"
              class="rounded-sm border bg-(--bg-elevated) px-(--spacing-sm) py-(--spacing-xs) text-(length:--text-code-small)"
            >
            <input
              v-else
              id="hook-target"
              v-model="url"
              type="text"
              placeholder="e.g. http://localhost:3000/hook"
              class="rounded-sm border bg-(--bg-elevated) px-(--spacing-sm) py-(--spacing-xs) text-(length:--text-code-small)"
            >
          </div>

          <div class="flex flex-col gap-(--spacing-xs)">
            <label
              for="hook-timeout"
              class="font-medium text-(--text-secondary) text-(length:--text-caption)"
            >Timeout (seconds)</label>
            <input
              id="hook-timeout"
              v-model="timeout"
              type="text"
              inputmode="numeric"
              :aria-invalid="!isTimeoutValid"
              :placeholder="handlerType === 'command' ? 'Default: 600' : 'Default: 30'"
              class="rounded-sm border bg-(--bg-elevated) px-(--spacing-sm) py-(--spacing-xs) text-(length:--text-code-small)"
              :class="isTimeoutValid ? '' : 'border-(--color-red)'"
            >
            <p
              v-if="!isTimeoutValid"
              role="alert"
              class="text-(--color-red) text-(length:--text-tiny)"
            >
              Timeout must be a whole number of seconds
            </p>
          </div>

          <div class="flex flex-col gap-(--spacing-xs)">
            <label
              for="hook-status"
              class="font-medium text-(--text-secondary) text-(length:--text-caption)"
            >Status Message</label>
            <input
              id="hook-status"
              v-model="statusMessage"
              type="text"
              placeholder="e.g. Checking command safety…"
              class="rounded-sm border bg-(--bg-elevated) px-(--spacing-sm) py-(--spacing-xs) text-(length:--text-code-small)"
            >
            <p class="text-(--text-tertiary) text-(length:--text-tiny)">
              Shown in Claude Code while the hook runs
            </p>
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
            :title="isEditing ? 'Save hook' : 'Create hook'"
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
