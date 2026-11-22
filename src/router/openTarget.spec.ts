// M3-58: what the main window does when the tray's `open_in_main_window` asks it to navigate.
import { describe, expect, it, vi } from 'vitest';
import type { NavScreen } from '@/ipc/bindings';
import { navigationItems } from './navigationItems';
import { openTarget } from './openTarget';

describe('openTarget', () => {
  it('opens a session in its detail view, by project and session id', () => {
    const push = vi.fn();
    openTarget({ push }, { kind: 'session', projectId: 'proj-1', sessionId: 'sess-9' });
    expect(push).toHaveBeenCalledExactlyOnceWith({ name: 'session-detail', params: { projectId: 'proj-1', sessionId: 'sess-9' } });
  });

  it('opens a screen at that sidebar item\'s own route', () => {
    const push = vi.fn();
    openTarget({ push }, { kind: 'screen', screen: 'sessions' });
    expect(push).toHaveBeenCalledExactlyOnceWith(navigationItems.find((i) => i.id === 'sessions')!.to);
  });

  // `NavScreen` is the contract's list of screens (bindings.ts); a screen with no sidebar entry would
  // be silently ignored, so every one of them has to resolve. `Record` makes a new variant a compile error here.
  const SCREENS: Record<NavScreen, true> = {
    sessions: true, todos: true, commands: true, skills: true, plans: true, memory: true, hooks: true,
    mcpServers: true, plugins: true, outputStyles: true, models: true, subAgents: true, analytics: true, history: true,
  };
  it.each(Object.keys(SCREENS) as NavScreen[])('every contract screen (%s) has a route to open', (screen) => {
    const push = vi.fn();
    openTarget({ push }, { kind: 'screen', screen });
    expect(push).toHaveBeenCalledExactlyOnceWith(navigationItems.find((i) => i.id === screen)!.to);
  });

  it('ignores a screen it does not know, rather than navigating somewhere else', () => {
    const push = vi.fn();
    openTarget({ push }, { kind: 'screen', screen: 'nowhere' as NavScreen });
    expect(push).not.toHaveBeenCalled();
  });
});
