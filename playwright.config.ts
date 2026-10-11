// port-plan.md P-8: "Playwright against the built frontend with fixture-backed IPC mocks on all
// OSes (WebKit engine stands in for WKWebView and WebKitGTK)" -- Chromium stands in for Windows'
// WebView2. `testDir` is scoped to `e2e/mocked` specifically: `e2e/smoke/` is WebdriverIO's own
// mocha-style suite (tauri-driver, Linux real-app smoke, M2-23's other half) and must never be
// picked up by Playwright's own test runner.
import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './e2e/mocked',
  fullyParallel: true,
  forbidOnly: Boolean(process.env.CI),
  retries: process.env.CI ? 1 : 0,
  reporter: process.env.CI ? 'github' : 'list',
  use: {
    baseURL: 'http://127.0.0.1:4173',
    trace: 'on-first-retry',
  },
  projects: [
    { name: 'chromium', use: { ...devices['Desktop Chrome'] } },
    { name: 'webkit', use: { ...devices['Desktop Safari'] } },
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
