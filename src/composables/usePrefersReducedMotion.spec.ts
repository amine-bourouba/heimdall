import { effectScope } from 'vue';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { usePrefersReducedMotion } from './usePrefersReducedMotion';

/** A controllable fake MediaQueryList: happy-dom implements matchMedia for real, but mocking it
 * directly gives this test full, explicit control over both states and the change event, rather
 * than depending on an environment's own CSS media-feature evaluation. */
function fakeMediaQueryList(initialMatches: boolean) {
  const listeners = new Set<(event: MediaQueryListEvent) => void>();
  return {
    matches: initialMatches,
    addEventListener: vi.fn((_type: string, cb: (event: MediaQueryListEvent) => void) => listeners.add(cb)),
    removeEventListener: vi.fn((_type: string, cb: (event: MediaQueryListEvent) => void) => listeners.delete(cb)),
    fire(matches: boolean) {
      this.matches = matches;
      for (const cb of listeners) cb({ matches } as MediaQueryListEvent);
    },
  };
}

let matchMediaSpy: ReturnType<typeof vi.spyOn>;

afterEach(() => {
  matchMediaSpy.mockRestore();
});

describe('usePrefersReducedMotion', () => {
  it('reflects the initial media query state', () => {
    const mql = fakeMediaQueryList(true);
    matchMediaSpy = vi.spyOn(window, 'matchMedia').mockReturnValue(mql as unknown as MediaQueryList);

    const scope = effectScope();
    const { prefersReducedMotion } = scope.run(() => usePrefersReducedMotion())!;
    expect(prefersReducedMotion.value).toBe(true);
    scope.stop();
  });

  it('starts false when the OS has no preference', () => {
    const mql = fakeMediaQueryList(false);
    matchMediaSpy = vi.spyOn(window, 'matchMedia').mockReturnValue(mql as unknown as MediaQueryList);

    const scope = effectScope();
    const { prefersReducedMotion } = scope.run(() => usePrefersReducedMotion())!;
    expect(prefersReducedMotion.value).toBe(false);
    scope.stop();
  });

  it('updates reactively when the OS setting changes while the app is open', () => {
    const mql = fakeMediaQueryList(false);
    matchMediaSpy = vi.spyOn(window, 'matchMedia').mockReturnValue(mql as unknown as MediaQueryList);

    const scope = effectScope();
    const { prefersReducedMotion } = scope.run(() => usePrefersReducedMotion())!;
    expect(prefersReducedMotion.value).toBe(false);

    mql.fire(true);
    expect(prefersReducedMotion.value).toBe(true);

    mql.fire(false);
    expect(prefersReducedMotion.value).toBe(false);
    scope.stop();
  });

  it('removes its change listener when the owning scope is disposed', () => {
    const mql = fakeMediaQueryList(false);
    matchMediaSpy = vi.spyOn(window, 'matchMedia').mockReturnValue(mql as unknown as MediaQueryList);

    const scope = effectScope();
    scope.run(() => usePrefersReducedMotion());
    expect(mql.removeEventListener).not.toHaveBeenCalled();

    scope.stop();
    expect(mql.removeEventListener).toHaveBeenCalledTimes(1);
  });
});
