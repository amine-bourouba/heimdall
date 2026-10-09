import { afterEach, beforeEach } from 'vitest';
import { clearMocks, mockIPC, mockWindows } from '@tauri-apps/api/mocks';

// Every test starts with a clean, consistently mocked IPC/window seam — mirrors the official
// mockIPC/clearMocks pattern exactly so a leftover handler from one spec can never answer another
// spec's invoke() call (clearMocks wipes both, so it has to be re-applied each time, not once).
// Guarded: a spec that opts out of the DOM with `// @vitest-environment node` (contrast.spec.ts,
// which only reads a file and does arithmetic) has no `window` for these to act on at all.
beforeEach(() => {
  if (typeof window === 'undefined') return;
  mockWindows('main');
  mockIPC(() => {
    throw new Error('invoke() was called without a mockIPC() handler for this test');
  });
});

afterEach(() => {
  if (typeof window === 'undefined') return;
  clearMocks();
});
