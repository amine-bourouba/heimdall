// Windows real-app smoke (port-plan.md P-8, this task's AC2): WebView2 is Chromium-based and,
// unlike WKWebView/WebKitGTK, exposes a real Chrome DevTools Protocol endpoint -- so Playwright
// can drive the actual compiled app directly, no WebDriver server needed on this one OS. WebView2
// only opens that endpoint when told to via `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` (its own,
// documented pass-through for Chromium flags), which is why this is a separate helper from
// `e2e/smoke/` rather than another tauri-driver target: tauri-driver has no Windows benefit over
// just connecting directly, and A-10 already rules it out for macOS.
import { spawn, type ChildProcess } from 'node:child_process';
import { pathToFileURL } from 'node:url';
import { chromium, type Browser, type Page } from '@playwright/test';

const CDP_PORT = 9223;
const CDP_URL = `http://127.0.0.1:${CDP_PORT}`;
const BINARY_PATH = 'src-tauri/target/debug/heimdall-desktop.exe';

async function waitForCdp(timeoutMs: number): Promise<void> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    try {
      const res = await fetch(`${CDP_URL}/json/version`);
      if (res.ok) return;
    } catch {
      // not up yet
    }
    await new Promise((resolve) => setTimeout(resolve, 500));
  }
  throw new Error(`WebView2 CDP endpoint at ${CDP_URL} never came up within ${timeoutMs}ms`);
}

export interface LaunchedTauriApp {
  process: ChildProcess;
  browser: Browser;
  page: Page;
  close: () => Promise<void>;
}

/** Launches the real debug binary with a fixture `HEIMDALL_CLAUDE_HOME`, waits for WebView2's CDP
 * endpoint, and connects Playwright to the window it actually opened. */
export async function launchTauriAppOverCdp(claudeHome: string): Promise<LaunchedTauriApp> {
  const child = spawn(BINARY_PATH, [], {
    env: {
      ...process.env,
      HEIMDALL_CLAUDE_HOME: claudeHome,
      WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${CDP_PORT}`,
    },
    stdio: 'pipe',
  });

  // A missing/unspawnable binary emits a Node 'error' event, not an 'exit' -- without a listener
  // for it, that is an unhandled 'error' event, which crashes the whole process instead of
  // rejecting this promise (confirmed directly: a first version here crashed with exactly that
  // for a binary that doesn't exist on this dev machine, rather than a clean, catchable error).
  const earlyExit = new Promise<never>((_, reject) => {
    child.once('error', (err) => reject(new Error(`failed to launch ${BINARY_PATH}: ${err.message}`)));
    child.once('exit', (code) => reject(new Error(`${BINARY_PATH} exited early with code ${code}`)));
  });

  await Promise.race([waitForCdp(15_000), earlyExit]);

  const browser = await chromium.connectOverCDP(CDP_URL);
  const context = browser.contexts()[0];
  const page = context?.pages()[0] ?? (await context!.waitForEvent('page'));

  return {
    process: child,
    browser,
    page,
    close: async () => {
      await browser.close();
      child.kill();
    },
  };
}

/** ADR 0010's acceptance test ("Playwright run on a malicious fixture transcript asserting no
 * dialog, navigation, off-origin request or new window", M2-26) -- the Windows half, against the
 * real binary over the same CDP connection `launchTauriAppOverCdp` already gives every caller
 * here. WebView2 is Chromium-based, so (unlike the Linux/tauri-driver half in
 * `e2e/smoke/app.smoke.mjs`) this gets the exact same `page.on('request')` network visibility the
 * mocked-WebKit spec (`e2e/security/malicious-transcript.spec.ts`) uses. */
export async function assertNoMaliciousTranscriptEscape(page: Page): Promise<void> {
  const dialogs: string[] = [];
  page.on('dialog', (dialog) => {
    dialogs.push(`${dialog.type()}: ${dialog.message()}`);
    void dialog.dismiss();
  });

  const popups: string[] = [];
  page.on('popup', (popup) => popups.push(popup.url()));

  await page.route('**evil.example**', (route) => route.abort());

  const offOriginRequests: string[] = [];
  const appOrigin = new URL(page.url()).origin;
  page.on('request', (request) => {
    if (new URL(request.url()).origin !== appOrigin) offOriginRequests.push(request.url());
  });

  // The fixture's only project has exactly one session -- clicking the first/only session row
  // rather than matching an exact title, since the real session's title is Rust-derived from the
  // transcript's own (attacker-controlled) first message text, not a clean string knowable ahead
  // of time the way the mocked spec's hand-set title is.
  await page.locator('nav[aria-label="Sessions"] li button').first().click();
  await page.waitForSelector('h1', { timeout: 10_000 });
  await page.waitForTimeout(500);

  if (dialogs.length > 0) throw new Error(`unexpected dialog(s): ${dialogs.join(', ')}`);
  if (popups.length > 0) throw new Error(`unexpected new window/tab: ${popups.join(', ')}`);
  if (offOriginRequests.length > 0) throw new Error(`unexpected off-origin request(s): ${offOriginRequests.join(', ')}`);
  if (!page.url().startsWith(appOrigin)) throw new Error(`navigated away from the app origin to ${page.url()}`);
}

/** The Windows real-app smoke itself (AC2): no separate spec file owns this -- there is nothing
 * in this task's Outputs to run it as a Playwright Test Runner spec, since it drives a real OS
 * process rather than a mocked page. Run directly: `node e2e/support/tauriCdp.ts [claudeHome]`
 * (M2-26: `node e2e/support/tauriCdp.ts fixtures/security/malicious-claude-home` runs the
 * malicious-transcript check above too, keyed off the path containing "malicious"). */
async function main() {
  const claudeHome = process.argv[2] ?? 'fixtures/screenshot/claude-home';
  const app = await launchTauriAppOverCdp(claudeHome);
  try {
    await app.page.waitForSelector('nav[aria-label="Primary"]', { timeout: 10_000 });

    console.log('OK: sidebar rendered');

    if (claudeHome.includes('malicious')) {
      await assertNoMaliciousTranscriptEscape(app.page);

      console.log('OK: malicious transcript produced no dialog, popup, off-origin request or navigation');
    }
  } finally {
    await app.close();
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main().catch((error: unknown) => {
     
    console.error(error);
    process.exitCode = 1;
  });
}
