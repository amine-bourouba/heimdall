// port-plan.md P-8: "Playwright against the built frontend with fixture-backed IPC mocks on all
// OSes (WebKit engine stands in for WKWebView and WebKitGTK)" -- Chromium stands in for Windows'
// WebView2. `e2e/smoke/` is WebdriverIO's own mocha-style suite (tauri-driver, Linux real-app
// smoke, M2-23's other half) and must never be picked up by Playwright's own test runner, so every
// project below has its own explicit `testDir` rather than one shared top-level default.
import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  fullyParallel: true,
  forbidOnly: Boolean(process.env.CI),
  retries: process.env.CI ? 1 : 0,
  reporter: process.env.CI ? 'github' : 'list',
  use: {
    baseURL: 'http://127.0.0.1:4173',
    trace: 'on-first-retry',
  },
  projects: [
    { name: 'chromium', testDir: './e2e/mocked', use: { ...devices['Desktop Chrome'] } },
    { name: 'webkit', testDir: './e2e/mocked', use: { ...devices['Desktop Safari'] } },
    // port-plan.md P-13: committed baselines are Linux-only -- WebKitGTK's own font rendering is
    // the closest engine Playwright ships to it, and the only one worth comparing against a
    // Linux-captured baseline at all (M2-24's own task, e2e/visual/).
    { name: 'visual', testDir: './e2e/visual', use: { ...devices['Desktop Safari'] } },
  ],
  webServer: {
    // The build itself can take longer than Playwright's default web-server timeout on a loaded
    // CI runner, confirmed empirically (a first version here timed out at 60s with the build
    // alone still in progress) -- building once up front and serving the already-built `dist/`
    // keeps server start fast and leaves the generous timeout as headroom, not the steady state.
    command: 'npm run build && npm run preview -- --host 127.0.0.1 --port 4173 --strictPort',
    url: 'http://127.0.0.1:4173',
    reuseExistingServer: !process.env.CI,
    timeout: 180_000,
  },
});
