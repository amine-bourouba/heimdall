// M3-47's own AC: "Matcher input appears only for events that support it (Hook.swift:52-60);
// default timeouts are 600s for command and 30s for http."
//
// Beyond that AC, this file pins the hazards HookForm.vue's own header comment lists as
// deliberately NOT ported from Swift (spec §10 item 5 and three found in Heimdall's own write
// path): multi-handler truncation, wrong-file writes, event change while editing, silent timeout
// loss -- plus the spec §5.2 keyboard contract (Esc/Mod+. cancel, Return saves).
//
// Every query goes through `document.body`, not `wrapper.find()` -- this component's root is a
// `<Teleport to="body">`; ExportOptionsDialog.spec.ts's own documented reasoning applies here too
// (Vue Test Utils does not search teleported content from the mounting wrapper).
// `onClose`/`onSaved` are passed as `vi.fn()` props (not asserted via `wrapper.emitted()`) --
// confirmed directly, not assumed: a manual `wrapper.vm.$emit(...)` on this component goes
// completely untracked by `wrapper.emitted()`, a real `@vue/test-utils` limitation for any
// component whose entire template root is a `<Teleport>`. HookForm.vue's own header comment
// explains why it uses callback props instead of `defineEmits` for exactly this reason.
import { DOMWrapper, flushPromises, mount } from '@vue/test-utils';
import { mockIPC } from '@tauri-apps/api/mocks';
import { createPinia, setActivePinia } from 'pinia';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { HookEntry, ProjectContext, TargetOs } from '@/ipc/bindings';
import { usePlatformStore } from '@/stores/platform';
import { useConfigStore } from '../store';
import HookForm from './HookForm.vue';

function body() {
  return new DOMWrapper(document.body);
}

let mounted: ReturnType<typeof mount>[] = [];
let savedCalls: unknown[] = [];

interface MountOptions {
  editingEntry?: HookEntry | null;
  outcome?: 'written' | 'cancelled';
  selectedProject?: ProjectContext | null;
  os?: TargetOs;
}

function mountForm({ editingEntry = null, outcome = 'written', selectedProject = null, os }: MountOptions = {}) {
  setActivePinia(createPinia());
  useConfigStore().selectedProject = selectedProject;
  if (os) usePlatformStore().os = os;
  savedCalls = [];
  mockIPC((cmd, args) => {
    if (cmd === 'save_hook') {
      savedCalls.push(args);
      return outcome;
    }
    throw new Error(`unexpected command: ${cmd}`);
  });
  const onClose = vi.fn();
  const onSaved = vi.fn();
  const wrapper = mount(HookForm, { props: { editingEntry, onClose, onSaved }, attachTo: document.body });
  mounted.push(wrapper);
  return { wrapper, onClose, onSaved };
}

function createButton() {
  return body().findAll('button').find((b) => b.text() === 'Create')!;
}

beforeEach(() => {
  document.body.innerHTML = '';
});

afterEach(() => {
  mounted.forEach((w) => w.unmount());
  mounted = [];
});

describe('HookForm: matcher visibility per event (Hook.swift:52-60)', () => {
  it('shows the matcher field for PreToolUse (supports matcher)', () => {
    mountForm();
    expect(body().find('[placeholder="e.g. Bash|Write"]').exists()).toBe(true);
  });

  it('hides the matcher field for Stop (does not support matcher)', async () => {
    mountForm();
    await body().find('select').setValue('Stop');
    expect(body().find('[placeholder="e.g. Bash|Write"]').exists()).toBe(false);
  });

  it.each([
    ['PreToolUse', true],
    ['PostToolUse', true],
    ['PostToolUseFailure', true],
    ['PermissionRequest', true],
    ['SessionStart', true],
    ['SessionEnd', true],
    ['UserPromptSubmit', false],
    ['Notification', true],
    ['SubagentStart', true],
    ['SubagentStop', true],
    ['Stop', false],
    ['TeammateIdle', false],
    ['TaskCompleted', false],
    ['InstructionsLoaded', false],
    ['ConfigChange', true],
    ['WorktreeCreate', false],
    ['WorktreeRemove', false],
    ['PreCompact', true],
  ])('%s: supportsMatcher=%s', async (event, supportsMatcher) => {
    mountForm();
    await body().find('select').setValue(event);
    expect(body().find('[placeholder="e.g. Bash|Write"]').exists()).toBe(supportsMatcher);
  });

  it('does not send a matcher typed for one event after switching to an event that has none', async () => {
    mountForm();
    await body().find('[placeholder="e.g. Bash|Write"]').setValue('Bash');
    await body().find('select').setValue('Stop');
    await body().find('[placeholder="e.g. .claude/hooks/check.sh"]').setValue('x');
    await createButton().trigger('click');
    await flushPromises();
    expect((savedCalls[0] as { group: { matcher: string | null } }).group.matcher).toBeNull();
  });
});

describe('HookForm: default timeouts', () => {
  it('shows "Default: 600" for the command handler type', () => {
    mountForm();
    expect(body().find('[placeholder="Default: 600"]').exists()).toBe(true);
  });

  it('shows "Default: 30" for the http handler type', async () => {
    mountForm();
    await body().findAll('button').find((b) => b.text() === 'HTTP')!.trigger('click');
    expect(body().find('[placeholder="Default: 30"]').exists()).toBe(true);
  });
});

describe('HookForm: validation', () => {
  it('disables Create when the command field is empty', () => {
    mountForm();
    expect(createButton().attributes('disabled')).toBeDefined();
  });

  it('enables Create once a command is entered', async () => {
    mountForm();
    await body().find('[placeholder="e.g. .claude/hooks/check.sh"]').setValue('.claude/hooks/check.sh');
    expect(createButton().attributes('disabled')).toBeUndefined();
  });

  it('validates against the URL field, not Command, once HTTP is selected', async () => {
    mountForm();
    await body().findAll('button').find((b) => b.text() === 'HTTP')!.trigger('click');
    expect(createButton().attributes('disabled')).toBeDefined();
    await body().find('[placeholder="e.g. http://localhost:3000/hook"]').setValue('http://localhost:3000/hook');
    expect(createButton().attributes('disabled')).toBeUndefined();
  });

  it.each([['30s'], ['1.5'], ['-5'], ['abc']])('disables Create and shows a hint for the non-numeric timeout "%s", instead of silently dropping it', async (bad) => {
    mountForm();
    await body().find('[placeholder="e.g. .claude/hooks/check.sh"]').setValue('x');
    await body().find('[placeholder="Default: 600"]').setValue(bad);
    expect(createButton().attributes('disabled')).toBeDefined();
    expect(body().find('[role="alert"]').text()).toContain('whole number');
  });

  it('sends a whole-number timeout as a number, and an empty one as null', async () => {
    mountForm();
    await body().find('[placeholder="e.g. .claude/hooks/check.sh"]').setValue('x');
    await body().find('[placeholder="Default: 600"]').setValue(' 45 ');
    await createButton().trigger('click');
    await flushPromises();
    expect((savedCalls[0] as { group: { handlers: { timeout: number | null }[] } }).group.handlers[0].timeout).toBe(45);
  });
});

describe('HookForm: editing an existing entry', () => {
  const entry: HookEntry = {
    event: 'PostToolUse',
    matcherGroupIndex: 2,
    matcher: 'Bash|Write',
    handlers: [{ type: 'http', command: null, url: 'http://localhost:4000/hook', timeout: 45, statusMessage: 'Checking…', extraKeys: [] }],
    scope: 'global',
  };

  it('pre-fills every field from the entry and shows Save instead of Create', () => {
    mountForm({ editingEntry: entry });
    expect((body().find('select').element as HTMLSelectElement).value).toBe('PostToolUse');
    expect((body().find('[placeholder="e.g. Bash|Write"]').element as HTMLInputElement).value).toBe('Bash|Write');
    expect((body().find('[placeholder="e.g. http://localhost:3000/hook"]').element as HTMLInputElement).value).toBe('http://localhost:4000/hook');
    expect((body().find('[placeholder="Default: 30"]').element as HTMLInputElement).value).toBe('45');
    expect(body().text()).toContain('Edit Hook');
    expect(body().find('button[title="Save hook"]').exists()).toBe(true);
  });

  it('locks the event picker (changing it would overwrite an unrelated group in the new event), but not when creating', () => {
    mountForm({ editingEntry: entry });
    expect(body().find('select').attributes('disabled')).toBeDefined();
    document.body.innerHTML = '';
    mountForm();
    expect(body().find('select').attributes('disabled')).toBeUndefined();
  });

  it('still shows an event newer than this catalog (kept as text) rather than a blank picker', () => {
    mountForm({ editingEntry: { ...entry, event: 'FutureEvent' } });
    expect((body().find('select').element as HTMLSelectElement).value).toBe('FutureEvent');
  });

  it('writes back with the entry\'s own matcherGroupIndex', async () => {
    mountForm({ editingEntry: entry });
    await body().find('button[title="Save hook"]').trigger('click');
    await flushPromises();
    expect((savedCalls[0] as { existingIndex: number }).existingIndex).toBe(2);
  });
});

describe('HookForm: multi-handler groups are not truncated (spec §10 item 5)', () => {
  const multi: HookEntry = {
    event: 'PreToolUse',
    matcherGroupIndex: 0,
    matcher: 'Bash',
    handlers: [
      { type: 'command', command: 'first.sh', url: null, timeout: 10, statusMessage: null, extraKeys: [] },
      { type: 'http', command: null, url: 'http://localhost:1/second', timeout: 5, statusMessage: 'second', extraKeys: ['headers'] },
      { type: 'command', command: 'third.sh', url: null, timeout: null, statusMessage: null, extraKeys: [] },
    ],
    scope: 'global',
  };

  it('says how many handlers are kept unchanged', () => {
    mountForm({ editingEntry: multi });
    expect(body().text()).toContain('This hook has 3 handlers');
    expect(body().text()).toContain('the other 2 are kept unchanged');
  });

  it('sends the edited first handler followed by every other handler as read, without extraKeys', async () => {
    mountForm({ editingEntry: multi });
    await body().find('[placeholder="e.g. .claude/hooks/check.sh"]').setValue('first-edited.sh');
    await body().find('button[title="Save hook"]').trigger('click');
    await flushPromises();
    expect((savedCalls[0] as { group: { handlers: unknown[] } }).group.handlers).toEqual([
      { type: 'command', command: 'first-edited.sh', url: null, timeout: 10, statusMessage: null },
      { type: 'http', command: null, url: 'http://localhost:1/second', timeout: 5, statusMessage: 'second' },
      { type: 'command', command: 'third.sh', url: null, timeout: null, statusMessage: null },
    ]);
  });

  it('shows no such note for a single-handler hook', () => {
    mountForm({ editingEntry: { ...multi, handlers: [multi.handlers[0]] } });
    expect(body().text()).not.toContain('handlers.');
    expect(body().text()).not.toContain('kept unchanged');
  });
});

describe('HookForm: writes target the entry\'s own scope, not just the selected project', () => {
  const project: ProjectContext = { kind: 'project', projectId: 'p1' };
  const globalEntry: HookEntry = { event: 'Stop', matcherGroupIndex: 4, matcher: null, handlers: [{ type: 'command', command: 'g.sh', url: null, timeout: null, statusMessage: null, extraKeys: [] }], scope: 'global' };
  const projectEntry: HookEntry = { ...globalEntry, matcherGroupIndex: 1, scope: 'project' };

  it('a global entry is saved with project: null even while a project is selected (its index belongs to the global file)', async () => {
    mountForm({ editingEntry: globalEntry, selectedProject: project });
    await body().find('button[title="Save hook"]').trigger('click');
    await flushPromises();
    expect((savedCalls[0] as { project: unknown }).project).toBeNull();
  });

  it('a project entry is saved with the selected project', async () => {
    mountForm({ editingEntry: projectEntry, selectedProject: project });
    await body().find('button[title="Save hook"]').trigger('click');
    await flushPromises();
    expect((savedCalls[0] as { project: unknown }).project).toEqual(project);
  });

  it('a NEW hook goes wherever the screen currently points (the selected project)', async () => {
    mountForm({ selectedProject: project });
    await body().find('[placeholder="e.g. .claude/hooks/check.sh"]').setValue('x');
    await createButton().trigger('click');
    await flushPromises();
    expect((savedCalls[0] as { project: unknown }).project).toEqual(project);
  });

  it('the scope badge shows the entry\'s own scope when editing, and the screen\'s when creating', () => {
    mountForm({ editingEntry: globalEntry, selectedProject: project });
    expect(body().text()).toContain('Global');
    document.body.innerHTML = '';
    mountForm({ selectedProject: project });
    expect(body().text()).toContain('Project');
  });
});

describe('HookForm: save', () => {
  it('calls save_hook with the matcher and one handler, then closes on a written outcome', async () => {
    const { onClose, onSaved } = mountForm();
    await body().find('[placeholder="e.g. Bash|Write"]').setValue('Bash');
    await body().find('[placeholder="e.g. .claude/hooks/check.sh"]').setValue('.claude/hooks/check.sh');
    await createButton().trigger('click');
    await flushPromises();

    expect(savedCalls).toEqual([
      {
        project: null,
        event: 'PreToolUse',
        group: { matcher: 'Bash', handlers: [{ type: 'command', command: '.claude/hooks/check.sh', url: null, timeout: null, statusMessage: null }] },
        existingIndex: null,
      },
    ]);
    expect(onSaved).toHaveBeenCalledTimes(1);
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('stays open and does not call onClose/onSaved when the native confirmation is declined (cancelled)', async () => {
    const { onClose, onSaved } = mountForm({ outcome: 'cancelled' });
    await body().find('[placeholder="e.g. .claude/hooks/check.sh"]').setValue('.claude/hooks/check.sh');
    await createButton().trigger('click');
    await flushPromises();

    expect(onSaved).not.toHaveBeenCalled();
    expect(onClose).not.toHaveBeenCalled();
  });

  it('can be retried after a declined confirmation (the in-flight guard is released)', async () => {
    mountForm({ outcome: 'cancelled' });
    await body().find('[placeholder="e.g. .claude/hooks/check.sh"]').setValue('x');
    await createButton().trigger('click');
    await flushPromises();
    await createButton().trigger('click');
    await flushPromises();
    expect(savedCalls).toHaveLength(2);
  });
});

describe('HookForm: keyboard (spec §5.2: Cancel = Esc / Mod+., Save = Return)', () => {
  it('focuses the dialog on open, so Esc/Return work before the user clicks anything', () => {
    mountForm();
    expect(document.activeElement).toBe(body().find('[role="dialog"]').element);
  });

  it('Return in a field saves when valid', async () => {
    const { onSaved } = mountForm();
    const input = body().find('[placeholder="e.g. .claude/hooks/check.sh"]');
    await input.setValue('x');
    await input.trigger('keydown', { key: 'Enter' });
    await flushPromises();
    expect(savedCalls).toHaveLength(1);
    expect(onSaved).toHaveBeenCalledTimes(1);
  });

  it('Return does nothing while the form is invalid', async () => {
    mountForm();
    await body().find('[placeholder="e.g. .claude/hooks/check.sh"]').trigger('keydown', { key: 'Enter' });
    await flushPromises();
    expect(savedCalls).toHaveLength(0);
  });

  it('Return on the focused Cancel button does not also save', async () => {
    mountForm();
    await body().find('[placeholder="e.g. .claude/hooks/check.sh"]').setValue('x');
    await body().find('button[title="Cancel"]').trigger('keydown', { key: 'Enter' });
    await flushPromises();
    expect(savedCalls).toHaveLength(0);
  });

  it('Return on the event select does not save either', async () => {
    mountForm();
    await body().find('[placeholder="e.g. .claude/hooks/check.sh"]').setValue('x');
    await body().find('select').trigger('keydown', { key: 'Enter' });
    await flushPromises();
    expect(savedCalls).toHaveLength(0);
  });

  it('a held-down Return does not send two saves (and two native confirmations)', async () => {
    mountForm();
    const input = body().find('[placeholder="e.g. .claude/hooks/check.sh"]');
    await input.setValue('x');
    void input.trigger('keydown', { key: 'Enter' });
    void input.trigger('keydown', { key: 'Enter' });
    await flushPromises();
    expect(savedCalls).toHaveLength(1);
  });

  it('Return during IME composition is left to the IME', async () => {
    mountForm();
    const input = body().find('[placeholder="e.g. .claude/hooks/check.sh"]');
    await input.setValue('x');
    await input.trigger('keydown', { key: 'Enter', isComposing: true });
    await flushPromises();
    expect(savedCalls).toHaveLength(0);
  });

  it('Ctrl+. cancels on Windows/Linux, and Cmd+. alone does not', async () => {
    const { onClose } = mountForm({ os: 'windows' });
    await body().find('.fixed.inset-0').trigger('keydown', { key: '.', metaKey: true });
    expect(onClose).not.toHaveBeenCalled();
    await body().find('.fixed.inset-0').trigger('keydown', { key: '.', ctrlKey: true });
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('Cmd+. cancels on macOS, and Ctrl+. alone does not', async () => {
    const { onClose } = mountForm({ os: 'macos' });
    await body().find('.fixed.inset-0').trigger('keydown', { key: '.', ctrlKey: true });
    expect(onClose).not.toHaveBeenCalled();
    await body().find('.fixed.inset-0').trigger('keydown', { key: '.', metaKey: true });
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('a bare "." typed into a field does not cancel', async () => {
    const { onClose } = mountForm();
    await body().find('[placeholder="e.g. .claude/hooks/check.sh"]').trigger('keydown', { key: '.' });
    expect(onClose).not.toHaveBeenCalled();
  });
});

// Found by driving the real build, not by any unit test: Return to save also reached the sidebar's
// document-level `useKeyboardNav`, which navigated to the highlighted sidebar item (Sessions) right
// after the save -- and it `preventDefault`s Return and both arrows unconditionally, which would
// also break Return on a focused Cancel and the arrow keys on the event select.
describe('HookForm: the modal owns the keys the app-level navigation would otherwise act on', () => {
  let documentSeen: string[] = [];
  const spy = (event: KeyboardEvent) => documentSeen.push(event.key);

  beforeEach(() => {
    documentSeen = [];
    document.addEventListener('keydown', spy);
  });
  afterEach(() => document.removeEventListener('keydown', spy));

  it.each([['Enter'], ['Escape'], ['ArrowDown'], ['ArrowUp']])('%s does not bubble out to a document-level listener', async (key) => {
    mountForm();
    await body().find('select').trigger('keydown', { key });
    await body().find('[placeholder="e.g. .claude/hooks/check.sh"]').trigger('keydown', { key });
    await body().find('button[title="Cancel"]').trigger('keydown', { key });
    expect(documentSeen).toEqual([]);
  });

  it('Return on a focused button is not preventDefault-ed, so the button\'s own click still happens', async () => {
    mountForm();
    const event = new KeyboardEvent('keydown', { key: 'Enter', bubbles: true, cancelable: true });
    body().find('button[title="Cancel"]').element.dispatchEvent(event);
    expect(event.defaultPrevented).toBe(false);
  });

  it('arrow keys on the event select are not preventDefault-ed, so the picker can be changed from the keyboard', async () => {
    mountForm();
    const event = new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true, cancelable: true });
    body().find('select').element.dispatchEvent(event);
    expect(event.defaultPrevented).toBe(false);
  });

  it('ordinary typing and modifier shortcuts (font size, search) still reach the app-level handlers', async () => {
    mountForm();
    const input = body().find('[placeholder="e.g. .claude/hooks/check.sh"]');
    await input.trigger('keydown', { key: 'a' });
    await input.trigger('keydown', { key: '=', ctrlKey: true });
    await input.trigger('keydown', { key: 'k', ctrlKey: true });
    expect(documentSeen).toEqual(['a', '=', 'k']);
  });
});

describe('HookForm: dismissal', () => {
  it('calls onClose on Cancel', async () => {
    const { onClose } = mountForm();
    await body().find('button[title="Cancel"]').trigger('click');
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('calls onClose on a backdrop click but not on a click inside the dialog', async () => {
    const { onClose } = mountForm();
    await body().find('[role="dialog"]').trigger('click');
    expect(onClose).not.toHaveBeenCalled();
    await body().find('.fixed.inset-0').trigger('click');
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('calls onClose on Escape', async () => {
    const { onClose } = mountForm();
    await body().find('.fixed.inset-0').trigger('keydown', { key: 'Escape' });
    expect(onClose).toHaveBeenCalledTimes(1);
  });
});

describe('HookForm: accessibility', () => {
  it('associates every field label with its input', () => {
    mountForm();
    for (const id of ['hook-event', 'hook-matcher', 'hook-target', 'hook-timeout', 'hook-status']) {
      expect(body().find(`label[for="${id}"]`).exists(), `label for #${id}`).toBe(true);
      expect(body().find(`#${id}`).exists(), `#${id}`).toBe(true);
    }
  });

  it('exposes the handler type choice as a pressed-state button group', async () => {
    mountForm();
    const [command, http] = body().findAll('[aria-labelledby="hook-handler-type"] button');
    expect(command.attributes('aria-pressed')).toBe('true');
    expect(http.attributes('aria-pressed')).toBe('false');
    await http.trigger('click');
    expect(http.attributes('aria-pressed')).toBe('true');
  });
});
