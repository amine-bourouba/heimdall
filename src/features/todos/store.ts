// Ported from TodosOverviewView.swift (M3-05), adapted to a real command that does not exist on
// the Swift side: `list_all_todos` (M1-42) already groups by *project*, with `projectName`
// resolved server-side the same way `list_projects` resolves it -- unlike
// `TodosOverviewView.swift`'s own per-session grouping, which falls back to
// `dirName.replacingOccurrences(of: "-", with: "/")` when `sessions-index.json` has no
// `originalPath` (`findSessionOnDisk`). This task's own AC ("no string replaces `-` with `/`
// anywhere in the feature") is automatically satisfied by using that command at all: there is
// nothing left to decode once Rust has already resolved every project name.
//
// `TodoList` carries only a `sessionId`, no session title (unlike the Swift reference app's own
// `sessionTitle(for:)` via an in-memory project search) -- not in this task's own Goal/AC, which
// names only project names, so the session id is shown as-is rather than reaching into a
// different feature's store for a title it was never asked to resolve.
import { computed, ref } from 'vue';
import { defineStore } from 'pinia';
import { ipc, onDataChanged } from '@/ipc/client';
// `listAllTodos()` only ever returns the `_Serialize` shape (camelCase `status`, confirmed
// directly against its own generated signature) -- the plain `TodoGroup`/`TodoList`/`TodoItem`
// aliases are a union with `_Deserialize` (snake_case, Rust's own on-disk read shape) for
// wherever Rust code needs to handle either; using the union here instead of the concrete
// wire type is what caused `item.status` to also admit `"in_progress"`, a value this command
// never actually sends.
import type { TodoGroup_Serialize as TodoGroup, TodoList_Serialize as TodoList } from '@/ipc/bindings';
import { fuzzyMatch } from '@/lib/fuzzyMatch';

export interface StatusCounts {
  completed: number;
  inProgress: number;
  pending: number;
}

function countStatuses(lists: TodoList[]): StatusCounts {
  const counts: StatusCounts = { completed: 0, inProgress: 0, pending: 0 };
  for (const list of lists) {
    for (const item of list.items) counts[item.status] += 1;
  }
  return counts;
}

function hasIncomplete(lists: TodoList[]): boolean {
  return lists.some((list) => list.items.some((item) => item.status !== 'completed'));
}

export const useTodosStore = defineStore('todos', () => {
  const groups = ref<TodoGroup[]>([]);
  const isLoading = ref(true);
  const query = ref('');
  let subscribed = false;

  const totalTodos = computed(() => groups.value.reduce((sum, g) => sum + g.lists.reduce((s, l) => s + l.items.length, 0), 0));
  const totalSessions = computed(() => groups.value.reduce((sum, g) => sum + g.lists.length, 0));

  // Ported from `loadTodos`'s own sort (TodosOverviewView.swift:286-291): active projects (any
  // non-completed todo, across any of their sessions) before fully-completed ones, then
  // alphabetical -- adapted from Swift's per-session version of the same rule to this task's own
  // per-project grouping.
  const sortedGroups = computed<TodoGroup[]>(() =>
    [...groups.value].sort((a, b) => {
      const aActive = hasIncomplete(a.lists);
      const bActive = hasIncomplete(b.lists);
      if (aActive !== bActive) return aActive ? -1 : 1;
      return (a.projectName ?? '').localeCompare(b.projectName ?? '', undefined, { sensitivity: 'base' });
    }),
  );

  // Ported from `filteredEntries` (TodosOverviewView.swift:46-62): best of the project-name score
  // and the highest-scoring todo content in that project, sorted by score when a query is active.
  const filteredGroups = computed<TodoGroup[]>(() => {
    const q = query.value.trim();
    if (q === '') return sortedGroups.value;
    return sortedGroups.value
      .map((group) => {
        const nameScore = fuzzyMatch(group.projectName ?? '', q)?.score ?? 0;
        const contentScore = group.lists
          .flatMap((list) => list.items)
          .reduce((best, item) => Math.max(best, fuzzyMatch(item.content, q)?.score ?? 0), 0);
        return { group, score: Math.max(nameScore, contentScore) };
      })
      .filter((r) => r.score > 0)
      .sort((a, b) => b.score - a.score)
      .map((r) => r.group);
  });

  function statusCountsFor(lists: TodoList[]): StatusCounts {
    return countStatuses(lists);
  }

  async function load() {
    isLoading.value = true;
    groups.value = await ipc.listAllTodos();
    isLoading.value = false;

    if (!subscribed) {
      subscribed = true;
      await onDataChanged(['todos'], () => void load());
    }
  }

  return { groups, isLoading, query, totalTodos, totalSessions, sortedGroups, filteredGroups, statusCountsFor, load };
});
