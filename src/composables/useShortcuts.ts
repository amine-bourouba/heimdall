// Global shortcuts (parity-matrix.md section J1-J5, J11; ContentView.swift:38-54,
// PoirotApp.swift:81-96). Every binding dispatches a named AppAction through useAppActions, the
// AC's own "single dispatcher" -- this file also registers the real handlers for the actions
// whose store/route already exists (theme's font scale, M2-05; back/forward, M2-06's
// useNavigation; settings/help/about, M3-16/M3-17 -- plain `router.push`, since none of
// `/settings`, `/help`, `/about` have a sidebar entry of their own to click instead). `search` and
// `toolFilter` are still a no-op here -- each is registered by the feature component that owns it
// instead (`SearchOverlay.vue`, `SessionDetail.vue`), the same reasoning `shortcuts` (M3-17,
// `ShortcutHelp.vue`'s own always-mounted overlay) now follows too.
//
// `BINDINGS` is exported (M3-17, port-plan.md decision P-7): `ShortcutHelp.vue` renders its list
// straight from this one array so the sheet can never drift from what Mod-key actually does.
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

export const BINDINGS: { key: string; action: AppAction }[] = [
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
    on('help', () => void router.push({ name: 'help' })),
    on('about', () => void router.push({ name: 'about' })),
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
