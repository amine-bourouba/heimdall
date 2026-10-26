// AC1: "Each listed Swift screenshot suite has a matching Playwright test." Ported from
// PoirotTests/Screenshots/ScreenshotTests_{History,Plans,Memory,Todos,DebugLog,Facets,
// FileHistory,ConfigScreens}.swift (29 of 30 Swift tests, docs/spec/test-carryover.md --
// `testHistoryGrid` dropped, see the History describe block below for why), plus three
// Heimdall-only additions this task's own Goal names with no Swift screenshot-test counterpart at
// all (search, onboarding, settings) -- each of those three screens was still a stub placeholder
// when `e2e/visual/session.spec.ts` (M2-24) was written, confirmed directly in that file's own
// closing comment ("Settings/Onboarding/Help... each still a stub route"); M3-16/M3-17 have since
// made all three real, and this task is what actually closes that gap.
//
// Where a Swift suite has a "component-only" test (`mainContent`/fixed `CGSize`) and a
// "composite app" test (`fullApp`) of the *same* view with no state difference, the two are
// folded into one Playwright test here -- Playwright always screenshots inside a real running app
// shell, so there is no separate "bare component" render to distinguish, the same reasoning
// session.spec.ts's own header comment already applies to its Sidebar/SessionDetail groups.
//
// Baselines are Linux-only (port-plan.md P-13) and committed from a real Linux CI run, never
// generated on this dev machine -- see session.spec.ts's own header comment for why.
import { test as base, expect } from '@playwright/test';
import { installMockIpc } from '../support/mockIpc';
import { buildFixtures } from '../support/fixtures';
import type { MockFixtures } from '../support/fixtures';

const SCREENSHOT_OPTS = { animations: 'disabled' as const };

/** Patches `invoke` a second time (the exact pattern session.spec.ts's own `testSidebarLoading`/
 * `testSessionSkeleton` already use), to answer the commands `mockIpc.ts`'s own fixed switch does
 * not cover -- every command this file's screens need beyond what the base mock already answers
 * (`get_prefs`, `list_projects`, `get_session`, ...). A canned response per command, not a
 * function: `page.addInitScript(fn, arg)` serialises `arg` as JSON, so a closure could never
 * survive the trip anyway, and no test here needs more than one fixed answer per command. */
function installExtraResponses(responses: Record<string, unknown>) {
  const win = window as unknown as { __TAURI_INTERNALS__: { invoke: (cmd: string, args?: unknown) => Promise<unknown> } };
  const real = win.__TAURI_INTERNALS__.invoke;
  win.__TAURI_INTERNALS__.invoke = (cmd: string, args?: unknown) => (cmd in responses ? Promise.resolve(responses[cmd]) : real(cmd, args));
}

async function gotoWithExtra(page: import('@playwright/test').Page, path: string, extra: Record<string, unknown>, fixtures: MockFixtures = buildFixtures()) {
  await installMockIpc(page, fixtures);
  await page.addInitScript(installExtraResponses, extra);
  await page.goto(path);
}

const test = base.extend<{ mockedPage: import('@playwright/test').Page }>({
  mockedPage: async ({ page }, use) => {
    await installMockIpc(page, buildFixtures());
    await use(page);
  },
});

// ScreenshotTests_History has a 3rd test, `testHistoryGrid` (sets `state.configLayouts[.history]
// = .grid` and expects a different render) -- dropped, not folded or faked: `HistoryPage.vue`'s
// own header comment (M3-02) states directly that "the project-filter picker, grid/list toggle...
// are not in this task's own Outputs/AC list -- left for a later pass if ever asked for", and no
// later task in docs/tasks/ adds one either (confirmed by grepping). Setting
// `prefs.configLayouts` has zero effect on this page today, so a `testHistoryGrid` test here would
// just assert a pixel-identical duplicate of `testHistory` forever -- a real behaviour gap to
// revisit if/when that toggle is ever built, not something to paper over with an inert fixture.
test.describe('History (ScreenshotTests_History: testHistoryList + testHistoryFullApp folded into testHistory; testHistoryGrid dropped, see comment above)', () => {
  test('testHistory', async ({ page }) => {
    await gotoWithExtra(page, '/#/history', {
      list_history: {
        entries: [
          { id: 'h0', display: 'Refactor the auth middleware to use JWT', pastedContents: {}, timestamp: '2026-02-20T14:00:00Z', project: '/Users/fixture/heimdall' },
          { id: 'h1', display: 'Add a visual regression test for the sidebar', pastedContents: {}, timestamp: '2026-02-19T09:30:00Z', project: '/Users/fixture/heimdall' },
          { id: 'h2', display: 'Fix the flaky CI job on Windows', pastedContents: {}, timestamp: '2026-01-15T11:00:00Z', project: '/Users/fixture/heimdall' },
        ],
        offset: 0,
        total: 3,
        skipped: 0,
      },
    });
    await expect(page.getByRole('heading', { name: 'History' })).toBeVisible();
    await expect(page.getByText('Refactor the auth middleware')).toBeVisible();
    await expect(page).toHaveScreenshot(SCREENSHOT_OPTS);
  });
});

test.describe('Plans (ScreenshotTests_Plans: testPlansList + testPlansFullApp folded into testPlans)', () => {
  test('testPlans', async ({ page }) => {
    await gotoWithExtra(page, '/#/plans', {
      list_config_items: {
        value: [
          { ref: { kind: 'plan', project: null, slug: 'refactor-auth' }, name: 'Refactor Auth', description: 'Migrate session auth from cookies to JWT', scope: 'global', meta: {} },
          { ref: { kind: 'plan', project: null, slug: 'visual-regression' }, name: 'Visual Regression Coverage', description: 'Add Playwright baselines for the remaining MVP screens', scope: 'project', meta: {} },
        ],
        skipped: 0,
      },
    });
    await expect(page.getByRole('heading', { name: 'Plans' })).toBeVisible();
    await expect(page.getByText('Refactor Auth')).toBeVisible();
    await expect(page).toHaveScreenshot(SCREENSHOT_OPTS);
  });
});

test.describe('Memory (ScreenshotTests_Memory: testMemoryList + testMemoryFullApp folded into testMemory)', () => {
  test('testMemory', async ({ page }) => {
    await gotoWithExtra(page, '/#/memory', {
      list_memory_sources: [{ id: 'project:heimdall', label: 'heimdall', kind: 'project', fileCount: 2 }],
      list_memory_files: [
        { id: 'mem0', sourceId: 'project:heimdall', name: 'Memory', filename: 'CLAUDE.md', isMain: true },
        { id: 'mem1', sourceId: 'project:heimdall', name: 'Testing Conventions', filename: 'testing-conventions.md', isMain: false },
      ],
    });
    await expect(page.getByRole('heading', { name: 'Memory' })).toBeVisible();
    await expect(page.getByText('CLAUDE.md')).toBeVisible();
    await expect(page).toHaveScreenshot(SCREENSHOT_OPTS);
  });
});

test.describe('Todos (ScreenshotTests_Todos: testTodosList + testTodosFullApp folded into testTodos; Solarized/HighContrast kept separate, genuinely different renders)', () => {
  const TODOS_RESPONSE = {
    list_all_todos: [
      {
        projectId: 'proj-heimdall',
        projectName: 'heimdall',
        lists: [
          {
            sessionId: 'session-0',
            agentId: null,
            items: [
              { content: 'Write the visual baseline spec', activeForm: 'Writing the visual baseline spec', status: 'inProgress' },
              { content: 'Run the local verification sequence', activeForm: 'Running the local verification sequence', status: 'pending' },
              { content: 'Read the Swift screenshot tests', activeForm: 'Reading the Swift screenshot tests', status: 'completed' },
            ],
          },
        ],
      },
    ],
  };

  test('testTodos', async ({ page }) => {
    await gotoWithExtra(page, '/#/todos', TODOS_RESPONSE);
    await expect(page.getByRole('heading', { name: 'TODOs' })).toBeVisible();
    await expect(page.getByText('Write the visual baseline spec')).toBeVisible();
    await expect(page).toHaveScreenshot(SCREENSHOT_OPTS);
  });

  for (const colorTheme of ['solarized', 'highContrast'] as const) {
    test(`testTodos${colorTheme === 'solarized' ? 'Solarized' : 'HighContrast'}`, async ({ page }) => {
      const fixtures = buildFixtures();
      fixtures.prefs = { ...fixtures.prefs, colorTheme };
      await gotoWithExtra(page, '/#/todos', TODOS_RESPONSE, fixtures);
      await expect(page.getByRole('heading', { name: 'TODOs' })).toBeVisible();
      await expect(page).toHaveScreenshot(SCREENSHOT_OPTS);
    });
  }
});

// A dedicated project+session for the three session-detail-hosted screens below (Debug Log,
// Facets, File History) -- each needs `hasDebugLog`/`hasFileHistory` true on the session summary
// to even show its own toolbar button, and `buildVisualFixtures()`'s own sessions (session.spec.ts's
// fixture set) default both false, so a minimal fixture is built here instead of there.
const MVP_PROJECT_ID = 'mvp-screens-project';
const MVP_SESSION_ID = 'mvp-screens-session';

function sessionFixtures(overrides: Partial<Record<string, unknown>> = {}) {
  const fixtures = buildFixtures();
  const summary = {
    id: MVP_SESSION_ID,
    projectId: MVP_PROJECT_ID,
    title: 'Visual baseline fixture session',
    preview: 'Can you check the build?',
    startedAt: '2026-02-20T14:00:00Z',
    endedAt: '2026-02-20T14:05:00Z',
    model: 'claude-opus-4-6',
    tokens: { input: 100, output: 50, cacheRead: 0, cacheCreation: 0 },
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
  const messages = [{ id: 'm0', role: 'user', timestamp: '2026-02-20T14:00:00Z', model: null, usage: null, blocks: [{ kind: 'text', id: 'b0', text: 'Can you check the build?', system: null }] }];
  fixtures.sessionDetails[MVP_SESSION_ID] = { detail: { summary, messageCount: messages.length, context: null, toolNames: [], skipped: 0 }, messages };
  fixtures.projects = [...fixtures.projects, { id: MVP_PROJECT_ID, name: 'mvp-screens-fixtures', path: '/mvp-screens-fixtures', pathSource: 'transcriptCwd', sessionCount: 1, lastActivity: summary.endedAt, hasMemory: false }];
  fixtures.sessionsByProject[MVP_PROJECT_ID] = [summary];
  return fixtures;
}

test.describe('Debug Log (ScreenshotTests_DebugLog: testDebugLogView)', () => {
  test('testDebugLogView', async ({ page }) => {
    const fixtures = sessionFixtures({ hasDebugLog: true });
    await gotoWithExtra(
      page,
      `/#/sessions/${MVP_PROJECT_ID}/${MVP_SESSION_ID}`,
      {
        get_debug_log_page: {
          entries: [
            { id: 0, timestamp: '2026-02-06T16:01:34.872Z', level: 'debug', message: 'Loading MCP servers from ~/.claude.json' },
            { id: 1, timestamp: '2026-02-06T16:01:35.456Z', level: 'warn', message: 'MCP server "notion" slow to respond (> 500ms)' },
            { id: 2, timestamp: '2026-02-06T16:01:36.789Z', level: 'error', message: 'MCP server "notion" failed to start: connection refused' },
          ],
          offset: 0,
          total: 3,
        },
      },
      fixtures,
    );
    await page.getByRole('button', { name: 'Debug Log' }).click();
    const dialog = page.getByRole('dialog', { name: 'Debug Log' });
    await expect(dialog.getByText('connection refused')).toBeVisible();
    await expect(dialog).toHaveScreenshot(SCREENSHOT_OPTS);
  });
});

test.describe('Facets (ScreenshotTests_Facets: 3 of 3, isolated AI Summary card captures)', () => {
  function facetsLocator(page: import('@playwright/test').Page) {
    return page.getByTitle('Collapse AI summary').locator('..');
  }

  test('testFacetsCardExpanded', async ({ page }) => {
    const fixtures = sessionFixtures();
    await gotoWithExtra(
      page,
      `/#/sessions/${MVP_PROJECT_ID}/${MVP_SESSION_ID}`,
      {
        get_session_facets: {
          sessionId: MVP_SESSION_ID,
          underlyingGoal: 'Refactor authentication to use JWT tokens with refresh token rotation',
          goalCategories: { refactoring: 2, security: 1, code_refactoring: 1 },
          outcome: 'success',
          userSatisfactionCounts: { likely_satisfied: 1 },
          claudeHelpfulness: 'very_helpful',
          sessionType: 'feature_implementation',
          frictionCounts: {},
          frictionDetail: '',
          primarySuccess: 'multi_file_changes',
          briefSummary: 'Migrated session auth from cookies to JWT with refresh token rotation. Updated middleware, auth routes, and added comprehensive test coverage.',
        },
      },
      fixtures,
    );
    const card = facetsLocator(page);
    await expect(card).toBeVisible();
    await expect(card).toHaveScreenshot(SCREENSHOT_OPTS);
  });

  test('testFacetsCardLight', async ({ page }) => {
    const fixtures = sessionFixtures();
    fixtures.prefs = { ...fixtures.prefs, colorTheme: 'white' };
    await gotoWithExtra(
      page,
      `/#/sessions/${MVP_PROJECT_ID}/${MVP_SESSION_ID}`,
      {
        get_session_facets: {
          sessionId: MVP_SESSION_ID,
          underlyingGoal: 'Refactor authentication to use JWT tokens with refresh token rotation',
          goalCategories: { refactoring: 2, security: 1, code_refactoring: 1 },
          outcome: 'success',
          userSatisfactionCounts: { likely_satisfied: 1 },
          claudeHelpfulness: 'very_helpful',
          sessionType: 'feature_implementation',
          frictionCounts: {},
          frictionDetail: '',
          primarySuccess: 'multi_file_changes',
          briefSummary: 'Migrated session auth from cookies to JWT with refresh token rotation.',
        },
      },
      fixtures,
    );
    const card = facetsLocator(page);
    await expect(card).toBeVisible();
    await expect(card).toHaveScreenshot(SCREENSHOT_OPTS);
  });

  test('testFacetsCardPartialOutcome', async ({ page }) => {
    const fixtures = sessionFixtures();
    await gotoWithExtra(
      page,
      `/#/sessions/${MVP_PROJECT_ID}/${MVP_SESSION_ID}`,
      {
        get_session_facets: {
          sessionId: MVP_SESSION_ID,
          underlyingGoal: 'Fix CI pipeline failures and add missing integration tests',
          goalCategories: { fix_bugs: 2, write_tests: 1, create_pr: 1 },
          outcome: 'partially_achieved',
          userSatisfactionCounts: { likely_satisfied: 1 },
          claudeHelpfulness: 'slightly_helpful',
          sessionType: 'multi_task',
          frictionCounts: { buggy_code: 1, tool_failures: 2 },
          frictionDetail: 'Unit tests initially failed due to coverage thresholds not being met, requiring additional test writing.',
          primarySuccess: 'multi_file_changes',
          briefSummary: 'Fixed most CI pipeline issues and created a PR, but some integration tests still need attention.',
        },
      },
      fixtures,
    );
    const card = facetsLocator(page);
    await expect(card).toBeVisible();
    await expect(card).toHaveScreenshot(SCREENSHOT_OPTS);
  });
});

test.describe('File History (ScreenshotTests_FileHistory: testFileHistory)', () => {
  test('testFileHistory', async ({ page }) => {
    const fixtures = sessionFixtures({ hasFileHistory: true });
    await gotoWithExtra(
      page,
      `/#/sessions/${MVP_PROJECT_ID}/${MVP_SESSION_ID}`,
      {
        // A single-version file keeps `FileHistory.vue`'s auto-selected-latest-version path on
        // `getFileVersion`+`diffLines` (both already answered by the base mock/this file's own
        // override) -- a second version would route through `diffFileVersions` instead, which
        // this fixture set deliberately avoids needing to also answer.
        get_file_history: {
          files: [
            { fileName: 'extensions/alert-triage/src/index.ts', versions: [{ versionId: 'a1b2c3@v1', version: 1, backupTime: '2026-02-20T14:01:00Z' }] },
          ],
        },
        get_file_version: { versionId: 'a1b2c3@v1', content: 'export const alertTriagePlugin = {\n  id: "alert-triage",\n  name: "Alert Triage",\n};\n' },
      },
      fixtures,
    );
    await page.getByRole('button', { name: 'File History' }).click();
    await expect(page.getByRole('heading', { name: 'File History' })).toBeVisible();
    await expect(page.getByText('index.ts')).toBeVisible();
    await expect(page).toHaveScreenshot(SCREENSHOT_OPTS);
  });
});

test.describe('Config screens (ScreenshotTests_ConfigScreens: 13 of 14 -- testConfigScreenHeader folded in, it is already part of every list screenshot below)', () => {
  test('testCommandsList', async ({ page }) => {
    await gotoWithExtra(page, '/#/config/command', {
      list_config_items: { value: [{ ref: { kind: 'command', project: null, slug: 'review' }, name: 'review', description: 'Review the current diff', scope: 'global', meta: { 'argument-hint': '[pr]' } }], skipped: 0 },
    });
    await expect(page.getByRole('heading', { name: 'Commands' })).toBeVisible();
    await expect(page).toHaveScreenshot(SCREENSHOT_OPTS);
  });

  test('testSkillsList', async ({ page }) => {
    await gotoWithExtra(page, '/#/config/skill', {
      list_config_items: { value: [{ ref: { kind: 'skill', project: null, slug: 'ponytail' }, name: 'ponytail', description: 'Forces the laziest solution that actually works', scope: 'global', meta: {} }], skipped: 0 },
    });
    await expect(page.getByRole('heading', { name: 'Skills' })).toBeVisible();
    await expect(page).toHaveScreenshot(SCREENSHOT_OPTS);
  });

  test('testMCPServersList', async ({ page }) => {
    await gotoWithExtra(page, '/#/config/mcp', {
      list_mcp_servers: { value: [{ id: 'playwright', name: 'playwright', rawName: 'playwright', scope: 'global', source: 'user', type: 'stdio', command: 'npx', args: ['@playwright/mcp'], url: null, envKeys: [], headerKeys: [], tools: ['browser_navigate', 'browser_click'], isWildcard: false }], skipped: 0 },
      get_mcp_statuses: { playwright: 'connected' },
    });
    await expect(page.getByRole('heading', { name: 'MCP Servers' })).toBeVisible();
    await expect(page).toHaveScreenshot(SCREENSHOT_OPTS);
  });

  test('testModelsList', async ({ page }) => {
    await gotoWithExtra(page, '/#/config/models', {
      list_models: { models: [{ displayName: 'Opus 5.5', description: 'Most capable model for complex reasoning', strengths: ['Deep reasoning', 'Long context'] }, { displayName: 'Sonnet 5.5', description: 'Balanced speed and capability', strengths: ['Everyday coding'] }], defaultModel: 'Sonnet 5.5', projectModel: null },
    });
    await expect(page.getByRole('heading', { name: 'Models' })).toBeVisible();
    await expect(page).toHaveScreenshot(SCREENSHOT_OPTS);
  });

  test('testSubAgentsList', async ({ page }) => {
    await gotoWithExtra(page, '/#/config/agent', {
      list_config_items: { value: [{ ref: { kind: 'agent', project: null, slug: 'code-reviewer' }, name: 'code-reviewer', description: 'Reviews code for quality and security', scope: 'builtIn', meta: {} }], skipped: 0 },
    });
    await expect(page.getByRole('heading', { name: 'Sub-agents' })).toBeVisible();
    await expect(page).toHaveScreenshot(SCREENSHOT_OPTS);
  });

  test('testPluginsList', async ({ page }) => {
    await gotoWithExtra(page, '/#/config/plugins', {
      list_plugins: [{ key: 'ponytail', name: 'Ponytail', author: 'Anthropic', version: '1.0.0', installedAt: '2026-01-01T00:00:00Z', enabled: true }],
    });
    await expect(page.getByRole('heading', { name: 'Plugins' })).toBeVisible();
    await expect(page).toHaveScreenshot(SCREENSHOT_OPTS);
  });

  test('testHooksList', async ({ page }) => {
    await gotoWithExtra(page, '/#/config/hooks', {
      list_hooks: { value: [{ event: 'PreToolUse', matcherGroupIndex: 0, matcher: 'Bash', handlers: [{ type: 'command', command: 'echo "running bash"', url: null, timeout: null, statusMessage: null, extraKeys: [] }], scope: 'global' }], skipped: 0 },
    });
    await expect(page.getByRole('heading', { name: 'Hooks' })).toBeVisible();
    await expect(page).toHaveScreenshot(SCREENSHOT_OPTS);
  });

  test('testOutputStylesList', async ({ page }) => {
    await gotoWithExtra(page, '/#/config/outputStyle', {
      list_config_items: { value: [{ ref: { kind: 'outputStyle', project: null, slug: 'concise' }, name: 'concise', description: 'Short, direct responses', scope: 'global', meta: {} }], skipped: 0 },
    });
    await expect(page.getByRole('heading', { name: 'Output Styles' })).toBeVisible();
    await expect(page).toHaveScreenshot(SCREENSHOT_OPTS);
  });

  test.describe('with a project selected (testCommandsListWithProject, testSkillsListWithProject, testMCPServersListWithProject)', () => {
    function withProjectFixtures() {
      const fixtures = buildFixtures();
      // `configStore.load()`'s own restore logic always maps a non-null `configProjectPath` to
      // `{kind: 'pickedFolder'}` (confirmed directly in config/store.ts) -- the path's content
      // only ever feeds the picker's own display label, never a lookup against `fixtures.projects`.
      fixtures.prefs = { ...fixtures.prefs, configProjectPath: '/Users/fixture/with-project' };
      return fixtures;
    }

    test('testCommandsListWithProject', async ({ page }) => {
      await gotoWithExtra(
        page,
        '/#/config/command',
        { list_config_items: { value: [{ ref: { kind: 'command', project: { id: 'with-project', name: 'with-project', path: '/Users/fixture/with-project', pathSource: 'transcriptCwd', sessionCount: 1, lastActivity: '2026-02-20T14:00:00Z', hasMemory: false }, slug: 'deploy' }, name: 'deploy', description: 'Deploy this project', scope: 'project', meta: {} }], skipped: 0 } },
        withProjectFixtures(),
      );
      await expect(page.getByTitle('Clear project')).toBeVisible();
      await expect(page).toHaveScreenshot(SCREENSHOT_OPTS);
    });

    test('testSkillsListWithProject', async ({ page }) => {
      await gotoWithExtra(
        page,
        '/#/config/skill',
        { list_config_items: { value: [], skipped: 0 } },
        withProjectFixtures(),
      );
      await expect(page.getByTitle('Clear project')).toBeVisible();
      await expect(page).toHaveScreenshot(SCREENSHOT_OPTS);
    });

    test('testMCPServersListWithProject', async ({ page }) => {
      await gotoWithExtra(
        page,
        '/#/config/mcp',
        { list_mcp_servers: { value: [], skipped: 0 }, get_mcp_statuses: {} },
        withProjectFixtures(),
      );
      await expect(page.getByTitle('Clear project')).toBeVisible();
      await expect(page).toHaveScreenshot(SCREENSHOT_OPTS);
    });
  });

  test.describe('empty states (testConfigEmptyState, testConfigEmptyStateMCP)', () => {
    test('testConfigEmptyState', async ({ page }) => {
      await gotoWithExtra(page, '/#/config/command', { list_config_items: { value: [], skipped: 0 } });
      await expect(page.getByText('No commands found')).toBeVisible();
      await expect(page).toHaveScreenshot(SCREENSHOT_OPTS);
    });

    test('testConfigEmptyStateMCP', async ({ page }) => {
      await gotoWithExtra(page, '/#/config/mcp', { list_mcp_servers: { value: [], skipped: 0 }, get_mcp_statuses: {} });
      await expect(page.getByText('No MCP servers found')).toBeVisible();
      await expect(page).toHaveScreenshot(SCREENSHOT_OPTS);
    });
  });
});

test.describe('Search, Onboarding and Settings (Heimdall-only: no Swift screenshot test exists for any of the three, M3-16/M3-17 only recently replaced their placeholders)', () => {
  test('testSearchOverlay', async ({ mockedPage: page }) => {
    await page.addInitScript(installExtraResponses, {
      search: { groups: [{ category: 'commands', hits: [{ title: 'review', subtitle: 'Review the current diff', highlights: [[0, 3]], action: { kind: 'openDetail', screen: 'commands', ref: { kind: 'command', project: null, slug: 'review' }, memoryFile: null } }] }] },
    });
    await page.goto('/');
    await page.evaluate(() => document.dispatchEvent(new KeyboardEvent('keydown', { key: 'k', ctrlKey: true })));
    await page.getByPlaceholder('Search everything…').fill('review');
    // `{ exact: true }`: a plain substring match also matches the hit's own subtitle ("Review the
    // current diff"), a real strict-mode violation caught on a real Linux CI run.
    await expect(page.getByRole('dialog', { name: 'Search' }).getByText('review', { exact: true })).toBeVisible();
    await expect(page.getByRole('dialog', { name: 'Search' })).toHaveScreenshot(SCREENSHOT_OPTS);
  });

  test('testOnboarding', async ({ page }) => {
    const fixtures = buildFixtures();
    fixtures.prefs = { ...fixtures.prefs, hasCompletedOnboarding: false };
    await gotoWithExtra(page, '/', { detect_claude_cli: { cliPath: '/usr/local/bin/claude', sessionsExist: true } }, fixtures);
    await expect(page.getByRole('dialog', { name: 'Welcome to Heimdall' })).toBeVisible();
    await expect(page).toHaveScreenshot(SCREENSHOT_OPTS);
  });

  test('testSettings', async ({ page }) => {
    await gotoWithExtra(page, '/#/settings', { list_memory_sources: [] });
    await expect(page.getByRole('heading', { name: 'Settings' })).toBeVisible();
    await expect(page).toHaveScreenshot(SCREENSHOT_OPTS);
  });
});
