// M3-49's own AC: "Fields: name, description, tools by category, model, colour, memory, prompt;
// built-in agents stay read-only." (The read-only half lives in SubAgentsPage.spec.ts: a built-in
// never reaches this form.)
//
// Beyond that, this file pins the five behaviours SubAgentForm.vue's own header lists as
// deliberately NOT ported from Swift -- defaults applied on edit, tools outside the known 12
// dropped, an un-sluggable name, a wrong file-name preview, swallowed errors.
//
// Every query goes through `document.body` (the root is a `<Teleport to="body">`) and
// `onClose`/`onSaved` are `vi.fn()` callback props, for the reasons HookForm.spec.ts documents.
import { DOMWrapper, flushPromises, mount } from '@vue/test-utils';
import { mockIPC } from '@tauri-apps/api/mocks';
import { createPinia, setActivePinia } from 'pinia';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { AgentInput, TargetOs } from '@/ipc/bindings';
import { usePlatformStore } from '@/stores/platform';
import { useToastsStore } from '@/stores/toasts';
import type { AgentFormModel } from './agentModel';
import SubAgentForm from './SubAgentForm.vue';

function body() {
  return new DOMWrapper(document.body);
}

interface SavedCall {
  input: AgentInput;
  originalSlug: string | null;
}

let mounted: ReturnType<typeof mount>[] = [];
let savedCalls: SavedCall[] = [];

interface MountOptions {
  editing?: AgentFormModel | null;
  existingSlugs?: string[];
  outcome?: 'written' | 'error';
  os?: TargetOs;
}

function mountForm({ editing = null, existingSlugs = [], outcome = 'written', os }: MountOptions = {}) {
  setActivePinia(createPinia());
  if (os) usePlatformStore().os = os;
  savedCalls = [];
  mockIPC((cmd, args) => {
    if (cmd === 'save_agent') {
      savedCalls.push(args as unknown as SavedCall);
      if (outcome === 'error') throw { kind: 'io', message: 'agents folder is read-only', path: null };
      return 'written';
    }
    throw new Error(`unexpected command: ${cmd}`);
  });
  const onClose = vi.fn();
  const onSaved = vi.fn();
  const wrapper = mount(SubAgentForm, { props: { editing, existingSlugs, onClose, onSaved }, attachTo: document.body });
  mounted.push(wrapper);
  return { wrapper, onClose, onSaved };
}

function editing(overrides: Partial<AgentFormModel> = {}): AgentFormModel {
  return {
    slug: 'reviewer',
    name: 'Reviewer',
    description: 'Reviews code',
    tools: ['Read', 'Grep'],
    model: 'sonnet',
    color: 'blue',
    memory: 'user',
    prompt: 'Be helpful.',
    ...overrides,
  };
}

const nameInput = () => body().find('#agent-name');
const descInput = () => body().find('#agent-description');
const createButton = () => body().findAll('button').find((b) => b.text() === 'Create')!;
const saveButton = () => body().find('button[title="Save agent"]');
const checkbox = (label: string) => body().findAll('label').find((l) => l.text() === label)!.find('input');
const pressed = (title: string) => body().find(`button[title="${title}"]`).attributes('aria-pressed');

async function fillValid(name = 'Code Reviewer') {
  await nameInput().setValue(name);
  await descInput().setValue('Reviews code');
}

beforeEach(() => {
  document.body.innerHTML = '';
});

afterEach(() => {
  mounted.forEach((w) => w.unmount());
  mounted = [];
});

describe('SubAgentForm: a new agent', () => {
  it('starts with Swift\'s defaults: Glob/Grep/Read/Bash, Sonnet, orange, memory on', () => {
    mountForm();
    expect(body().text()).toContain('4 of 12');
    for (const tool of ['Glob', 'Grep', 'Read', 'Bash']) expect((checkbox(tool).element as HTMLInputElement).checked).toBe(true);
    expect((checkbox('Edit').element as HTMLInputElement).checked).toBe(false);
    expect(body().findAll('[aria-labelledby="agent-model-label"] button').map((b) => b.attributes('aria-pressed'))).toEqual(['false', 'true', 'false']);
    expect(pressed('Orange')).toBe('true');
    expect((body().find('input[type="radio"][name="agent-memory"]:checked').element as HTMLInputElement).value).toBe('user');
    expect(body().text()).toContain('New Agent');
  });

  it('disables Create until both a name and a description are entered', async () => {
    mountForm();
    expect(createButton().attributes('disabled')).toBeDefined();
    await nameInput().setValue('Reviewer');
    expect(createButton().attributes('disabled')).toBeDefined();
    await descInput().setValue('Reviews code');
    expect(createButton().attributes('disabled')).toBeUndefined();
  });

  it('treats a whitespace-only name or description as blank', async () => {
    mountForm();
    await nameInput().setValue('   ');
    await descInput().setValue('   ');
    expect(createButton().attributes('disabled')).toBeDefined();
  });

  it('saves the full payload in the known-tool order, a new agent having no original slug', async () => {
    const { onSaved, onClose } = mountForm();
    await fillValid('  Code Reviewer  ');
    await body().find('#agent-prompt').setValue('Be helpful.\nBe brief.');
    await createButton().trigger('click');
    await flushPromises();
    expect(savedCalls).toEqual([
      {
        input: {
          name: 'Code Reviewer',
          icon: 'person.crop.circle.badge.plus',
          description: 'Reviews code',
          tools: ['Bash', 'Glob', 'Grep', 'Read'],
          model: 'sonnet',
          color: 'orange',
          prompt: 'Be helpful.\nBe brief.',
          memory: 'user',
        },
        originalSlug: null,
      },
    ]);
    expect(useToastsStore().current).toMatchObject({ message: 'Created Code Reviewer', style: 'success' });
    expect(onSaved).toHaveBeenCalledTimes(1);
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('sends an empty prompt as null', async () => {
    mountForm();
    await fillValid();
    await createButton().trigger('click');
    await flushPromises();
    expect(savedCalls[0].input.prompt).toBeNull();
  });

  it('sends exactly what was chosen for model, colour and memory', async () => {
    mountForm();
    await fillValid();
    await body().find('[aria-labelledby="agent-model-label"]').findAll('button')[0].trigger('click'); // Opus
    await body().find('button[title="Teal"]').trigger('click');
    await body().findAll('input[type="radio"][name="agent-memory"]')[1].setValue(); // None
    await createButton().trigger('click');
    await flushPromises();
    expect(savedCalls[0].input).toMatchObject({ model: 'opus', color: 'teal', memory: 'none' });
  });
});

describe('SubAgentForm: tools by category', () => {
  it('"All tools" selects and clears every known tool', async () => {
    mountForm();
    await checkbox('All tools').setValue(true);
    expect(body().text()).toContain('12 of 12');
    await checkbox('All tools').setValue(false);
    expect(body().text()).toContain('0 of 12');
  });

  it('a category toggles exactly its own tools', async () => {
    mountForm();
    await checkbox('Edit tools').setValue(true);
    for (const tool of ['Edit', 'Write', 'NotebookEdit']) expect((checkbox(tool).element as HTMLInputElement).checked).toBe(true);
    expect(body().text()).toContain('7 of 12');
    await checkbox('Edit tools').setValue(false);
    expect(body().text()).toContain('4 of 12');
  });

  it('shows a partly selected group as indeterminate, not checked', () => {
    mountForm(); // Read-only tools: Glob, Grep, Read selected; WebFetch, WebSearch not
    const readOnly = checkbox('Read-only tools').element as HTMLInputElement;
    expect([readOnly.checked, readOnly.indeterminate]).toEqual([false, true]);
    const all = checkbox('All tools').element as HTMLInputElement;
    expect([all.checked, all.indeterminate]).toEqual([false, true]);
  });

  it('an individual tool toggles on its own', async () => {
    mountForm();
    await checkbox('WebFetch').setValue(true);
    expect(body().text()).toContain('5 of 12');
    await fillValid();
    await createButton().trigger('click');
    await flushPromises();
    expect(savedCalls[0].input.tools).toEqual(['Bash', 'Glob', 'Grep', 'Read', 'WebFetch']);
  });

  it('says so when no tool is selected, since the file then has no tools line', async () => {
    mountForm();
    // 4 of 12 is an unchecked-but-indeterminate "All tools": a click checks it first, then clears.
    await checkbox('All tools').setValue(true);
    await checkbox('All tools').setValue(false);
    expect(body().text()).toContain('No tools selected: the file will have no tools line.');
  });
});

describe('SubAgentForm: editing an existing agent', () => {
  it('pre-fills every field and shows Save instead of Create', () => {
    mountForm({ editing: editing() });
    expect((nameInput().element as HTMLInputElement).value).toBe('Reviewer');
    expect((descInput().element as HTMLInputElement).value).toBe('Reviews code');
    expect((body().find('#agent-prompt').element as HTMLTextAreaElement).value).toBe('Be helpful.');
    expect(pressed('Blue')).toBe('true');
    expect(body().text()).toContain('2 of 12');
    expect(body().text()).toContain('Edit Agent');
    expect(saveButton().exists()).toBe(true);
  });

  it('writes back with the file stem as the original slug', async () => {
    mountForm({ editing: editing(), existingSlugs: ['reviewer'] });
    await saveButton().trigger('click');
    await flushPromises();
    expect(savedCalls[0].originalSlug).toBe('reviewer');
    expect(useToastsStore().current?.message).toBe('Updated Reviewer');
  });

  // Swift's `existingAgent?.model ?? "sonnet"` / `?.color ?? .orange` / `?.memory ?? .user`: saving
  // an agent that had none of them pinned a model, added a colour and switched memory ON.
  it('leaves an unset model, colour and memory unset instead of applying Swift\'s defaults', async () => {
    mountForm({ editing: editing({ model: null, color: null, memory: null }) });
    expect(body().findAll('[aria-labelledby="agent-model-label"] button').map((b) => b.attributes('aria-pressed'))).toEqual(['false', 'false', 'false']);
    expect(body().findAll('[aria-labelledby="agent-color-label"] button').every((b) => b.attributes('aria-pressed') === 'false')).toBe(true);
    expect(body().find('input[type="radio"][name="agent-memory"]:checked').exists()).toBe(false);
    expect(body().text()).toContain('Not set on this agent');
    await saveButton().trigger('click');
    await flushPromises();
    expect(savedCalls[0].input).toMatchObject({ model: null, color: null, memory: null });
  });

  it('keeps a model the form has no button for (shown, and sent back) until one is picked', async () => {
    mountForm({ editing: editing({ model: 'claude-opus-4-1' }) });
    expect(body().text()).toContain('Currently "claude-opus-4-1"');
    await saveButton().trigger('click');
    await flushPromises();
    expect(savedCalls[0].input.model).toBe('claude-opus-4-1');

    document.body.innerHTML = '';
    mountForm({ editing: editing({ model: 'claude-opus-4-1' }) });
    await body().find('[aria-labelledby="agent-model-label"]').findAll('button')[2].trigger('click'); // Haiku
    await saveButton().trigger('click');
    await flushPromises();
    expect(savedCalls[0].input.model).toBe('haiku');
  });
});

// Swift saves `knownTools.filter { selected.contains }`, so a tool outside the 12 (an MCP tool, say)
// was silently dropped on every edit.
describe('SubAgentForm: tools outside the known 12 are kept', () => {
  const withExtras = () => editing({ tools: ['mcp__github__create_issue', 'Read', 'Skill'] });

  it('lists them as chips, and sends them back after the known tools', async () => {
    mountForm({ editing: withExtras() });
    expect(body().text()).toContain('mcp__github__create_issue');
    expect(body().text()).toContain('Skill');
    await saveButton().trigger('click');
    await flushPromises();
    expect(savedCalls[0].input.tools).toEqual(['Read', 'mcp__github__create_issue', 'Skill']);
  });

  it('drops one only when it is explicitly removed', async () => {
    mountForm({ editing: withExtras() });
    await body().find('button[title="Remove Skill"]').trigger('click');
    await saveButton().trigger('click');
    await flushPromises();
    expect(savedCalls[0].input.tools).toEqual(['Read', 'mcp__github__create_issue']);
  });

  it('"All tools" off does not clear them', async () => {
    mountForm({ editing: withExtras() });
    await checkbox('All tools').setValue(true);
    await checkbox('All tools').setValue(false);
    await saveButton().trigger('click');
    await flushPromises();
    expect(savedCalls[0].input.tools).toEqual(['mcp__github__create_issue', 'Skill']);
  });

  it('shows no such section for an agent that has none', () => {
    mountForm({ editing: editing() });
    expect(body().text()).not.toContain('Other tools on this agent');
  });
});

describe('SubAgentForm: the file name', () => {
  it('shows a dash until there is a name', async () => {
    mountForm();
    expect(body().text()).toContain('—');
    await nameInput().setValue('Code Reviewer');
    expect(body().text()).toContain('~/.claude/agents/code-reviewer.md');
  });

  it('a name with no letters or numbers is invalid, with the reason, instead of failing in the writer', async () => {
    mountForm();
    await nameInput().setValue('!!!');
    await descInput().setValue('d');
    expect(createButton().attributes('disabled')).toBeDefined();
    expect(body().find('[role="alert"]').text()).toContain('at least one letter or number');
  });

  it('previews the file a clash would really get (reviewer-2), and says why', async () => {
    mountForm({ existingSlugs: ['reviewer'] });
    await nameInput().setValue('Reviewer');
    expect(body().text()).toContain('~/.claude/agents/reviewer-2.md');
    expect(body().text()).toContain('reviewer.md already exists');
  });

  it('says that a changed name renames the file, and sends the original slug so the old one is removed', async () => {
    mountForm({ editing: editing(), existingSlugs: ['reviewer'] });
    expect(body().text()).not.toContain('renames the file');
    await nameInput().setValue('Code Reviewer');
    expect(body().text()).toContain('~/.claude/agents/code-reviewer.md');
    expect(body().text()).toContain('Saving renames the file from reviewer.md');
    await saveButton().trigger('click');
    await flushPromises();
    expect(savedCalls[0].originalSlug).toBe('reviewer');
  });

  it('an edit that keeps its name does not claim a rename or a clash with itself', () => {
    mountForm({ editing: editing(), existingSlugs: ['reviewer'] });
    expect(body().text()).toContain('~/.claude/agents/reviewer.md');
    expect(body().text()).not.toContain('already exists');
  });
});

describe('SubAgentForm: a failed save', () => {
  it('shows the cause, keeps the form open with the input intact, and can be retried', async () => {
    const { onClose, onSaved } = mountForm({ outcome: 'error' });
    await fillValid();
    await createButton().trigger('click');
    await flushPromises();
    expect(useToastsStore().current).toMatchObject({ message: 'agents folder is read-only', style: 'error' });
    expect(onClose).not.toHaveBeenCalled();
    expect(onSaved).not.toHaveBeenCalled();
    expect((nameInput().element as HTMLInputElement).value).toBe('Code Reviewer');
    await createButton().trigger('click');
    await flushPromises();
    expect(savedCalls).toHaveLength(2);
  });
});

describe('SubAgentForm: keyboard (spec §9.2: Cancel = Esc / Mod+., Save = Return)', () => {
  it('focuses the dialog on open', () => {
    mountForm();
    expect(document.activeElement).toBe(body().find('[role="dialog"]').element);
  });

  it('Return in a text field saves when valid, and does nothing when not', async () => {
    mountForm();
    await nameInput().trigger('keydown', { key: 'Enter' });
    await flushPromises();
    expect(savedCalls).toHaveLength(0);
    await fillValid();
    await nameInput().trigger('keydown', { key: 'Enter' });
    await flushPromises();
    expect(savedCalls).toHaveLength(1);
  });

  it('Return in the prompt is a new line: it neither saves nor is default-prevented', async () => {
    mountForm();
    await fillValid();
    const event = new KeyboardEvent('keydown', { key: 'Enter', bubbles: true, cancelable: true });
    body().find('#agent-prompt').element.dispatchEvent(event);
    await flushPromises();
    expect(savedCalls).toHaveLength(0);
    expect(event.defaultPrevented).toBe(false);
  });

  it('Return on a focused button does not also save', async () => {
    mountForm();
    await fillValid();
    await body().find('button[title="Cancel"]').trigger('keydown', { key: 'Enter' });
    await flushPromises();
    expect(savedCalls).toHaveLength(0);
  });

  it('a held-down Return sends one save, not two', async () => {
    mountForm();
    await fillValid();
    void nameInput().trigger('keydown', { key: 'Enter' });
    void nameInput().trigger('keydown', { key: 'Enter' });
    await flushPromises();
    expect(savedCalls).toHaveLength(1);
  });

  it('Esc cancels', async () => {
    const { onClose } = mountForm();
    await body().find('.fixed.inset-0').trigger('keydown', { key: 'Escape' });
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('Ctrl+. cancels on Windows/Linux and Cmd+. alone does not', async () => {
    const { onClose } = mountForm({ os: 'windows' });
    await body().find('.fixed.inset-0').trigger('keydown', { key: '.', metaKey: true });
    expect(onClose).not.toHaveBeenCalled();
    await body().find('.fixed.inset-0').trigger('keydown', { key: '.', ctrlKey: true });
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('Cmd+. cancels on macOS and Ctrl+. alone does not', async () => {
    const { onClose } = mountForm({ os: 'macos' });
    await body().find('.fixed.inset-0').trigger('keydown', { key: '.', ctrlKey: true });
    expect(onClose).not.toHaveBeenCalled();
    await body().find('.fixed.inset-0').trigger('keydown', { key: '.', metaKey: true });
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  describe('keys the app-level sidebar navigation would also act on stay in the dialog', () => {
    let seen: string[] = [];
    const spy = (event: KeyboardEvent) => seen.push(event.key);
    beforeEach(() => {
      seen = [];
      document.addEventListener('keydown', spy);
    });
    afterEach(() => document.removeEventListener('keydown', spy));

    it.each([['Enter'], ['Escape'], ['ArrowDown'], ['ArrowUp']])('%s does not bubble out', async (key) => {
      mountForm();
      await nameInput().trigger('keydown', { key });
      await body().find('#agent-prompt').trigger('keydown', { key });
      expect(seen).toEqual([]);
    });

    it('ordinary typing and modifier shortcuts still do', async () => {
      mountForm();
      await nameInput().trigger('keydown', { key: 'a' });
      await nameInput().trigger('keydown', { key: '=', ctrlKey: true });
      expect(seen).toEqual(['a', '=']);
    });
  });
});

describe('SubAgentForm: dismissal', () => {
  it('Cancel closes', async () => {
    const { onClose } = mountForm();
    await body().find('button[title="Cancel"]').trigger('click');
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('a backdrop click closes but a click inside the dialog does not', async () => {
    const { onClose } = mountForm();
    await body().find('[role="dialog"]').trigger('click');
    expect(onClose).not.toHaveBeenCalled();
    await body().find('.fixed.inset-0').trigger('click');
    expect(onClose).toHaveBeenCalledTimes(1);
  });
});

describe('SubAgentForm: accessibility', () => {
  it('associates each text field with its label', () => {
    mountForm();
    for (const id of ['agent-name', 'agent-description', 'agent-prompt']) {
      expect(body().find(`label[for="${id}"]`).exists(), `label for #${id}`).toBe(true);
    }
  });

  it('exposes model, colour, tools and memory as labelled groups, and each colour by name', () => {
    mountForm();
    for (const id of ['agent-model-label', 'agent-color-label', 'agent-tools-label', 'agent-memory-label']) {
      expect(body().find(`[aria-labelledby="${id}"]`).exists(), id).toBe(true);
      expect(body().find(`#${id}`).exists(), id).toBe(true);
    }
    expect(body().findAll('[aria-labelledby="agent-color-label"] button').map((b) => b.attributes('aria-label'))).toEqual(['Orange', 'Red', 'Blue', 'Green', 'Purple', 'Teal']);
  });
});
