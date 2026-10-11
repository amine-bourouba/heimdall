// Shapes `fixtures/dto/` (already real, Rust-generated DTOs -- `fixtures/README.md`: "Generated,
// not hand-written... regenerates it byte-identically from heimdall-core's DTOs") into the IPC
// responses `mockIpc.ts` answers with, and exposes the one custom Playwright fixture every mocked
// spec in `e2e/mocked/` actually needs: a `Page` that already believes it's running in Tauri.
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { test as base, expect } from '@playwright/test';
import { installMockIpc } from './mockIpc';

const FIXTURES_DIR = fileURLToPath(new URL('../../fixtures/dto', import.meta.url));

function readJson<T>(name: string): T {
  return JSON.parse(readFileSync(`${FIXTURES_DIR}/${name}`, 'utf-8')) as T;
}

interface ProjectSummaryLike {
  id: string;
  [key: string]: unknown;
}
interface SessionSummaryLike {
  projectId: string;
  [key: string]: unknown;
}

export interface MockFixtures {
  appInfo: { version: string; identifier: string; os: 'windows' | 'linux' | 'macos'; arch: string; claudeHomeFound: boolean };
  prefs: Record<string, unknown>;
  sidebarCounts: Record<string, number>;
  projects: ProjectSummaryLike[];
  sessionsByProject: Record<string, SessionSummaryLike[]>;
}

/** The current OS, the one `get_app_info` field a real Tauri build derives natively and a mock
 * has to guess some other way -- `process.platform`, matching what this harness itself runs on. */
function currentOs(): 'windows' | 'linux' | 'macos' {
  if (process.platform === 'win32') return 'windows';
  if (process.platform === 'darwin') return 'macos';
  return 'linux';
}

export function buildFixtures(): MockFixtures {
  const projects = readJson<ProjectSummaryLike[]>('projects.json');
  const sessions = readJson<SessionSummaryLike[]>('sessions-poirot.json');
  const sessionsByProject: Record<string, SessionSummaryLike[]> = {};
  for (const session of sessions) {
    (sessionsByProject[session.projectId] ??= []).push(session);
  }

  return {
    appInfo: { version: '0.0.0-e2e', identifier: 'io.github.amine-bourouba.heimdall.e2e', os: currentOs(), arch: 'x86_64', claudeHomeFound: true },
    prefs: {
      schemaVersion: 1,
      showMenuBarIcon: true,
      hasCompletedOnboarding: true,
      colorTheme: 'default',
      fontScale: 1.0,
      sessionLayout: 'grid',
      configProjectPath: null,
      configLayouts: {},
      showAgentSessions: false,
      textEditor: 'vscode',
      preferredTerminal: 'terminal',
      openTerminalOnBash: false,
      claudeCodePath: null,
      showAnimations: true,
      wrapCodeLines: true,
      autoExpandBlocks: true,
      parseMarkdownInResults: true,
      globalSearchMode: 'fuzzy',
      customMemoryFolders: [],
    },
    sidebarCounts: { todos: 0, history: 0, commands: 0, skills: 0, plans: 0, memory: 0, mcpServers: 0, models: 0, hooks: 0, subAgents: 0, plugins: 0, outputStyles: 0 },
    projects,
    sessionsByProject,
  };
}

export const test = base.extend<{ mockedPage: import('@playwright/test').Page }>({
  mockedPage: async ({ page }, use) => {
    await installMockIpc(page, buildFixtures());
    await use(page);
  },
});

export { expect };
