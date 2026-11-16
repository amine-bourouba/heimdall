// M3-51: delete a history entry, and clear history older than N days. The confirmation is Rust's native
// one, so the key assertions are what happens to each answer, above all that `cancelled` changes nothing
// (the AC): the entry stays, the list is not re-read, and nothing is said.
import { flushPromises, mount } from '@vue/test-utils';
import { mockIPC } from '@tauri-apps/api/mocks';
import { createPinia, setActivePinia } from 'pinia';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { HistoryEntry, HistoryPage as HistoryPageData } from '@/ipc/bindings';
import { useToastsStore } from '@/stores/toasts';
import HistoryPage from './HistoryPage.vue';

vi.mock('@tauri-apps/plugin-clipboard-manager', () => ({ writeText: vi.fn().mockResolvedValue(undefined) }));

function entry(id: string, display: string): HistoryEntry {
  return { id, display, pastedContents: {}, timestamp: new Date().toISOString(), project: '/Users/test/project' };
}

const page = (entries: HistoryEntry[]): HistoryPageData => ({ entries, offset: 0, total: entries.length, skipped: 0 });
const THREE = [entry('a', 'first prompt'), entry('b', 'second prompt'), entry('c', 'third prompt')];

const busy = () => {
  throw { kind: 'busy', message: 'history.jsonl is in use by another program', path: null };
};

type Answer = 'written' | 'cancelled' | 'unchanged' | 'error' | 'hold';

let mounted: ReturnType<typeof mount>[] = [];
let sent: [string, unknown][];
let toasts: ReturnType<typeof useToastsStore>;
let release: (value: unknown) => void = () => {};

/** `lists`: what each successive list_history returns (the last one repeats). */
async function mountPage(answer: Answer, lists: HistoryPageData[] = [page(THREE)]) {
  setActivePinia(createPinia());
  toasts = useToastsStore();
  sent = [];
  let listCalls = 0;
  mockIPC((cmd, args) => {
    sent.push([cmd, args]);
    if (cmd === 'list_history') return lists[Math.min(listCalls++, lists.length - 1)];
    if (cmd === 'delete_history_entry' || cmd === 'clear_history_older_than') {
      if (answer === 'error') return busy();
      if (answer === 'hold') return new Promise((resolve) => (release = resolve));
      return answer;
    }
    throw new Error(`unexpected command: ${cmd}`);
  });
  const wrapper = mount(HistoryPage, { attachTo: document.body });
  mounted.push(wrapper);
  await flushPromises();
  return wrapper;
}

const shown = (wrapper: ReturnType<typeof mount>) => wrapper.findAll('button[title="Delete Prompt"]').map((b) => b.attributes('aria-label'));
const listCalls = () => sent.filter(([cmd]) => cmd === 'list_history').length;

beforeEach(() => {
  document.body.innerHTML = '';
});

afterEach(() => {
  mounted.forEach((w) => w.unmount());
  mounted = [];
});

describe('HistoryPage: deleting one prompt', () => {
  it('gives each prompt a delete button that says which prompt', async () => {
    const wrapper = await mountPage('written');
    expect(shown(wrapper)).toEqual(['Delete prompt: first prompt', 'Delete prompt: second prompt', 'Delete prompt: third prompt']);
  });

  it('sends that entry\'s id, removes only it, and says so', async () => {
    const wrapper = await mountPage('written');
    await wrapper.find('button[aria-label="Delete prompt: second prompt"]').trigger('click');
    await flushPromises();
    expect(sent).toContainEqual(['delete_history_entry', { entryId: 'b' }]);
    expect(shown(wrapper)).toEqual(['Delete prompt: first prompt', 'Delete prompt: third prompt']);
    expect(toasts.current).toMatchObject({ message: 'Prompt deleted', style: 'info' });
  });

  it('changes nothing and says nothing when the native confirmation is declined (AC)', async () => {
    const wrapper = await mountPage('cancelled');
    await wrapper.find('button[aria-label="Delete prompt: second prompt"]').trigger('click');
    await flushPromises();
    expect(shown(wrapper)).toHaveLength(3);
    expect(toasts.current).toBeNull();
    expect(wrapper.text()).toContain('3 prompts');
  });

  it('removes a prompt that was already gone, and says it was', async () => {
    const wrapper = await mountPage('unchanged');
    await wrapper.find('button[aria-label="Delete prompt: first prompt"]').trigger('click');
    await flushPromises();
    expect(shown(wrapper)).toHaveLength(2);
    expect(toasts.current).toMatchObject({ message: 'That prompt was already gone' });
  });

  it('shows the cause and keeps the prompt when the delete fails', async () => {
    const wrapper = await mountPage('error');
    await wrapper.find('button[aria-label="Delete prompt: first prompt"]').trigger('click');
    await flushPromises();
    expect(shown(wrapper)).toHaveLength(3);
    expect(toasts.current).toMatchObject({ message: 'history.jsonl is in use by another program', style: 'error' });
  });

  it('disables the button while the confirmation is open, so there is one command', async () => {
    const wrapper = await mountPage('hold');
    const button = () => wrapper.find('button[aria-label="Delete prompt: first prompt"]');
    await button().trigger('click');
    expect(button().attributes('disabled')).toBeDefined();
    await button().trigger('click');
    release('written');
    await flushPromises();
    expect(sent.filter(([cmd]) => cmd === 'delete_history_entry')).toHaveLength(1);
  });
});

describe('HistoryPage: clearing old prompts', () => {
  const toggle = (wrapper: ReturnType<typeof mount>) => wrapper.find('button[aria-label="Clear old prompts"]');
  const items = (wrapper: ReturnType<typeof mount>) => wrapper.findAll('[role="menuitem"]');
  const open = async (wrapper: ReturnType<typeof mount>) => {
    await toggle(wrapper).trigger('click');
    await flushPromises();
  };

  it('is disabled when there is nothing to clear', async () => {
    const wrapper = await mountPage('written', [page([])]);
    expect(toggle(wrapper).attributes('disabled')).toBeDefined();
  });

  it('offers the four ages from the Swift menu, and focuses the first', async () => {
    const wrapper = await mountPage('written');
    await open(wrapper);
    expect(toggle(wrapper).attributes('aria-expanded')).toBe('true');
    expect(items(wrapper).map((i) => i.text())).toEqual(['Older than 2 weeks', 'Older than 1 month', 'Older than 3 months', 'Older than 6 months']);
    expect(document.activeElement).toBe(items(wrapper)[0].element);
  });

  it.each([
    ['Older than 2 weeks', 14],
    ['Older than 1 month', 30],
    ['Older than 3 months', 90],
    ['Older than 6 months', 180],
  ])('"%s" sends %i days', async (label, days) => {
    const wrapper = await mountPage('cancelled');
    await open(wrapper);
    await items(wrapper).find((i) => i.text() === label)!.trigger('click');
    await flushPromises();
    expect(sent).toContainEqual(['clear_history_older_than', { days }]);
  });

  it('reads the list again and says what it did when something was deleted', async () => {
    const wrapper = await mountPage('written', [page(THREE), page([THREE[0]])]);
    await open(wrapper);
    await items(wrapper)[1].trigger('click');
    await flushPromises();
    expect(shown(wrapper)).toEqual(['Delete prompt: first prompt']);
    expect(toasts.current).toMatchObject({ message: 'Deleted prompts older than 1 month', style: 'info' });
    expect(wrapper.find('[role="menu"]').exists()).toBe(false);
  });

  it('changes nothing, reads nothing and says nothing when the native confirmation is declined (AC)', async () => {
    const wrapper = await mountPage('cancelled');
    await open(wrapper);
    await items(wrapper)[0].trigger('click');
    await flushPromises();
    expect(shown(wrapper)).toHaveLength(3);
    expect(listCalls()).toBe(1);
    expect(toasts.current).toBeNull();
  });

  it('says so when nothing was that old', async () => {
    const wrapper = await mountPage('unchanged');
    await open(wrapper);
    await items(wrapper)[3].trigger('click');
    await flushPromises();
    expect(toasts.current).toMatchObject({ message: 'No prompts older than 6 months', style: 'info' });
    expect(listCalls()).toBe(1);
  });

  it('shows the cause when the clear fails', async () => {
    const wrapper = await mountPage('error');
    await open(wrapper);
    await items(wrapper)[0].trigger('click');
    await flushPromises();
    expect(toasts.current).toMatchObject({ message: 'history.jsonl is in use by another program', style: 'error' });
    expect(shown(wrapper)).toHaveLength(3);
  });

  describe('the menu itself', () => {
    it('stays open while focus moves from the toggle to one of its items, and closes when focus leaves it', async () => {
      const wrapper = await mountPage('written');
      await open(wrapper);
      toggle(wrapper).element.dispatchEvent(new FocusEvent('focusout', { bubbles: true, relatedTarget: items(wrapper)[1].element }));
      await flushPromises();
      expect(wrapper.find('[role="menu"]').exists()).toBe(true);
      items(wrapper)[1].element.dispatchEvent(new FocusEvent('focusout', { bubbles: true, relatedTarget: document.body }));
      await flushPromises();
      expect(wrapper.find('[role="menu"]').exists()).toBe(false);
    });

    it('moves between items with the arrow keys, wrapping, without the sidebar seeing them', async () => {
      const seen: string[] = [];
      const spy = (event: KeyboardEvent) => seen.push(event.key);
      document.addEventListener('keydown', spy);
      const wrapper = await mountPage('written');
      await open(wrapper);
      const press = (key: string) => items(wrapper)[0].element.dispatchEvent(new KeyboardEvent('keydown', { key, bubbles: true, cancelable: true }));
      press('ArrowDown');
      expect(document.activeElement).toBe(items(wrapper)[1].element);
      (document.activeElement as HTMLElement).dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowUp', bubbles: true, cancelable: true }));
      (document.activeElement as HTMLElement).dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowUp', bubbles: true, cancelable: true }));
      expect(document.activeElement).toBe(items(wrapper)[3].element);
      expect(seen).toEqual([]);
      document.removeEventListener('keydown', spy);
    });

    it('closes on Escape, returns to the toggle, and the app\'s own Escape does not also run', async () => {
      const seen: string[] = [];
      const spy = (event: KeyboardEvent) => seen.push(event.key);
      document.addEventListener('keydown', spy);
      const wrapper = await mountPage('written');
      await open(wrapper);
      items(wrapper)[0].element.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }));
      await flushPromises();
      expect(wrapper.find('[role="menu"]').exists()).toBe(false);
      expect(document.activeElement).toBe(toggle(wrapper).element);
      expect(seen).toEqual([]);
      document.removeEventListener('keydown', spy);
    });
  });
});
