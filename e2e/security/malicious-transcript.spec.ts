// ADR 0010's own acceptance test: "Playwright run on a malicious fixture transcript asserting no
// dialog, navigation, off-origin request or new window" (M2-26). Runs mocked, in WebKit only
// (playwright.config.ts's `security` project) -- the real-app half of this same acceptance test
// (Linux and Windows, loading the real `fixtures/security/malicious-claude-home/` tree through
// heimdall-core) is e2e/support/tauriCdp.ts and e2e/smoke/app.smoke.mjs, extended by this task to
// run the same four checks against the real binary.
import { test as base, expect } from '@playwright/test';
import type { Page } from '@playwright/test';
import { installMockIpc } from '../support/mockIpc';
import { buildMaliciousFixtures, MALICIOUS_PROJECT_ID, MALICIOUS_SESSION_ID } from '../support/fixtures';

const test = base.extend<{ maliciousPage: Page }>({
  maliciousPage: async ({ page }, use) => {
    await installMockIpc(page, buildMaliciousFixtures());
    await use(page);
  },
});

test('opening a transcript full of attack payloads causes no dialog, no navigation, no off-origin request and no new window', async ({ maliciousPage: page, baseURL }) => {
  const dialogs: string[] = [];
  page.on('dialog', (dialog) => {
    dialogs.push(`${dialog.type()}: ${dialog.message()}`);
    void dialog.dismiss();
  });

  const popups: string[] = [];
  page.on('popup', (popup) => popups.push(popup.url()));

  // Blocked, not just observed: a real sanitization failure here would otherwise make a real
  // DNS/network attempt to a domain that doesn't exist, which is slow and can hang rather than
  // fail fast. Aborting still fires the `request` event below, so detection is unaffected.
  await page.route('**evil.example**', (route) => route.abort());

  const offOriginRequests: string[] = [];
  const appOrigin = new URL(baseURL!).origin;
  page.on('request', (request) => {
    if (new URL(request.url()).origin !== appOrigin) offOriginRequests.push(request.url());
  });

  await page.goto(`/#/sessions/${MALICIOUS_PROJECT_ID}/${MALICIOUS_SESSION_ID}`);
  await expect(page.getByRole('heading', { name: 'Malicious transcript fixture' })).toBeVisible();

  // Both blocks start expanded already: the fixture's own `prefs.autoExpandBlocks: true` (inherited
  // from `buildFixtures()`) is exactly what `ThinkingBlock`/`ToolBlock` read `onMounted`. The tool
  // block's own content still renders as plain, already-escaped text at this point though --
  // `ToolBlock.vue`'s `showRawContent` starts `true` for every tool, with no toggle to leave it at
  // all for Bash (`v-if="!isBash()"`); the fixture deliberately uses `Read`, not `Bash`, as its
  // tool so this toggle exists, and clicking it is what actually sends the tool-output bucket's
  // payloads through `MarkdownView`'s real v-html/DOMPurify path instead of a safe-by-construction
  // text interpolation.
  await page.getByTitle('Show rendered markdown').click();

  // markdown-it's `html: false` renders every *raw-HTML* payload (onerror, svg onload, iframe,
  // meta refresh, the raw `<a>`-tag scheme-obfuscation variants) as inert escaped text outright --
  // DOMPurify never even gets a tag to strip for those. It still runs `linkify: true` on the
  // escaped text, though, which turns the bare URL substring inside the iframe-remote payload's
  // own escaped markup into one real, clickable `<a href="https://evil.example/">` link: the one
  // actually-live element in this whole transcript that could navigate anywhere interesting.
  // Clicking it exercises ADR 0010 point 2's click-interception end to end: `MarkdownView.vue`'s
  // `onContentClick` `preventDefault()`s every link and routes it through the (mocked)
  // `open_external_url` command instead of ever letting the webview navigate there for real.
  await page.getByRole('link', { name: 'https://evil.example/', exact: true }).first().click();

  // Give any (hypothetical, sanitization-failure) scripted payload -- an onload/onerror/meta
  // refresh fires synchronously on render -- a moment to act before asserting silence.
  await page.waitForTimeout(500);

  expect(dialogs, `unexpected dialog(s): ${dialogs.join(', ')}`).toEqual([]);
  expect(popups, `unexpected new window/tab: ${popups.join(', ')}`).toEqual([]);
  expect(offOriginRequests, `unexpected off-origin request(s): ${offOriginRequests.join(', ')}`).toEqual([]);
  expect(page.url().startsWith(appOrigin), `navigated away from the app origin to ${page.url()}`).toBe(true);
});
