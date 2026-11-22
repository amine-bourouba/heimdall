// Open in the user's editor, open a terminal, reveal in the file manager (M3-55), written once for the
// four places that offer them (tool blocks, the session toolbar, config detail, the MCP screen).
//
// Every command takes an id and resolves the path itself (ADR 0010 rule 5): nothing here ever holds
// a path. What this adds is *availability* (M3-55 AC2: "Buttons are hidden for editors or terminals
// not available on the running OS"): the preferred editor or terminal is only offered if it exists on
// this OS (`launchOptions`, the same tables the settings pickers use), so a saved `xcode` on Windows or
// `iterm2` on Linux shows no button rather than one that can only fail.
import { computed } from 'vue';
import { ipc } from '@/ipc/client';
import { EDITOR_LABELS, TERMINAL_LABELS, editorOptions, fileManagerName, terminalOptions } from '@/lib/launchOptions';
import { usePlatformStore } from '@/stores/platform';
import { usePrefsStore } from '@/stores/prefs';
import { useToastsStore } from '@/stores/toasts';
import type { EditorTarget, RevealTarget } from '@/ipc/bindings';

export function useLauncher() {
  const prefsStore = usePrefsStore();
  const platformStore = usePlatformStore();
  const toastsStore = useToastsStore();

  /** The preferred editor's name, or `null` when it does not exist on this OS (so no button). Swift's default is VS Code. */
  const editorName = computed<string | null>(() => {
    const editor = prefsStore.prefs?.textEditor ?? 'vscode';
    return editorOptions(platformStore.os).includes(editor) ? EDITOR_LABELS[editor] : null;
  });

  /** The preferred terminal's name, or `null` when it does not exist on this OS (or the OS is not known yet). */
  const terminalName = computed<string | null>(() => {
    const terminal = prefsStore.prefs?.preferredTerminal;
    return terminal && terminalOptions(platformStore.os).includes(terminal) ? TERMINAL_LABELS[terminal] : null;
  });

  /** `openTerminalOnBash`: copying a command also opens the terminal, but only a terminal that is available here. */
  const opensTerminalOnCopy = computed(() => !!prefsStore.prefs?.openTerminalOnBash && terminalName.value !== null);

  const fileManager = computed(() => fileManagerName(platformStore.os));

  async function run(action: () => Promise<unknown>, failure: string) {
    try {
      await action();
    } catch (error) {
      toastsStore.showError(error, failure);
    }
  }

  const openInEditor = (target: EditorTarget) => run(() => ipc.openInEditor(target), `Could not open ${editorName.value ?? 'the editor'}`);

  /** With a session, Rust also copies `claude --resume <id>`; without one it only opens the terminal at the project. */
  const openTerminal = (projectId: string, sessionId: string | null) => run(() => ipc.openTerminal(projectId, sessionId), `Could not open ${terminalName.value ?? 'the terminal'}`);

  const reveal = (target: RevealTarget) => run(() => ipc.revealInFileManager(target), `Could not show it in the ${fileManager.value}`);

  return { editorName, terminalName, opensTerminalOnCopy, fileManager, openInEditor, openTerminal, reveal };
}
