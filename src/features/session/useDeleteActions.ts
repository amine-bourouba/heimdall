// Delete a session or a project (M3-51), written once for the three places that offer it (the session
// card, the project row, the session toolbar) so they behave and speak alike.
//
// The confirmation is Rust's: `delete_session` / `delete_project` show a native dialog naming what goes,
// and answer `cancelled` if it is declined (M3-46). A `cancelled` answer means nothing was deleted, so the
// session store changes nothing (see its `deleteSession`) and this says nothing either. Swift has a
// session-delete control only in the session toolbar (SessionToolbarContent.swift:218-246) and no control
// for deleting a project at all (`AppState.deleteProject` has no caller).
import { ref } from 'vue';
import type { WriteOutcome } from '@/ipc/bindings';
import { useToastsStore } from '@/stores/toasts';
import { useSessionStore } from './store';

export function useDeleteActions() {
  const sessionStore = useSessionStore();
  const toastsStore = useToastsStore();
  // A second click while the native confirmation or the delete is still open must not send a second command.
  const isBusy = ref(false);

  async function run(action: () => Promise<WriteOutcome>, done: string, gone: string, failure: string) {
    if (isBusy.value) return;
    isBusy.value = true;
    try {
      const outcome = await action();
      if (outcome === 'written') toastsStore.show(done, { style: 'info' });
      else if (outcome === 'unchanged') toastsStore.show(gone, { style: 'info' });
    } catch (error) {
      toastsStore.showError(error, failure);
    } finally {
      isBusy.value = false;
    }
  }

  function deleteSession(sessionId: string) {
    return run(() => sessionStore.deleteSession(sessionId), 'Session deleted', 'That session was already gone', 'Could not delete the session');
  }

  function deleteProject(project: { id: string; name: string }) {
    return run(() => sessionStore.deleteProject(project.id), `Deleted ${project.name}`, `${project.name} was already gone`, 'Could not delete the project');
  }

  return { isBusy, deleteSession, deleteProject };
}
