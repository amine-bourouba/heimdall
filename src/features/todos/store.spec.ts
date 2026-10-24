// M3-05's own AC: "Spec covers grouping by project and status counts; no string replaces `-`
// with `/` anywhere in the feature."
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { mockIPC } from '@tauri-apps/api/mocks';
import { flushPromises } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { describe, expect, it } from 'vitest';
import { events } from '@/ipc/bindings';
import type { TodoGroup_Serialize as TodoGroup, TodoItem_Serialize as TodoItem, TodoList_Serialize as TodoList } from '@/ipc/bindings';
import { useTodosStore } from './store';

function makeItem(overrides: Partial<TodoItem> = {}): TodoItem {
  return { content: 'do the thing', activeForm: 'Doing the thing', status: 'pending', ...overrides };
}

function makeList(overrides: Partial<TodoList> = {}): TodoList {
  return { sessionId: '11111111-1111-4111-8111-111111111111', agentId: null, items: [makeItem()], ...overrides };
}

function makeGroup(overrides: Partial<TodoGroup> = {}): TodoGroup {
  return { projectId: 'proj-1', projectName: 'my-project', lists: [makeList()], ...overrides };
}

// A literal, mechanical guard for the AC's own absence-of-pattern wording: `decodeProjectName`'s
// Swift equivalent (`dirName.replacingOccurrences(of: "-", with: "/")`,
// TodosOverviewView.swift:375) must never exist anywhere in this feature, not just "probably
// doesn't" -- `list_all_todos` already resolves real project names server-side, so there is
// never a reason to decode one from a path in this feature's own files.
describe('AC: no string replaces "-" with "/" anywhere in this feature', () => {
  it('store.ts and TodosPage.vue contain no "-"-to-"/" replace', () => {
    const files = ['store.ts', 'TodosPage.vue'];
    for (const file of files) {
      const content = readFileSync(fileURLToPath(new URL(file, import.meta.url)), 'utf-8');
      expect(content).not.toMatch(/replace(All)?\(\s*['"]-['"]/);
    }
  });
});

describe('todos store: loading and totals', () => {
  it('load() fetches every group via list_all_todos', async () => {
    setActivePinia(createPinia());
    mockIPC((cmd) => {
      expect(cmd).toBe('list_all_todos');
      return [makeGroup()];
    }, { shouldMockEvents: true });
    const store = useTodosStore();

    await store.load();

    expect(store.groups.length).toBe(1);
    expect(store.isLoading).toBe(false);
  });

  it('totalTodos/totalSessions sum across every group and list', async () => {
    setActivePinia(createPinia());
    mockIPC(
      () => [
        makeGroup({ projectId: 'p1', lists: [makeList({ sessionId: 's1', items: [makeItem(), makeItem()] })] }),
        makeGroup({ projectId: 'p2', lists: [makeList({ sessionId: 's2' }), makeList({ sessionId: 's3' })] }),
      ],
      { shouldMockEvents: true },
    );
    const store = useTodosStore();

    await store.load();

    expect(store.totalTodos).toBe(4); // 2 + 1 + 1
    expect(store.totalSessions).toBe(3); // s1, s2, s3
  });
});

describe('todos store: status counts (AC: "status counts")', () => {
  it('statusCountsFor tallies completed/inProgress/pending across every list in the group', () => {
    setActivePinia(createPinia());
    const store = useTodosStore();
    const lists: TodoList[] = [
      makeList({ items: [makeItem({ status: 'completed' }), makeItem({ status: 'pending' })] }),
      makeList({ items: [makeItem({ status: 'inProgress' }), makeItem({ status: 'completed' })] }),
    ];

    expect(store.statusCountsFor(lists)).toEqual({ completed: 2, inProgress: 1, pending: 1 });
  });
});

describe('todos store: grouping and sorting (AC: "grouping by project")', () => {
  it('sortedGroups puts a project with any incomplete todo before a fully-completed one', async () => {
    setActivePinia(createPinia());
    mockIPC(
      () => [
        makeGroup({ projectId: 'done', projectName: 'Zebra', lists: [makeList({ items: [makeItem({ status: 'completed' })] })] }),
        makeGroup({ projectId: 'active', projectName: 'Apple', lists: [makeList({ items: [makeItem({ status: 'pending' })] })] }),
      ],
      { shouldMockEvents: true },
    );
    const store = useTodosStore();
    await store.load();

    expect(store.sortedGroups.map((g) => g.projectId)).toEqual(['active', 'done']);
  });

  it('two equally-active (or equally-completed) projects sort alphabetically by name', async () => {
    setActivePinia(createPinia());
    mockIPC(
      () => [
        makeGroup({ projectId: 'z', projectName: 'Zebra', lists: [makeList({ items: [makeItem({ status: 'pending' })] })] }),
        makeGroup({ projectId: 'a', projectName: 'Apple', lists: [makeList({ items: [makeItem({ status: 'pending' })] })] }),
      ],
      { shouldMockEvents: true },
    );
    const store = useTodosStore();
    await store.load();

    expect(store.sortedGroups.map((g) => g.projectId)).toEqual(['a', 'z']);
  });

  it('a null projectName sorts without throwing (an orphaned/unresolvable project)', async () => {
    setActivePinia(createPinia());
    mockIPC(() => [makeGroup({ projectId: null, projectName: null })], { shouldMockEvents: true });
    const store = useTodosStore();

    await store.load();

    expect(store.sortedGroups.length).toBe(1);
  });
});

describe('todos store: search (AC: fuzzy over project name and todo content)', () => {
  it('matches on the project name', async () => {
    setActivePinia(createPinia());
    mockIPC(
      () => [makeGroup({ projectId: 'p1', projectName: 'deploy-tool' }), makeGroup({ projectId: 'p2', projectName: 'other' })],
      { shouldMockEvents: true },
    );
    const store = useTodosStore();
    await store.load();

    store.query = 'deploy';

    expect(store.filteredGroups.map((g) => g.projectId)).toEqual(['p1']);
  });

  it('matches on a todo item\'s content even when the project name does not match', async () => {
    setActivePinia(createPinia());
    mockIPC(
      () => [makeGroup({ projectId: 'p1', projectName: 'other', lists: [makeList({ items: [makeItem({ content: 'deploy the service' })] })] })],
      { shouldMockEvents: true },
    );
    const store = useTodosStore();
    await store.load();

    store.query = 'deploy';

    expect(store.filteredGroups.map((g) => g.projectId)).toEqual(['p1']);
  });
});

describe('todos store: refreshes on data-changed for the todos area', () => {
  it('a real data-changed event for "todos" triggers a reload', async () => {
    setActivePinia(createPinia());
    let version = 0;
    mockIPC(() => (version === 0 ? [] : [makeGroup()]), { shouldMockEvents: true });
    const store = useTodosStore();
    await store.load();
    expect(store.groups.length).toBe(0);

    version = 1;
    await events.dataChanged.emit([{ area: 'todos', key: null }]);
    await flushPromises();

    expect(store.groups.length).toBe(1);
  });
});
