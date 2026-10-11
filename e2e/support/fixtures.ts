// Shapes `fixtures/dto/` (already real, Rust-generated DTOs -- `fixtures/README.md`: "Generated,
// not hand-written... regenerates it byte-identically from heimdall-core's DTOs") into the IPC
// responses `mockIpc.ts` answers with, and exposes the one custom Playwright fixture every mocked
// spec in `e2e/mocked/` actually needs: a `Page` that already believes it's running in Tauri.
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { test as base, expect } from '@playwright/test';
import { installMockIpc } from './mockIpc';
import { allToolsBlocks, DIFFS, THINKING } from '../visual/visualData';
import { joinedMaliciousContent } from './maliciousPayloads';
import { generateLargeTranscriptMessages } from '../../scripts/gen-large-transcript.mjs';

export const VISUAL_PROJECT_ID = 'visual-project';
/** Stable session ids `e2e/visual/session.spec.ts` navigates to, one per scenario group -- kept
 * separate (not one giant session) because a few Swift test groups need the *same* block type in
 * different `isExpanded`/text states side by side, which only works as separate messages, each
 * individually click-to-expand-able, not as one shared `autoExpandBlocks` setting could give. */
export const VISUAL_SESSION_ID = 'visual-session'; // all tools + thinking (expanded) -- conversation/tool-block/search/filter scenarios
export const VISUAL_THINKING_SESSION_ID = 'visual-thinking'; // 4 messages, one per ThinkingBlocks variant
export const VISUAL_DIFFS_SESSION_ID = 'visual-diffs'; // 4 messages, one per EditDiff variant
export const VISUAL_EMPTY_SESSION_ID = 'visual-empty'; // SessionDetail's own "no messages" state

function sessionSummary(id: string, overrides: Partial<Record<string, unknown>> = {}) {
  return {
    id,
    projectId: VISUAL_PROJECT_ID,
    title: 'Visual regression fixture session',
    preview: null,
    startedAt: '2026-02-20T14:00:00Z',
    endedAt: '2026-02-20T14:05:00Z',
    model: 'claude-opus-4-6',
    tokens: { input: 0, output: 0, cacheRead: 0, cacheCreation: 0 },
    turnCount: 1,
    isSidechain: false,
    parentSessionId: null,
    agentType: null,
    agentDescription: null,
    hasDebugLog: false,
    hasFileHistory: false,
    hasTodos: false,
    ...overrides,
  };
}

function sessionDetail(summary: ReturnType<typeof sessionSummary>, messages: unknown[], overrides: Partial<Record<string, unknown>> = {}) {
  return {
    detail: {
      summary,
      messageCount: messages.length,
      context: null,
      toolNames: [],
      skipped: 0,
      ...overrides,
    },
    messages,
  };
}

export function buildVisualSessionDetails(): MockFixtures['sessionDetails'] {
  const blocks = allToolsBlocks();
  const toolNames = blocks.filter((b): b is typeof b & { kind: 'toolUse'; name: string } => b.kind === 'toolUse').map((b) => b.name);
  const allToolsMessages = [
    { id: 'm0', role: 'user', timestamp: '2026-02-20T14:00:00Z', model: null, usage: null, blocks: [{ kind: 'text', id: 'b0', text: 'Can you check the build, review the code, and think through the approach?', system: null }] },
    { id: 'm1', role: 'assistant', timestamp: '2026-02-20T14:00:05Z', model: 'claude-opus-4-6', usage: { input: 1200, output: 400, cacheRead: 300, cacheCreation: 100 }, blocks: [{ kind: 'thinking', id: 'b1', text: THINKING.expanded }, ...blocks] },
  ];

  const thinkingMessages = Object.entries(THINKING).map(([key, text], i) => ({
    id: `think-${key}`,
    role: 'assistant',
    timestamp: `2026-02-20T14:0${i}:00Z`,
    model: 'claude-opus-4-6',
    usage: null,
    blocks: [{ kind: 'thinking', id: `think-block-${key}`, text }],
  }));

  const diffMessages = Object.entries(DIFFS).map(([key, diff], i) => ({
    id: `diff-${key}`,
    role: 'assistant',
    timestamp: `2026-02-20T14:0${i}:00Z`,
    model: 'claude-opus-4-6',
    usage: null,
    blocks: [{ kind: 'toolUse', id: `diff-tool-${key}`, toolUseId: `diff-tool-${key}`, name: 'Edit', inputPretty: '{}', command: null, filePath: diff.filePath, edit: { oldString: diff.oldString, newString: diff.newString } }],
  }));

  return {
    // Each session gets a distinct title -- the first draft reused one literal title for all
    // four, and every `getByText('...title...')` locator broke with a real strict-mode
    // violation (8 elements: the sidebar row, the grid card, and their own sub-strings, for all
    // four sessions at once) the moment more than one was loaded into the same fixture project.
    [VISUAL_SESSION_ID]: sessionDetail(sessionSummary(VISUAL_SESSION_ID, { title: 'Visual regression: all tools', preview: 'Can you check the build, review the code, and think through the approach?', tokens: { input: 1200, output: 400, cacheRead: 300, cacheCreation: 100 }, turnCount: 2 }), allToolsMessages, {
      context: { current: 2000, peak: 2000, window: 200000, inputTokens: 1200, outputTokens: 400, cacheReadTokens: 300, cacheCreationTokens: 100, series: [800, 1400, 2000] },
      toolNames,
    }),
    [VISUAL_THINKING_SESSION_ID]: sessionDetail(sessionSummary(VISUAL_THINKING_SESSION_ID, { title: 'Visual regression: thinking variants' }), thinkingMessages),
    [VISUAL_DIFFS_SESSION_ID]: sessionDetail(sessionSummary(VISUAL_DIFFS_SESSION_ID, { title: 'Visual regression: edit diffs' }), diffMessages, { toolNames: ['Edit'] }),
    [VISUAL_EMPTY_SESSION_ID]: sessionDetail(sessionSummary(VISUAL_EMPTY_SESSION_ID, { title: 'Visual regression: empty session' }), []),
  };
}

export const MALICIOUS_PROJECT_ID = 'malicious-project';
export const MALICIOUS_SESSION_ID = 'malicious-session';

/** `fixtures/security/malicious-claude-home/`'s own session, rebuilt here in Heimdall's DTO
 * shape (not raw Claude JSONL) for the mocked-WebKit half of M2-26's two run modes -- the real
 * `~/.claude` tree is what the real-app Linux/Windows run loads instead. Both pull the identical
 * corpus from the same round-robin split (`maliciousPayloads.ts`), so "payloads sit in user text,
 * assistant text, tool output, thinking" (the four locations with a real, built view today --
 * `TextBlock`/`ToolBlock` through `MarkdownView`, `ThinkingBlock` through plain interpolation) is
 * true of this mocked session too, not just the real-app fixture tree. The plan and memory file
 * buckets have no UI to click through at all yet (`PlansPage.vue`/`src/features/memory` are
 * M2-06-era stubs) -- only the real-app fixture tree carries those two today.
 */
export function buildMaliciousSessionDetails(): MockFixtures['sessionDetails'] {
  const toolUseId = 'malicious-tool-use';
  const messages = [
    { id: 'm0', role: 'user', timestamp: '2026-09-20T10:00:00Z', model: null, usage: null, blocks: [{ kind: 'text', id: 'm0b0', text: joinedMaliciousContent('user'), system: null }] },
    {
      id: 'm1',
      role: 'assistant',
      timestamp: '2026-09-20T10:00:05Z',
      model: 'claude-opus-4-6',
      usage: { input: 100, output: 50, cacheRead: 0, cacheCreation: 0 },
      blocks: [
        { kind: 'thinking', id: 'm1b0', text: joinedMaliciousContent('thinking') },
        { kind: 'text', id: 'm1b1', text: joinedMaliciousContent('assistant'), system: null },
        // `Read`, not `Bash`: `ToolBlock.vue`'s own `showRawContent` starts `true` for every tool
        // and there's no toggle to leave raw text at all for Bash (`v-if="!isBash()"`) -- tool
        // output only ever reaches `MarkdownView`'s real v-html/DOMPurify path for a non-Bash
        // tool, and only once "Show rendered markdown" is actually clicked.
        { kind: 'toolUse', id: 'm1b2', toolUseId, name: 'Read', inputPretty: '{}', command: null, filePath: '/Users/fixture/malicious-project/suspicious-file.txt', edit: null },
        { kind: 'toolResult', id: 'm1b3', toolUseId, content: joinedMaliciousContent('tool-output'), isError: false },
      ],
    },
  ];

  return {
    [MALICIOUS_SESSION_ID]: sessionDetail(sessionSummary(MALICIOUS_SESSION_ID, { projectId: MALICIOUS_PROJECT_ID, title: 'Malicious transcript fixture', preview: joinedMaliciousContent('user'), turnCount: 2 }), messages),
  };
}

/** `buildFixtures()` plus the malicious-transcript session, mirroring `buildVisualFixtures()`'s
 * own "inject a synthetic project with real session-summary rows" shape for the same reason:
 * `sessionStore.selectSession` resolves the summary from `projects.value` before ever calling
 * `get_session`. */
export function buildMaliciousFixtures(): MockFixtures {
  const fixtures = buildFixtures();
  fixtures.sessionDetails = buildMaliciousSessionDetails();
  const summaries = Object.values(fixtures.sessionDetails).map((s) => (s.detail as { summary: SessionSummaryLike }).summary);
  fixtures.projects = [...fixtures.projects, { id: MALICIOUS_PROJECT_ID, name: 'malicious-fixtures', path: '/malicious-fixtures', pathSource: 'transcriptCwd', sessionCount: summaries.length, lastActivity: '2026-09-20T10:00:05Z', hasMemory: false }];
  fixtures.sessionsByProject[MALICIOUS_PROJECT_ID] = summaries;
  return fixtures;
}

export const PERF_PROJECT_ID = 'perf-project';
export const PERF_SESSION_ID = 'perf-session';

/** `buildFixtures()` plus one session of 10,000+ blocks (`scripts/gen-large-transcript.mjs`,
 * M2-28), for `e2e/perf/transcript-scroll.spec.ts`. `detail.messageCount` is set to the *full*
 * generated message count up front, so the virtualizer's own real pagination
 * (`sessionStore.loadMoreMessages`, 20 messages/page) runs exactly as it does for a real session
 * of this size -- scrolling through the lazy-loading itself is part of what the scroll-smoothness
 * budget is measuring, not something to fixture around. */
export function buildPerfFixtures(minBlockCount = 10_000): MockFixtures {
  const fixtures = buildFixtures();
  const messages = generateLargeTranscriptMessages(minBlockCount);
  const summary = sessionSummary(PERF_SESSION_ID, { projectId: PERF_PROJECT_ID, title: 'Performance fixture: 10k+ blocks', turnCount: messages.length });
  fixtures.sessionDetails = { [PERF_SESSION_ID]: sessionDetail(summary, messages) };
  fixtures.projects = [...fixtures.projects, { id: PERF_PROJECT_ID, name: 'perf-fixtures', path: '/perf-fixtures', pathSource: 'transcriptCwd', sessionCount: 1, lastActivity: '2026-01-01T00:00:00Z', hasMemory: false }];
  fixtures.sessionsByProject[PERF_PROJECT_ID] = [summary];
  return fixtures;
}

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
  /** Keyed by session id -- `get_session`/`get_session_messages` answer from here when the
   * requested id matches. `e2e/visual/session.spec.ts` is the one real consumer (a `SessionSummary`
   * with real `Message[]` content needs more than `fixtures/dto/`'s own list-row-only shape). */
  sessionDetails: Record<string, { detail: unknown; messages: unknown[] }>;
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
    sessionDetails: {},
  };
}

/** `buildFixtures()` plus the visual-regression sessions (AC2's conversation view, and every
 * other "full session" scenario `e2e/visual/session.spec.ts` needs) -- kept separate from the
 * mocked-smoke-test default so that spec's own fixtures stay exactly what AC1 needs, no more.
 * `sessionStore.selectSession` looks the summary up in `projects.value` *before* ever calling
 * `get_session` (confirmed directly in `store.ts`) -- `visual-project` has to be a real project
 * with real session-summary rows, or selection resolves to `null` without `mockIpc.ts`'s own
 * `get_session`/`get_session_messages` handlers ever being reached at all. */
export function buildVisualFixtures(): MockFixtures {
  const fixtures = buildFixtures();
  fixtures.sessionDetails = buildVisualSessionDetails();
  const summaries = Object.values(fixtures.sessionDetails).map((s) => (s.detail as { summary: SessionSummaryLike }).summary);
  fixtures.projects = [...fixtures.projects, { id: VISUAL_PROJECT_ID, name: 'visual-fixtures', path: '/visual-fixtures', pathSource: 'transcriptCwd', sessionCount: summaries.length, lastActivity: '2026-02-20T14:05:00Z', hasMemory: false }];
  fixtures.sessionsByProject[VISUAL_PROJECT_ID] = summaries;
  return fixtures;
}

export const test = base.extend<{ mockedPage: import('@playwright/test').Page }>({
  mockedPage: async ({ page }, use) => {
    await installMockIpc(page, buildFixtures());
    await use(page);
  },
});

export { expect };
