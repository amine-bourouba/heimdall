// AC3: "The Linux real-app smoke does the same through tauri-driver under `xvfb-run`." Same
// assertion `tauriCdp.ts`'s Windows smoke makes -- the sidebar's own `<nav aria-label="Primary">`
// (Sidebar.vue) -- so a pass means the same real thing on both OSes: the app launched for real,
// loaded prefs/projects through the real `HEIMDALL_CLAUDE_HOME` fixture override, and rendered.
describe('Heimdall real-app smoke (Linux, tauri-driver)', () => {
  it('renders the primary sidebar', async () => {
    const sidebar = await $('nav[aria-label="Primary"]');
    await sidebar.waitForExist({ timeout: 10000 });
    await expect(sidebar).toBeDisplayed();
  });
});

// ADR 0010's acceptance test ("Playwright run on a malicious fixture transcript asserting no
// dialog, navigation, off-origin request or new window", M2-26) -- the Linux half, run only when
// `wdio.conf.mjs` was pointed at `fixtures/security/malicious-claude-home` via
// `HEIMDALL_E2E_CLAUDE_HOME` (`npm run test:e2e:linux-smoke:security`); a no-op describe block
// during the ordinary AC1 run above, which never sets that var.
//
// tauri-driver proxies WebKitGTK's own native WebDriver protocol, which has no CDP-style network
// interception at all -- unlike `e2e/security/malicious-transcript.spec.ts` (the mocked-WebKit
// half of this same acceptance test, run through Playwright's own `page.on('request')`), "no
// request to a non-app origin" is honestly not checked here for that reason; window count,
// in-page alert dialogs and the URL are.
if (process.env.HEIMDALL_E2E_CLAUDE_HOME?.includes('malicious')) {
  describe('Heimdall real-app smoke: malicious transcript (Linux, tauri-driver, M2-26)', () => {
    it('opening the malicious transcript session shows no alert dialog, no new window, and no navigation away from the app', async () => {
      const startOrigin = new URL(await browser.getUrl()).origin;

      // The fixture's only project has exactly one session -- clicking the first/only session
      // row rather than matching an exact title, since the real session's title is Rust-derived
      // from the transcript's own (attacker-controlled) first message text, not a clean string
      // knowable ahead of time the way the mocked spec's hand-set title is.
      const sessionButton = await $('nav[aria-label="Sessions"] li button');
      await sessionButton.waitForExist({ timeout: 10000 });
      await sessionButton.click();
      await browser.pause(500);

      const handles = await browser.getWindowHandles();
      expect(handles.length).toBe(1);

      // The in-app hash route is expected to change (normal SPA navigation into the session); the
      // origin itself must not -- that's the one thing an off-origin navigation would actually
      // change that normal in-app routing never does.
      expect(new URL(await browser.getUrl()).origin).toBe(startOrigin);

      let alertIsPresent = true;
      try {
        await browser.getAlertText();
      } catch {
        alertIsPresent = false;
      }
      expect(alertIsPresent).toBe(false);
    });
  });
}
