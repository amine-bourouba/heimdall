// port-plan.md §9: "Transcript scrolling at 10,000+ blocks | median frame ≤ 16.7 ms, p95 ≤ 33 ms"
// (M2-28). Chromium only (`playwright.config.ts`'s `perf` project) -- frame-timing precision
// matters more here than engine coverage, and Chromium's `requestAnimationFrame`/
// `performance.now()` is the most consistent of the three engines Heimdall ships on.
import { test as base, expect } from '@playwright/test';
import type { Page } from '@playwright/test';
import { installMockIpc } from '../support/mockIpc';
import { buildPerfFixtures, PERF_PROJECT_ID, PERF_SESSION_ID } from '../support/fixtures';

const MEDIAN_BUDGET_MS = 16.7;
const P95_BUDGET_MS = 33;
const SCROLL_DURATION_MS = 4000;
// 60px/frame at ~60fps is roughly 1 message-row's worth (`estimateSize: 160` in
// SessionDetail.vue) every other frame -- brisk enough to travel past several of the
// virtualizer's own 20-message pagination boundaries within the sample window, not just sit
// still sampling idle frames.
const SCROLL_STEP_PX = 60;

const test = base.extend<{ perfPage: Page }>({
  perfPage: async ({ page }, use) => {
    await installMockIpc(page, buildPerfFixtures());
    await use(page);
  },
});

function percentile(sortedAscending: number[], p: number): number {
  const index = Math.min(sortedAscending.length - 1, Math.floor(p * sortedAscending.length));
  return sortedAscending[index];
}

test('scrolling a 10,000+ block transcript stays within the median/p95 frame-time budget', async ({ perfPage: page }) => {
  await page.goto(`/#/sessions/${PERF_PROJECT_ID}/${PERF_SESSION_ID}`);
  await expect(page.getByRole('heading', { name: 'Performance fixture: 10k+ blocks' })).toBeVisible();

  const frameTimesMs = await page.evaluate(
    async ({ durationMs, stepPx }) => {
      const container = document.querySelector('[data-testid="session-scroll-container"]');
      if (!container) throw new Error('session-scroll-container not found');

      return await new Promise<number[]>((resolve) => {
        const deltas: number[] = [];
        let last: number | null = null;
        const start = performance.now();

        function tick(now: number) {
          if (last !== null) deltas.push(now - last);
          last = now;
          container!.scrollTop += stepPx;
          if (now - start < durationMs) requestAnimationFrame(tick);
          else resolve(deltas);
        }
        requestAnimationFrame(tick);
      });
    },
    { durationMs: SCROLL_DURATION_MS, stepPx: SCROLL_STEP_PX },
  );

  expect(frameTimesMs.length, 'expected at least a few dozen sampled frames').toBeGreaterThan(20);

  const sorted = [...frameTimesMs].sort((a, b) => a - b);
  const median = percentile(sorted, 0.5);
  const p95 = percentile(sorted, 0.95);

  // The real numbers this task's own AC3 asks to be copied into docs/perf-baseline.md -- a
  // passing assertion alone doesn't surface them anywhere.
  console.log(`transcript-scroll perf: samples=${frameTimesMs.length} median=${median.toFixed(2)}ms p95=${p95.toFixed(2)}ms`);

  expect(median, `median frame time ${median.toFixed(2)}ms exceeds the ${MEDIAN_BUDGET_MS}ms budget`).toBeLessThanOrEqual(MEDIAN_BUDGET_MS);
  expect(p95, `p95 frame time ${p95.toFixed(2)}ms exceeds the ${P95_BUDGET_MS}ms budget`).toBeLessThanOrEqual(P95_BUDGET_MS);
});
