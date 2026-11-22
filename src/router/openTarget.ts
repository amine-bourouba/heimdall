// Where the main window goes when the tray asks (`open_in_main_window` -> `navigate`, M3-58): a session
// opens its detail, a screen opens that screen. The same two destinations the search overlay reaches,
// without its overlay side effects. A screen with no sidebar entry is ignored rather than guessed at.
import type { Router } from 'vue-router';
import type { NavTarget } from '@/ipc/bindings';
import { navigationItems } from './navigationItems';

export function openTarget(router: Pick<Router, 'push'>, target: NavTarget) {
  if (target.kind === 'session') {
    void router.push({ name: 'session-detail', params: { projectId: target.projectId, sessionId: target.sessionId } });
    return;
  }
  const item = navigationItems.find((i) => i.id === target.screen);
  if (item) void router.push(item.to);
}
