<script setup lang="ts">
// Ported from ContentView.swift's `splitView`: sidebar + detail (RouterView here, M2-06) with
// navigation/contextual toolbars and overlay mount points. Three startup steps this task's own
// dependencies left as explicit follow-ups, done here because this is the first component Vue
// actually mounts into a real `<html>`:
//  - `platformStore.load()`/`prefsStore.load()` + `themeStore.applyToDocument()` (flagged by
//    M2-05/M2-06 -- nothing before this task ever ran after `<html>` existed to call them).
//  - `document.documentElement.dataset.os` for `--titlebar-inset` (main.css) to key off of.
//  - The real `menu-action` IPC listener -> `useAppActions().dispatch(...)` (flagged by M2-13):
//    `useShortcuts.ts` already registers real handlers for `fontIncrease`/`fontDecrease`/
//    `fontReset`/`back`/`forward` in that same registry, so wiring the listener is the only
//    piece those five actions were missing to work end-to-end from the native macOS menu too.
//  - `useShortcuts()` itself (flagged by M2-06/M2-09's own comment just above) was never actually
//    called anywhere outside its own spec -- its document-level keydown listener was dead code,
//    so Mod+T (M2-20's own "toggles the tool filter") could never have fired regardless of
//    whether `SessionDetail.vue` registered a handler for it. Calling it here is this task's own
//    fix, not new scope: it is the one call site that makes every binding in `useShortcuts.ts`
//    (search, back/forward, tool filter, font scale, help, settings) live at all.
import { onMounted, onUnmounted } from 'vue';
import { onMenuAction } from '@/ipc/client';
import { usePlatformStore } from '@/stores/platform';
import { usePrefsStore } from '@/stores/prefs';
import { useThemeStore } from '@/stores/theme';
import { useAppActions } from '@/composables/useAppActions';
import { useShortcuts } from '@/composables/useShortcuts';
import ToastOverlay from '@/components/ToastOverlay.vue';
import Sidebar from './Sidebar.vue';
import Toolbar from './Toolbar.vue';
import AppMenu from './AppMenu.vue';
import SearchOverlay from '@/features/search/SearchOverlay.vue';
import Onboarding from '@/features/onboarding/Onboarding.vue';
import ShortcutHelp from '@/features/help/ShortcutHelp.vue';

const platformStore = usePlatformStore();
const prefsStore = usePrefsStore();
const themeStore = useThemeStore();
useShortcuts();

let unlistenMenuAction: (() => void) | undefined;

onMounted(async () => {
  await Promise.all([platformStore.load(), prefsStore.load()]);
  document.documentElement.dataset.os = platformStore.os ?? undefined;
  themeStore.applyToDocument();
  unlistenMenuAction = await onMenuAction((action) => useAppActions().dispatch(action));
});

onUnmounted(() => unlistenMenuAction?.());
</script>

<template>
  <div class="flex h-screen w-screen overflow-hidden bg-(--bg-app) text-(--text-primary)">
    <Sidebar />
    <div class="flex flex-1 flex-col overflow-hidden">
      <div class="flex items-center justify-between pt-(--titlebar-inset)">
        <Toolbar />
        <AppMenu v-if="platformStore.os && platformStore.os !== 'macos'" />
      </div>
      <main class="flex-1 overflow-y-auto">
        <RouterView />
      </main>
    </div>

    <ToastOverlay />
    <SearchOverlay />
    <Onboarding v-if="prefsStore.prefs && !prefsStore.prefs.hasCompletedOnboarding" />
    <ShortcutHelp />
  </div>
</template>
