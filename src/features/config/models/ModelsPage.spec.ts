// M3-50, Models half: "Set as Default", "Set as Project Default" and "Clear Project Override".
// What matters most is the argument: the model's `id`, never its display name (port-plan C-27), since
// Swift's writing "Opus 5" into Claude Code's `model` setting is the defect this screen must not repeat.
import { flushPromises, mount } from '@vue/test-utils';
import { mockIPC } from '@tauri-apps/api/mocks';
import { createPinia, setActivePinia } from 'pinia';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import type { ClaudeModelInfo, ModelsView, ProjectContext } from '@/ipc/bindings';
import { useToastsStore } from '@/stores/toasts';
import { useConfigStore } from '../store';
import ModelsPage from './ModelsPage.vue';

const project: ProjectContext = { kind: 'project', projectId: 'p1' };

function model(displayName: string, id: string): ClaudeModelInfo {
  return { displayName, id, description: `About ${displayName}`, strengths: [] };
}

const OPUS = model('Opus 5', 'claude-opus-5');
const SONNET = model('Sonnet 5', 'claude-sonnet-5');
// A model a transcript recorded as `<synthetic>`: it names no known family, so it has no id to write.
const SYNTHETIC = model('<synthetic>', '');

const busy = () => {
  throw { kind: 'busy', message: 'settings.json is in use by another program', path: null };
};

let mounted: ReturnType<typeof mount>[] = [];

interface Seen {
  lists: unknown[];
  defaults: unknown[];
  projects: unknown[];
}

/** `views` is what each successive list_models returns (the last one repeats). */
async function mountPage(views: ModelsView[], options: { project?: ProjectContext | null; overrides?: Partial<Record<string, (args: unknown) => unknown>> } = {}) {
  setActivePinia(createPinia());
  if (options.project) useConfigStore().selectedProject = options.project;
  const seen: Seen = { lists: [], defaults: [], projects: [] };
  mockIPC((cmd, args) => {
    if (options.overrides?.[cmd]) return options.overrides[cmd]!(args);
    switch (cmd) {
      case 'list_models':
        seen.lists.push(args);
        return views[Math.min(seen.lists.length - 1, views.length - 1)];
      case 'set_default_model':
        seen.defaults.push(args);
        return 'written';
      case 'set_project_model':
        seen.projects.push(args);
        return 'written';
      default:
        throw new Error(`unexpected command: ${cmd}`);
    }
  });
  const wrapper = mount(ModelsPage, { attachTo: document.body });
  mounted.push(wrapper);
  await flushPromises();
  return { wrapper, seen, toasts: useToastsStore() };
}

function view(overrides: Partial<ModelsView> = {}): ModelsView {
  return { models: [OPUS, SONNET, SYNTHETIC], defaultModel: 'Opus 5', projectModel: null, ...overrides };
}

const buttonNames = (wrapper: ReturnType<typeof mount>) => wrapper.findAll('button[aria-label]').map((b) => b.attributes('aria-label'));

beforeEach(() => {
  document.body.innerHTML = '';
});

afterEach(() => {
  mounted.forEach((w) => w.unmount());
  mounted = [];
});

describe('ModelsPage: which buttons a card shows (ModelsListView.swift:304-356)', () => {
  it('without a project: "Set as Default" on every writable model but the default, and no project buttons', async () => {
    const { wrapper } = await mountPage([view()]);

    expect(buttonNames(wrapper)).toEqual(['Set as Default: Sonnet 5']);
    expect(wrapper.text()).toContain('first pick a project');
  });

  it('with a project: "Set as Project Default" on the others, "Clear Project Override" only on the project default', async () => {
    const { wrapper } = await mountPage([view({ projectModel: 'Sonnet 5' })], { project });

    expect(buttonNames(wrapper)).toEqual(['Set as Project Default: Opus 5', 'Set as Default: Sonnet 5', 'Clear Project Override: Sonnet 5']);
    expect(wrapper.text()).not.toContain('first pick a project');
  });

  it('offers nothing to set on a model that has no id', async () => {
    const { wrapper } = await mountPage([view()], { project });

    expect(buttonNames(wrapper).filter((n) => n?.includes('<synthetic>'))).toEqual([]);
  });

  it('still lets a project override be cleared when its model has no id', async () => {
    const { wrapper } = await mountPage([view({ projectModel: '<synthetic>' })], { project });

    expect(buttonNames(wrapper)).toContain('Clear Project Override: <synthetic>');
  });

  it('keeps the visible words in each button\'s accessible name, plus which model it is for', async () => {
    const { wrapper } = await mountPage([view()], { project });

    const withText = wrapper.findAll('button[aria-label]').map((b) => [b.text(), b.attributes('aria-label')]);
    for (const [visible, name] of withText) expect(name).toContain(visible);
  });
});

describe('ModelsPage: Set as Default', () => {
  it('writes the model\'s id, not its display name, says so, and moves the badge', async () => {
    const { wrapper, seen, toasts } = await mountPage([view({ defaultModel: 'Opus 5' }), view({ defaultModel: 'Sonnet 5' })]);
    expect(wrapper.text().match(/Global Default/g)).toHaveLength(1);

    await wrapper.find('button[aria-label="Set as Default: Sonnet 5"]').trigger('click');
    await flushPromises();

    expect(seen.defaults).toEqual([{ model: 'claude-sonnet-5' }]);
    expect(toasts.current).toMatchObject({ message: 'Set Sonnet 5 as default model', style: 'success' });
    expect(seen.lists).toHaveLength(2);
    // The view was re-read, so the default (and with it which card offers "Set as Default") moved.
    expect(wrapper.text().match(/Global Default/g)).toHaveLength(1);
    expect(buttonNames(wrapper)).toEqual(['Set as Default: Opus 5']);
  });

  it('shows the cause and writes nothing more when the write fails, and can be retried', async () => {
    let fail = true;
    const { wrapper, seen, toasts } = await mountPage([view()], { overrides: { set_default_model: () => (fail ? busy() : 'written') } });

    await wrapper.find('button[aria-label="Set as Default: Sonnet 5"]').trigger('click');
    await flushPromises();

    expect(toasts.current).toMatchObject({ message: 'settings.json is in use by another program', style: 'error' });
    expect(seen.lists).toHaveLength(1);

    fail = false;
    await wrapper.find('button[aria-label="Set as Default: Sonnet 5"]').trigger('click');
    await flushPromises();
    expect(seen.lists).toHaveLength(2);
  });

  it('disables every button while a write is in flight, so a second click cannot queue another', async () => {
    let release: (value: unknown) => void = () => {};
    let calls = 0;
    const { wrapper } = await mountPage([view()], {
      project,
      overrides: {
        set_default_model: () => {
          calls++;
          return new Promise((resolve) => (release = resolve));
        },
      },
    });

    await wrapper.find('button[aria-label="Set as Default: Sonnet 5"]').trigger('click');
    expect(wrapper.findAll('button[aria-label]').every((b) => b.attributes('disabled') !== undefined)).toBe(true);
    await wrapper.find('button[aria-label="Set as Default: Sonnet 5"]').trigger('click');

    release('written');
    await flushPromises();
    expect(calls).toBe(1);
  });
});

describe('ModelsPage: project default', () => {
  it('writes the id for the selected project, and says so', async () => {
    const { wrapper, seen, toasts } = await mountPage([view(), view({ projectModel: 'Sonnet 5' })], { project });

    await wrapper.find('button[aria-label="Set as Project Default: Sonnet 5"]').trigger('click');
    await flushPromises();

    expect(seen.projects).toEqual([{ project, model: 'claude-sonnet-5' }]);
    expect(toasts.current).toMatchObject({ message: 'Set Sonnet 5 as project default', style: 'success' });
    expect(buttonNames(wrapper)).toContain('Clear Project Override: Sonnet 5');
  });

  it('clears the override by sending a null model for the selected project', async () => {
    const { wrapper, seen, toasts } = await mountPage([view({ projectModel: 'Sonnet 5' }), view()], { project });

    await wrapper.find('button[aria-label="Clear Project Override: Sonnet 5"]').trigger('click');
    await flushPromises();

    expect(seen.projects).toEqual([{ project, model: null }]);
    expect(toasts.current).toMatchObject({ message: 'Cleared project model override' });
    expect(buttonNames(wrapper)).not.toContain('Clear Project Override: Sonnet 5');
  });

  it('shows the cause when the project write fails', async () => {
    const { wrapper, toasts } = await mountPage([view()], { project, overrides: { set_project_model: busy } });

    await wrapper.find('button[aria-label="Set as Project Default: Sonnet 5"]').trigger('click');
    await flushPromises();

    expect(toasts.current).toMatchObject({ message: 'settings.json is in use by another program', style: 'error' });
  });

  it('reads the models for the selected project', async () => {
    const { seen } = await mountPage([view()], { project });
    expect(seen.lists).toEqual([{ project }]);
  });
});

describe('ModelsPage: loading', () => {
  it('shows the cause when the models cannot be read', async () => {
    const { toasts } = await mountPage([view()], { overrides: { list_models: busy } });
    expect(toasts.current).toMatchObject({ message: 'settings.json is in use by another program', style: 'error' });
  });
});
