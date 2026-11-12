// Create-from-template and delete for commands, skills and output styles (M3-50), written once so
// the three screens behave identically: the same toasts, the same confirmation, the same error
// handling. (Swift repeats it per screen; its delete lives in the detail toolbar, `ConfigToolbarDelete`.)
//
// Create is always global, as Swift's `createCommandTemplate()`/`createSkillTemplate()`/
// `createOutputStyleTemplate()` take no project: a stray file is harmless in `~/.claude`, whereas one
// dropped into a repository the user only meant to look at is not. Swift then opens the new file in
// the user's editor; that is M3-55, so the toast names the file instead.
//
// Delete asks through the webview's `window.confirm`, the decision recorded in the progress log
// ("hook and sub-agent deletes keep `window.confirm` for now"): it is the one place on these screens
// that does, so moving the confirmation into Rust later is a change to `confirmDelete` alone.
import { ref } from 'vue';
import { ipc } from '@/ipc/client';
import { useToastsStore } from '@/stores/toasts';
import type { ConfigRef } from '@/ipc/bindings';

export type TemplateKind = 'command' | 'skill' | 'outputStyle';

const NOUN: Record<TemplateKind, string> = { command: 'command', skill: 'skill', outputStyle: 'output style' };

export function useConfigWriteActions(kind: TemplateKind) {
  const toasts = useToastsStore();
  const noun = NOUN[kind];
  // A second click while a call is in flight must not create a second template or send a second delete.
  const isBusy = ref(false);

  /** Resolves `true` once a template exists (the caller reloads its list), `false` if nothing was created. */
  async function createFromTemplate(): Promise<boolean> {
    if (isBusy.value) return false;
    isBusy.value = true;
    try {
      const created = await ipc.createConfigTemplate(kind, null);
      toasts.show(`Created new ${noun} template: ${created.slug}`);
      return true;
    } catch (error) {
      toasts.showError(error, `Could not create the ${noun} template`);
      return false;
    } finally {
      isBusy.value = false;
    }
  }

  function confirmDelete(name: string): boolean {
    // `delete_config_item` removes a skill's whole folder (its SKILL.md and anything beside it).
    const what =
      kind === 'skill'
        ? `Delete the skill "${name}"? Its whole folder is removed, and this cannot be undone.`
        : `Delete the ${noun} "${name}"? The file is removed, and this cannot be undone.`;
    return window.confirm(what);
  }

  /** Resolves `true` once the item is gone (the caller leaves the detail view and reloads). */
  async function deleteItem(configRef: ConfigRef, name: string): Promise<boolean> {
    if (isBusy.value || !confirmDelete(name)) return false;
    isBusy.value = true;
    try {
      const outcome = await ipc.deleteConfigItem(configRef);
      toasts.show(outcome === 'written' ? `Deleted ${name}` : `${name} was already gone`, { style: 'info' });
      return true;
    } catch (error) {
      toasts.showError(error, `Could not delete the ${noun}`);
      return false;
    } finally {
      isBusy.value = false;
    }
  }

  return { isBusy, createFromTemplate, deleteItem };
}
