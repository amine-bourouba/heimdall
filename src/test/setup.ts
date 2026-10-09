import { afterEach, beforeEach } from 'vitest';
import { clearMocks, mockIPC, mockWindows } from '@tauri-apps/api/mocks';

// Every test starts with a clean, consistently mocked IPC/window seam — mirrors the official
// mockIPC/clearMocks pattern exactly so a leftover handler from one spec can never answer another
// spec's invoke() call (clearMocks wipes both, so it has to be re-applied each time, not once).
beforeEach(() => {
  mockWindows('main');
  mockIPC(() => {
    throw new Error('invoke() was called without a mockIPC() handler for this test');
  });
});

afterEach(() => {
  clearMocks();
});
