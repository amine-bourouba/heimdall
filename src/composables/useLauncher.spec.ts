// M3-55: what is offered on which OS, and what each action sends. The commands take ids, never paths
// (ADR 0010), so the assertions are on the exact target; the availability rules are M3-55's AC2.
import { mockIPC } from '@tauri-apps/api/mocks';
import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it } from 'vitest';
import type { Prefs, PreferredEditor, PreferredTerminal, TargetOs } from '@/ipc/bindings';
import { usePlatformStore } from '@/stores/platform';
import { usePrefsStore } from '@/stores/prefs';
import { useToastsStore } from '@/stores/toasts';
import { useLauncher } from './useLauncher';

function prefs(overrides: Partial<Prefs> = {}): Prefs {
  return {
    schemaVersion: 1, showMenuBarIcon: true, hasCompletedOnboarding: true, colorTheme: 'default', fontScale: 1, sessionLayout: 'grid',
    configProjectPath: null, configLayouts: {}, showAgentSessions: false, textEditor: 'vscode', preferredTerminal: 'terminal',
    openTerminalOnBash: false, claudeCodePath: null, showAnimations: true, wrapCodeLines: true, autoExpandBlocks: true,
    parseMarkdownInResults: true, globalSearchMode: 'fuzzy', customMemoryFolders: [], ...overrides,
  };
}

let sent: [string, unknown][];

function setUp(os: TargetOs | null, overrides: Partial<Prefs> = {}, failure?: { message: string }) {
  setActivePinia(createPinia());
  usePlatformStore().os = os;
  usePrefsStore().prefs = prefs(overrides);
  sent = [];
  mockIPC((cmd, args) => {
    sent.push([cmd, args]);
    if (failure) throw { kind: 'notFound', message: failure.message, path: null };
    return null;
  });
  return useLauncher();
}

beforeEach(() => {
  sent = [];
});

describe('the editor is offered only where it exists', () => {
  it.each<[TargetOs, PreferredEditor, string | null]>([
    ['macos', 'xcode', 'Xcode'],
    ['windows', 'xcode', null],
    ['linux', 'xcode', null],
    ['windows', 'vscode', 'VS Code'],
    ['linux', 'cursor', 'Cursor'],
    ['macos', 'zed', 'Zed'],
  ])('on %s a saved %s gives %s', (os, editor, name) => {
    expect(setUp(os, { textEditor: editor }).editorName.value).toBe(name);
  });

  it('defaults to VS Code, as Swift does, before the preferences are known', () => {
    setActivePinia(createPinia());
    usePlatformStore().os = 'linux';
    expect(useLauncher().editorName.value).toBe('VS Code');
  });
});

describe('the terminal is offered only where it exists', () => {
  it.each<[TargetOs, PreferredTerminal, string | null]>([
    ['macos', 'iterm2', 'iTerm2'],
    ['windows', 'iterm2', null],
    ['linux', 'iterm2', null],
    ['windows', 'windowsTerminal', 'Windows Terminal'],
    ['macos', 'windowsTerminal', null],
    ['linux', 'system', 'System default'],
    ['linux', 'kitty', 'Kitty'],
    ['windows', 'kitty', null],
  ])('on %s a saved %s gives %s', (os, terminal, name) => {
    expect(setUp(os, { preferredTerminal: terminal }).terminalName.value).toBe(name);
  });

  it('is not offered until the OS is known, rather than guessed', () => {
    expect(setUp(null).terminalName.value).toBeNull();
  });

  it('is not offered when the preferences are not known', () => {
    setActivePinia(createPinia());
    usePlatformStore().os = 'macos';
    expect(useLauncher().terminalName.value).toBeNull();
  });
});

describe('opening a terminal when a command is copied (openTerminalOnBash)', () => {
  it('needs the preference on and a terminal that exists here', () => {
    expect(setUp('macos', { openTerminalOnBash: true }).opensTerminalOnCopy.value).toBe(true);
    expect(setUp('macos', { openTerminalOnBash: false }).opensTerminalOnCopy.value).toBe(false);
    expect(setUp('linux', { openTerminalOnBash: true, preferredTerminal: 'iterm2' }).opensTerminalOnCopy.value).toBe(false);
  });
});

describe('the file manager is named for the OS', () => {
  it.each<[TargetOs, string]>([
    ['macos', 'Finder'],
    ['windows', 'File Explorer'],
    ['linux', 'file manager'],
  ])('%s: %s', (os, name) => {
    expect(setUp(os).fileManager.value).toBe(name);
  });
});

describe('the actions send ids, never paths', () => {
  it('opens a tool block by project, session and block id', async () => {
    const launcher = setUp('windows');
    await launcher.openInEditor({ kind: 'toolBlock', projectId: 'p1', sessionId: 's1', blockId: 'm1:2', line: null });
    expect(sent).toEqual([['open_in_editor', { target: { kind: 'toolBlock', projectId: 'p1', sessionId: 's1', blockId: 'm1:2', line: null } }]]);
  });

  it('opens a terminal at a project, with or without a session to resume', async () => {
    const launcher = setUp('macos');
    await launcher.openTerminal('p1', 's1');
    await launcher.openTerminal('p1', null);
    expect(sent).toEqual([
      ['open_terminal', { projectId: 'p1', sessionId: 's1' }],
      ['open_terminal', { projectId: 'p1', sessionId: null }],
    ]);
  });

  it('reveals by id', async () => {
    const launcher = setUp('linux');
    await launcher.reveal({ kind: 'session', projectId: 'p1', sessionId: 's1' });
    await launcher.reveal({ kind: 'mcpConfigFile' });
    expect(sent.map(([cmd, args]) => [cmd, (args as { target: { kind: string } }).target.kind])).toEqual([
      ['reveal_in_file_manager', 'session'],
      ['reveal_in_file_manager', 'mcpConfigFile'],
    ]);
  });
});

describe('a failure is shown, with its cause', () => {
  it('shows what Rust said when the editor cannot be opened', async () => {
    const launcher = setUp('windows', {}, { message: 'the `code` command is not on your PATH' });
    await launcher.openInEditor({ kind: 'mcpConfig', serverName: 'gh' });
    expect(useToastsStore().current).toMatchObject({ message: 'the `code` command is not on your PATH', style: 'error' });
  });

  it.each([
    ['openTerminal', (l: ReturnType<typeof useLauncher>) => l.openTerminal('p1', null)],
    ['reveal', (l: ReturnType<typeof useLauncher>) => l.reveal({ kind: 'mcpConfigFile' })],
  ])('%s shows its cause too', async (_name, act) => {
    await act(setUp('macos', {}, { message: 'no such project' }));
    expect(useToastsStore().current).toMatchObject({ message: 'no such project', style: 'error' });
  });
});
