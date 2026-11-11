// M3-49's own AC, page half: "built-in agents stay read-only". Plus the create / edit / duplicate /
// export / delete / import wiring the task's Goal names, each checked for the exact command and
// argument it sends (a wrong `ref` or slug here would write or delete the wrong file).
//
// SubAgentForm is a `<Teleport to="body">` root, so queries into it go through `document.body`
// (SubAgentForm.spec.ts / HookForm.spec.ts document why).
import { DOMWrapper, flushPromises, mount } from '@vue/test-utils';
import { mockIPC } from '@tauri-apps/api/mocks';
import { createPinia, setActivePinia } from 'pinia';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { ConfigItemDetail, ConfigItemSummary } from '@/ipc/bindings';
import { useToastsStore } from '@/stores/toasts';
import SubAgentsPage from './SubAgentsPage.vue';

function body() {
  return new DOMWrapper(document.body);
}

function summary(slug: string, overrides: Partial<ConfigItemSummary> = {}): ConfigItemSummary {
  return {
    ref: { kind: 'agent', project: null, slug },
    name: slug,
    description: '',
    scope: 'global',
    meta: {},
    ...overrides,
  };
}

const explore = summary('explore', { name: 'Explore', scope: 'builtIn', description: 'Fast read-only agent', meta: { tools: 'Glob,Grep,Read' } });
const reviewer = summary('reviewer', { name: 'Reviewer', description: 'Reviews code', meta: { model: 'sonnet', color: 'Blue', tools: 'Read,Grep', memory: 'user' } });
const planner = summary('planner', { name: 'Planner', description: 'Plans work', meta: { tools: 'Read' } });

const details: Record<string, ConfigItemDetail> = {
  explore: { ...explore, body: '', displayPath: '' },
  reviewer: { ...reviewer, body: 'Be helpful.', displayPath: '~/.claude/agents/reviewer.md' },
  planner: { ...planner, body: 'Plan things.', displayPath: '~/.claude/agents/planner.md' },
};

let mounted: ReturnType<typeof mount>[] = [];

interface Calls {
  list: number;
  get: unknown[];
  save: unknown[];
  delete: unknown[];
  export: unknown[];
  import: number;
}

async function mountPage(overrides: Partial<Record<string, (args: unknown) => unknown>> = {}, items: ConfigItemSummary[] = [explore, planner, reviewer]) {
  setActivePinia(createPinia());
  const calls: Calls = { list: 0, get: [], save: [], delete: [], export: [], import: 0 };
  mockIPC((cmd, args) => {
    if (overrides[cmd]) return overrides[cmd]!(args);
    switch (cmd) {
      case 'list_projects':
        return { value: [], skipped: 0 };
      case 'list_config_items':
        calls.list++;
        return { value: items, skipped: 0 };
      case 'get_config_item': {
        calls.get.push(args);
        return details[(args as { configRef: { slug: string } }).configRef.slug];
      }
      case 'save_agent':
        calls.save.push(args);
        return 'written';
      case 'delete_config_item':
        calls.delete.push(args);
        return 'written';
      case 'export_agent':
        calls.export.push(args);
        return 'written';
      case 'import_agent':
        calls.import++;
        return 'written';
    }
    throw new Error(`unexpected command: ${cmd}`);
  });
  const wrapper = mount(SubAgentsPage, { attachTo: document.body });
  mounted.push(wrapper);
  await flushPromises();
  return { wrapper, calls };
}

const busy = () => {
  throw { kind: 'busy', message: 'the agents folder is in use', path: null };
};

beforeEach(() => {
  document.body.innerHTML = '';
});

afterEach(() => {
  vi.unstubAllGlobals();
  mounted.forEach((w) => w.unmount());
  mounted = [];
});

describe('SubAgentsPage: built-in agents stay read-only (AC)', () => {
  it('offers a built-in only Duplicate: no Edit, Export or Delete', async () => {
    const { wrapper } = await mountPage({}, [explore]);
    expect(wrapper.find('button[title="Duplicate agent"]').exists()).toBe(true);
    for (const title of ['Edit agent', 'Export agent as JSON', 'Delete agent']) {
      expect(wrapper.find(`button[title="${title}"]`).exists(), title).toBe(false);
    }
  });

  it('offers a custom agent all four, and the name is the Edit button', async () => {
    const { wrapper } = await mountPage({}, [reviewer]);
    for (const title of ['Edit agent', 'Duplicate agent', 'Export agent as JSON', 'Delete agent']) {
      expect(wrapper.find(`button[title="${title}"]`).exists(), title).toBe(true);
    }
    expect(wrapper.find('button[title="Edit agent"]').text()).toBe('Reviewer');
  });

  it('counts the actions per kind across a mixed list (1 built-in, 2 custom)', async () => {
    const { wrapper } = await mountPage();
    expect(wrapper.findAll('button[title="Duplicate agent"]')).toHaveLength(3);
    expect(wrapper.findAll('button[title="Edit agent"]')).toHaveLength(2);
    expect(wrapper.findAll('button[title="Export agent as JSON"]')).toHaveLength(2);
    expect(wrapper.findAll('button[title="Delete agent"]')).toHaveLength(2);
  });
});

describe('SubAgentsPage: create and edit', () => {
  it('opens the form in create mode, and it knows which file names are taken', async () => {
    const { wrapper } = await mountPage();
    await wrapper.find('button[title="New Agent"]').trigger('click');
    expect(body().find('[role="dialog"]').text()).toContain('New Agent');
    await body().find('#agent-name').setValue('Reviewer');
    expect(body().text()).toContain('~/.claude/agents/reviewer-2.md');
  });

  it('does not count a built-in\'s id as a taken file name', async () => {
    const { wrapper } = await mountPage();
    await wrapper.find('button[title="New Agent"]').trigger('click');
    await body().find('#agent-name').setValue('Explore');
    expect(body().text()).toContain('~/.claude/agents/explore.md');
    expect(body().text()).not.toContain('already exists');
  });

  it('fetches the full agent (the summary has no prompt) and opens the form pre-filled', async () => {
    const { wrapper, calls } = await mountPage();
    await wrapper.findAll('button[title="Edit agent"]')[0].trigger('click'); // Planner (the API sorts custom agents by name)
    await flushPromises();
    expect(calls.get).toEqual([{ configRef: planner.ref }]);
    expect(body().find('[role="dialog"]').text()).toContain('Edit Agent');
    expect((body().find('#agent-prompt').element as HTMLTextAreaElement).value).toBe('Plan things.');
  });

  it('reloads the list after a save', async () => {
    const { wrapper, calls } = await mountPage();
    await wrapper.find('button[title="New Agent"]').trigger('click');
    await body().find('#agent-name').setValue('Fresh');
    await body().find('#agent-description').setValue('New one');
    await body().findAll('button').find((b) => b.text() === 'Create')!.trigger('click');
    await flushPromises();
    expect(calls.save).toHaveLength(1);
    expect(calls.list).toBe(2);
  });

  it('shows the cause, and opens no form, when the agent cannot be fetched for editing', async () => {
    const { wrapper } = await mountPage({ get_config_item: busy });
    await wrapper.findAll('button[title="Edit agent"]')[0].trigger('click');
    await flushPromises();
    expect(useToastsStore().current).toMatchObject({ message: 'the agents folder is in use', style: 'error' });
    expect(body().find('[role="dialog"]').exists()).toBe(false);
  });

  it('ignores a second click while the first is still fetching', async () => {
    const { wrapper, calls } = await mountPage();
    const edit = wrapper.findAll('button[title="Edit agent"]')[0];
    void edit.trigger('click');
    void edit.trigger('click');
    await flushPromises();
    expect(calls.get).toHaveLength(1);
  });
});

describe('SubAgentsPage: duplicate', () => {
  it('saves a custom agent as "<name> Copy" with its prompt, as a new file, then reloads', async () => {
    const { wrapper, calls } = await mountPage({}, [reviewer]);
    await wrapper.find('button[title="Duplicate agent"]').trigger('click');
    await flushPromises();
    expect(calls.save).toEqual([
      {
        input: { name: 'Reviewer Copy', icon: 'person.crop.circle.badge.plus', description: 'Reviews code', tools: ['Read', 'Grep'], model: 'sonnet', color: 'blue', prompt: 'Be helpful.', memory: 'user' },
        originalSlug: null,
      },
    ]);
    expect(calls.list).toBe(2);
    expect(useToastsStore().current?.message).toBe('Created Reviewer Copy');
  });

  it('can duplicate a built-in (that is how one becomes editable), with no prompt', async () => {
    const { wrapper, calls } = await mountPage({}, [explore]);
    await wrapper.find('button[title="Duplicate agent"]').trigger('click');
    await flushPromises();
    expect(calls.get).toEqual([{ configRef: explore.ref }]);
    expect((calls.save[0] as { input: { name: string; prompt: unknown; tools: string[] } }).input).toMatchObject({ name: 'Explore Copy', prompt: null, tools: ['Glob', 'Grep', 'Read'] });
  });

  it('shows the cause when the copy cannot be saved, and does not reload', async () => {
    const { wrapper, calls } = await mountPage({ save_agent: busy }, [reviewer]);
    await wrapper.find('button[title="Duplicate agent"]').trigger('click');
    await flushPromises();
    expect(useToastsStore().current).toMatchObject({ message: 'the agents folder is in use', style: 'error' });
    expect(calls.list).toBe(1);
  });
});

describe('SubAgentsPage: delete', () => {
  it('confirms, then deletes exactly that agent by its ref, and reloads', async () => {
    const confirmSpy = vi.fn().mockReturnValue(true);
    vi.stubGlobal('confirm', confirmSpy);
    const { wrapper, calls } = await mountPage();
    await wrapper.findAll('button[title="Delete agent"]')[1].trigger('click'); // Reviewer
    await flushPromises();
    expect(confirmSpy.mock.calls[0][0]).toContain('"Reviewer"');
    expect(calls.delete).toEqual([{ configRef: reviewer.ref }]);
    expect(calls.list).toBe(2);
    expect(useToastsStore().current).toMatchObject({ message: 'Deleted Reviewer', style: 'info' });
  });

  it('does nothing when the confirmation is declined', async () => {
    vi.stubGlobal('confirm', vi.fn().mockReturnValue(false));
    const { wrapper, calls } = await mountPage();
    await wrapper.findAll('button[title="Delete agent"]')[0].trigger('click');
    await flushPromises();
    expect(calls.delete).toEqual([]);
  });

  it('shows the cause when the delete fails', async () => {
    vi.stubGlobal('confirm', vi.fn().mockReturnValue(true));
    const { wrapper } = await mountPage({ delete_config_item: busy });
    await wrapper.findAll('button[title="Delete agent"]')[0].trigger('click');
    await flushPromises();
    expect(useToastsStore().current).toMatchObject({ message: 'the agents folder is in use', style: 'error' });
  });
});

describe('SubAgentsPage: export and import', () => {
  it('exports by the file stem and says so', async () => {
    const { wrapper, calls } = await mountPage({}, [reviewer]);
    await wrapper.find('button[title="Export agent as JSON"]').trigger('click');
    await flushPromises();
    expect(calls.export).toEqual([{ slug: 'reviewer' }]);
    expect(useToastsStore().current?.message).toBe('Exported Reviewer');
  });

  it('stays quiet when the save dialog was cancelled', async () => {
    const { wrapper } = await mountPage({ export_agent: () => 'cancelled' }, [reviewer]);
    await wrapper.find('button[title="Export agent as JSON"]').trigger('click');
    await flushPromises();
    expect(useToastsStore().current).toBeNull();
  });

  it('imports through the Rust-opened dialog, then reloads', async () => {
    const { wrapper, calls } = await mountPage();
    await wrapper.find('button[title="Import Agent from JSON"]').trigger('click');
    await flushPromises();
    expect(calls.import).toBe(1);
    expect(calls.list).toBe(2);
    expect(useToastsStore().current?.message).toBe('Imported agent');
  });

  it('shows the importer\'s own reason when the JSON is invalid (Swift showed a bare "Invalid agent JSON")', async () => {
    const { wrapper, calls } = await mountPage({
      import_agent: () => {
        throw { kind: 'invalidInput', message: 'agent JSON needs a "name"', path: null };
      },
    });
    await wrapper.find('button[title="Import Agent from JSON"]').trigger('click');
    await flushPromises();
    expect(useToastsStore().current).toMatchObject({ message: 'agent JSON needs a "name"', style: 'error' });
    expect(calls.list).toBe(1);
  });

  it('shows the cause when an export fails', async () => {
    const { wrapper } = await mountPage({ export_agent: busy }, [reviewer]);
    await wrapper.find('button[title="Export agent as JSON"]').trigger('click');
    await flushPromises();
    expect(useToastsStore().current).toMatchObject({ message: 'the agents folder is in use', style: 'error' });
  });
});

describe('SubAgentsPage: loading', () => {
  it('shows the cause and the empty state, not a skeleton forever, when the list fails to load', async () => {
    const { wrapper } = await mountPage({ list_config_items: busy });
    expect(useToastsStore().current).toMatchObject({ message: 'the agents folder is in use', style: 'error' });
    expect(wrapper.text()).toContain('No agents match');
  });
});
