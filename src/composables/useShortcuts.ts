// Global shortcuts (parity-matrix.md section J1-J5, J11; ContentView.swift:38-54,
// PoirotApp.swift:81-96). Every binding dispatches a named AppAction through useAppActions, the
// AC's own "single dispatcher" -- this file also registers the real handlers for the actions
// whose store/route already exists (theme's font scale, M2-05; back/forward, M2-06's
// useNavigation; settings, M3-16 -- `router.push`, since `/settings` is a plain route with no
// sidebar entry of its own to click instead). The rest (search, toolFilter, help) still dispatch
// into a registry with no handler yet -- a silent no-op until the feature that owns each one
// exists, not an error.

import { onMounted, onScopeDispose } from 'vue';
import { useRouter } from 'vue-router';
import type { AppAction } from '@/ipc/bindings';
import { useAppActions } from './useAppActions';
import { useNavigation } from './useNavigation';
import { usePlatformStore } from '@/stores/platform';
import { useThemeStore } from '@/stores/theme';

/** macOS's Mod is metaKey (⌘); everywhere else it's ctrlKey (docs/parity-matrix.md section J:
 * "On Windows and Linux, ⌘ becomes Ctrl everywhere"). */
function isMod<R extends KeyboardEvent>(event: R, isMac: boolean): boolean {
  return isMac ? event.metaKey : event.ctrlKey;
}

const BINDINGS: { key: string; action: AppAction }[] = [
  { key: 'k', action: 'search' },
  { key: '[', action: 'back' },
  { key: ']', action: 'forward' },
  { key: 't', action: 'toolFilter' },
  { key: '=', action: 'fontIncrease' },
  { key: '-', action: 'fontDecrease' },
  { key: '0', action: 'fontReset' },
  { key: '?', action: 'help' },
  { key: ',', action: 'settings' },
];

export function useShortcuts() {
  const { dispatch, on } = useAppActions();
  const platformStore = usePlatformStore();
  const themeStore = useThemeStore();
  const router = useRouter();
  const { goBack, goForward } = useNavigation();

  const unregister = [
    on('fontIncrease', () => themeStore.increaseFontScale()),
    on('fontDecrease', () => themeStore.decreaseFontScale()),
    on('fontReset', () => themeStore.resetFontScale()),
    on('back', goBack),
    on('forward', goForward),
    on('settings', () => void router.push({ name: 'settings' })),
  ];

  function onKeydown(event: KeyboardEvent) {
    const isMac = platformStore.os === 'macos';
    if (!isMod(event, isMac)) return;
    const binding = BINDINGS.find((b) => b.key === event.key.toLowerCase());
    if (!binding) return;
    event.preventDefault();
    dispatch(binding.action);
  }

  onMounted(() => document.addEventListener('keydown', onKeydown));
  onScopeDispose(() => {
    document.removeEventListener('keydown', onKeydown);
    unregister.forEach((off) => off());
  });
}
