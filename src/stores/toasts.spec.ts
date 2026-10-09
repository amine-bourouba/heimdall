import { mockIPC } from '@tauri-apps/api/mocks';
import { createPinia, setActivePinia } from 'pinia';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { useToastsStore } from './toasts';

beforeEach(() => {
  setActivePinia(createPinia());
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
});

// From-scratch design (ported from ToastOverlay.swift's behaviour, not a Swift test -- see
// toasts.ts's own header comment: AppStateTests.swift has no toast cases at all).
describe('useToastsStore', () => {
  it('is a single-item queue: only the first toast is "current"', () => {
    const store = useToastsStore();
    store.show('first');
    store.show('second');

    expect(store.current?.message).toBe('first');
    expect(store.queue).toHaveLength(2);
  });

  it('auto-dismisses after 4s with no URL', () => {
    const store = useToastsStore();
    store.show('saved');
    expect(store.current).not.toBeNull();

    vi.advanceTimersByTime(3_999);
    expect(store.current).not.toBeNull();
    vi.advanceTimersByTime(1);
    expect(store.current).toBeNull();
  });

  it('auto-dismisses after 10s when a URL is attached', () => {
    const store = useToastsStore();
    store.show('update available', { url: 'https://example.com' });

    vi.advanceTimersByTime(4_000);
    expect(store.current).not.toBeNull(); // the short timeout must not fire for a URL toast
    vi.advanceTimersByTime(6_000);
    expect(store.current).toBeNull();
  });

  it('dismissing the current toast advances to the next and restarts its own timer', () => {
    const store = useToastsStore();
    store.show('first');
    store.show('second', { url: 'https://example.com' });

    store.dismissCurrent();
    expect(store.current?.message).toBe('second');

    vi.advanceTimersByTime(4_000);
    expect(store.current).not.toBeNull(); // second's own 10s window, not first's 4s
    vi.advanceTimersByTime(6_000);
    expect(store.current).toBeNull();
  });

  it('clickCurrent opens the URL through open_external_url, then dismisses', async () => {
    const opened: string[] = [];
    mockIPC((cmd, args) => {
      if (cmd === 'open_external_url') {
        opened.push((args as { url: string }).url);
        return null;
      }
      throw new Error(`unexpected command: ${cmd}`);
    });

    const store = useToastsStore();
    store.show('release notes', { url: 'https://example.com/release' });

    await store.clickCurrent();

    expect(opened).toEqual(['https://example.com/release']);
    expect(store.current).toBeNull();
  });

  it('clickCurrent with no URL just dismisses, without calling IPC', async () => {
    mockIPC((cmd) => {
      throw new Error(`unexpected command: ${cmd}`);
    });

    const store = useToastsStore();
    store.show('no link here');

    await store.clickCurrent();
    expect(store.current).toBeNull();
  });

  it('clickCurrent on an empty queue is a no-op', async () => {
    const store = useToastsStore();
    await expect(store.clickCurrent()).resolves.toBeUndefined();
  });
});
