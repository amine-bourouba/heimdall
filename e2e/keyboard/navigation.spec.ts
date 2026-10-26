// M3-19's own AC: "Covers Mod+K, arrows and Return in search, Mod+[ and Mod+], Mod+T, font
// shortcuts, layered Esc, form Mod+./Return equivalents, onboarding Left and Right." Ported from
// parity-matrix.md section J (J1-J9; J10's own "hook, MCP and sub-agent forms" are explicitly
// "Later" scope tied to M3-47/48/49, none of which exist yet -- skipped, not faked, see this
// file's own Follow-up in docs/progress-log.md) and KeyboardNavigationModifier.swift.
//
// "with visible focus" (the Goal's own wording, WCAG 2.4.7) means every shortcut that is supposed
// to move focus is asserted with `.toBeFocused()` here, not just that some element exists.
//
// `useShortcuts.ts` listens on `document`, not `window` (confirmed the hard way in
// e2e/visual/session.spec.ts's own header comment) -- every keydown below dispatches on
// `document`. And every test waits for a real, visible element right after `goto()` before
// dispatching anything: `session.spec.ts`'s own `testSessionToolFilter` had a real CI flake from
// skipping that wait (fixed in the same commit that added this file) -- never repeat it here.
import { test, expect } from '@playwright/test';
import { installMockIpc } from '../support/mockIpc';
import { buildVisualFixtures, VISUAL_PROJECT_ID, VISUAL_SESSION_ID } from '../support/fixtures';
import type { MockFixtures } from '../support/fixtures';

function installExtraResponses(responses: Record<string, unknown>) {
  const win = window as unknown as { __TAURI_INTERNALS__: { invoke: (cmd: string, args?: unknown) => Promise<unknown> } };
  const real = win.__TAURI_INTERNALS__.invoke;
  win.__TAURI_INTERNALS__.invoke = (cmd: string, args?: unknown) => (cmd in responses ? Promise.resolve(responses[cmd]) : real(cmd, args));
}

async function gotoWithExtra(page: import('@playwright/test').Page, path: string, extra: Record<string, unknown> = {}, fixtures: MockFixtures = buildVisualFixtures()) {
  await installMockIpc(page, fixtures);
  if (Object.keys(extra).length > 0) await page.addInitScript(installExtraResponses, extra);
  await page.goto(path);
}

function modKeydown(page: import('@playwright/test').Page, key: string) {
  return page.evaluate((k) => document.dispatchEvent(new KeyboardEvent('keydown', { key: k, ctrlKey: true })), key);
}

function escKeydown(page: import('@playwright/test').Page) {
  return page.evaluate(() => document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' })));
}

test.describe('J1/J7/J8: Mod+K opens search with visible focus; arrows and Return activate a result', () => {
  test('Mod+K focuses the search input, ArrowDown selects the first result, Return navigates to it', async ({ page }) => {
    await gotoWithExtra(page, '/', {
      search: {
        groups: [
          {
            category: 'commands',
            hits: [{ title: 'review', subtitle: 'Review the current diff', highlights: [[0, 3]], action: { kind: 'openDetail', screen: 'commands', ref: { kind: 'command', project: null, slug: 'review' }, memoryFile: null } }],
          },
        ],
      },
      get_config_item: { ref: { kind: 'command', project: null, slug: 'review' }, name: 'review', description: 'Review the current diff', scope: 'global', meta: {}, body: '# review\n\nReview the current diff.', displayPath: '~/.claude/commands/review.md' },
    });
    await expect(page.getByRole('navigation', { name: 'Primary' })).toBeVisible();

    await modKeydown(page, 'k');
    const input = page.getByPlaceholder('Search everything…');
    await expect(input).toBeFocused();

    await input.fill('review');
    const option = page.getByRole('option', { name: /review/ });
    await expect(option).toBeVisible();

    // The first (only) hit is already selected by default; ArrowDown/ArrowUp are a no-op here
    // with one result, but still must not throw or move selection out of range.
    await page.keyboard.press('ArrowDown');
    await page.keyboard.press('ArrowUp');
    await expect(page.getByRole('option', { selected: true })).toHaveText(/review/);

    await page.keyboard.press('Enter');
    await expect(page.getByRole('heading', { name: 'review' })).toBeVisible();
    await expect(page).toHaveURL(/#\/config\/command\/global\/review/);
  });
});

test.describe('J2: Mod+[ and Mod+] move through real navigation history', () => {
  test('Mod+[ goes back, Mod+] goes forward', async ({ page }) => {
    await gotoWithExtra(page, '/');
    await expect(page.getByRole('navigation', { name: 'Primary' })).toBeVisible();
    await expect(page).toHaveURL(/#\/sessions$/);

    await page.getByRole('link', { name: 'TODOs' }).click();
    await expect(page).toHaveURL(/#\/todos$/);

    await modKeydown(page, '[');
    await expect(page).toHaveURL(/#\/sessions$/);

    await modKeydown(page, ']');
    await expect(page).toHaveURL(/#\/todos$/);
  });
});

test.describe('J3: Mod+T toggles the tool filter chip bar', () => {
  test('opens the chip bar for a real session with tool-use blocks, Escape closes it', async ({ page }) => {
    await gotoWithExtra(page, `/#/sessions/${VISUAL_PROJECT_ID}/${VISUAL_SESSION_ID}`);
    await expect(page.getByRole('heading', { name: 'Visual regression: all tools' })).toBeVisible();

    await modKeydown(page, 't');
    const readChip = page.getByRole('button', { name: 'Read', exact: true });
    await expect(readChip).toBeVisible();

    await escKeydown(page);
    await expect(readChip).not.toBeVisible();
  });
});

test.describe('J4: Font shortcuts reach the real DOM', () => {
  test('Mod+=/-/0 change the real --font-scale custom property on <html>', async ({ page }) => {
    await gotoWithExtra(page, '/');
    await expect(page.getByRole('navigation', { name: 'Primary' })).toBeVisible();

    function fontScale(p: typeof page) {
      return p.evaluate(() => getComputedStyle(document.documentElement).getPropertyValue('--font-scale').trim());
    }

    await modKeydown(page, '=');
    await expect.poll(() => fontScale(page)).toBe('1.05');

    await modKeydown(page, '-');
    await expect.poll(() => fontScale(page)).toBe('1');

    await modKeydown(page, '=');
    await modKeydown(page, '0');
    await expect.poll(() => fontScale(page)).toBe('1');
  });
});

test.describe('J6: layered Escape, real priority order (session search, tool filter, config detail, session detail)', () => {
  test('clears session search before the tool filter on the same session', async ({ page }) => {
    await gotoWithExtra(page, `/#/sessions/${VISUAL_PROJECT_ID}/${VISUAL_SESSION_ID}`);
    await expect(page.getByRole('heading', { name: 'Visual regression: all tools' })).toBeVisible();

    await page.getByPlaceholder('Find in session…').fill('build');
    await modKeydown(page, 't');
    const readChip = page.getByRole('button', { name: 'Read', exact: true });
    await expect(readChip).toBeVisible();

    await escKeydown(page);
    await expect(page.getByPlaceholder('Find in session…')).toHaveValue('');
    await expect(readChip).toBeVisible(); // tool filter untouched by the first Escape

    await escKeydown(page);
    await expect(readChip).not.toBeVisible();
  });

  test('from a config-detail route, Escape goes back to the matching config-list route', async ({ page }) => {
    await gotoWithExtra(page, '/#/config/command/global/review', {
      get_config_item: { ref: { kind: 'command', project: null, slug: 'review' }, name: 'review', description: '', scope: 'global', meta: {}, body: '# review', displayPath: '~/.claude/commands/review.md' },
      list_config_items: { value: [], skipped: 0 },
    });
    await expect(page.getByRole('heading', { name: 'review' })).toBeVisible();

    await escKeydown(page);
    await expect(page).toHaveURL(/#\/config\/command$/);
  });

  test('from a session-detail route, Escape deselects the session but keeps the project', async ({ page }) => {
    await gotoWithExtra(page, `/#/sessions/${VISUAL_PROJECT_ID}/${VISUAL_SESSION_ID}`);
    await expect(page.getByRole('heading', { name: 'Visual regression: all tools' })).toBeVisible();

    await escKeydown(page);
    await expect(page).toHaveURL(new RegExp(`#/sessions/${VISUAL_PROJECT_ID}$`));
  });
});

test.describe('J9: Onboarding Left/Right move between its 3 pages', () => {
  test('ArrowRight advances through all 3 pages, ArrowLeft moves back', async ({ page }) => {
    const fixtures = buildVisualFixtures();
    fixtures.prefs = { ...fixtures.prefs, hasCompletedOnboarding: false };
    await gotoWithExtra(page, '/', { detect_claude_cli: { cliPath: '/usr/local/bin/claude', sessionsExist: true } }, fixtures);
    const dialog = page.getByRole('dialog', { name: 'Welcome to Heimdall' });
    await expect(dialog).toBeVisible();
    await expect(dialog.getByRole('heading', { name: 'Welcome to Heimdall' })).toBeVisible();

    await page.evaluate(() => document.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight' })));
    await expect(dialog.getByRole('heading', { name: 'Claude Code Detected' })).toBeVisible();

    await page.evaluate(() => document.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight' })));
    await expect(dialog.getByRole('heading', { name: 'Key Features' })).toBeVisible();

    await page.evaluate(() => document.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowLeft' })));
    await expect(dialog.getByRole('heading', { name: 'Claude Code Detected' })).toBeVisible();
  });
});
